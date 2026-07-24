# Architecture

## The engine: transcribe.cpp (ggml)

`stt-server` is a thin HTTP/WebSocket service over
[`transcribe.cpp`](https://github.com/handy-computer/transcribe.cpp) — a
ggml-based STT inference library that runs **16+ ASR model families** on one
runtime with Metal / Vulkan / CUDA / tinyBLAS-CPU backends.

This replaces the three separate stacks the Notare monorepo's STT server
carried:

| Notare (old) | stt-server (new) |
|---|---|
| `whisper-rs` (Vulkan) for Whisper | `transcribe-cpp` (Vulkan) for Whisper + 15 more families |
| `llama-cpp-2` (ROCm) for Voxtral Hinglish | `transcribe-cpp` (Vulkan) for Voxtral — one backend |
| `ort` / ONNX Runtime for Parakeet | `transcribe-cpp` for Parakeet (no second onnxruntime) |

One runtime, one backend matrix, one model catalog.

## Why Vulkan, not ROCm (on AMD)

The Notare monorepo experimented with a `Dockerfile.rocm` (whisper-rs
`hipblas` feature, `AMDGPU_TARGETS=gfx1030`, `HSA_OVERRIDE_GFX_VERSION=10.3.0`
for the RX 6600 gfx1032). Findings:

- **ROCm was ~28% faster on `QuantizedSmall`** (Wave-1 result).
- **But ~40% SLOWER on `QuantizedLargeTurbo`** (7.43x vs Vulkan 13.46x on the
  same probe) — the production model. Reverted.
- ROCm adds a heavy SDK + a per-GPU Tensile-library gap (gfx1032 has no
  shipped Tensile lib → must build as gfx1030 + runtime override).
- `transcribe.cpp` has **no HIP/ROCm backend** — only Metal/Vulkan/CUDA. So
  even if we wanted ROCm, the unifying engine doesn't offer it.

**Decision:** Vulkan is the AMD production path. The `Dockerfile.rocm` is
**not** carried into this project. (If a future transcribe.cpp HIP backend
lands and beats Vulkan on the production model, revisit.)

## GPU backend selection

| Image | Backend | Hardware |
|---|---|---|
| `stt-server:cpu` | tinyBLAS CPU | Any (no GPU) |
| `stt-server:vulkan` | ggml Vulkan | AMD RDNA2/3, Intel Arc, NVIDIA (via Vulkan) |
| `stt-server:cuda` | ggml CUDA | NVIDIA |

`STT_REQUIRE_GPU=true` (default on GPU images) refuses to start if the backend
isn't working — the admin panel surfaces a `CPU Fallback` state if it slips
through.

## Tailscale sidecar

The server itself is plain HTTP. For non-loopback exposure (the Notare desktop
client requires HTTPS/wss on custom servers), ship a Tailscale sidecar that
shares the network namespace and terminates HTTPS via Tailscale **Serve** on a
`*.ts.net` name — tailnet-only by default. The landing-page toggle walks the
operator through auth-key + hostname + Serve/Funnel setup. Funnel (public
internet) is opt-in with a warning.

Reference compose: `docker/docker-compose.tailscale.yml`.

## Reliability (carried over from notare-stt-server, proven in prod)

- **RTF health-check** — periodic speech-probe (not silence); on sustained
  degradation flips `/health` to 503 + exits cleanly so the container
  supervisor restarts (clears Vulkan/driver state).
- **WS keepalive + bounded writes** — every `send().await` in the streaming
  loop is bounded by a 20s timeout so a Tailscale path hiccup can't freeze
  the loop forever (the 88%-hang fix).
- **Batch VAD chunk-packing** — pack adjacent VAD chunks up to 25s before
  transcribing (the 1x→13-15x speedup; whisper.cpp's GPU encoder pays a
  fixed 30s-window cost per call).
- **Startup model reconciliation** — every installed model re-verified
  (existence + size + hash) on boot; corrupt ones quarantined.

## What stays in Notare

Only the **server** moves out. The Notare desktop client's **in-process local
STT** (`crates/transcribe-*`, `plugins/transcription`) stays — that's the
on-device privacy moat. The client's "Custom STT server" provider already
speaks this server's `/v1/listen` contract, so it needs zero code change, just
a docs pointer to the new GHCR images.