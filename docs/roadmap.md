# Roadmap

## Done (scaffold, 2026-07-24)
- [x] Decoupling plan + vault doc.
- [x] Cargo workspace scaffold (`apps/stt-server`, `crates/transcribe-engine`,
      `crates/model-catalog`).
- [x] Model catalog (9 entries across 7 families) + `default_for_language`.
- [x] Server skeleton (config, router, state, assets, Dockerfiles, GHCR
      workflow, Tailscale compose, README, architecture doc).
- [x] Landing page with Tailscale toggle + dashboard placeholder.

## Spike 1 — transcribe.cpp Voxtral-on-Vulkan on the RX 6600 (HIGHEST LEVERAGE)
The RDNA2 Vulkan heap-corruption that forced server-side Hinglish to CPU-only
Whisper was in **llama.cpp's mtmd path**, not necessarily transcribe.cpp's
ggml Voxtral path. If transcribe.cpp runs Voxtral clean on Vulkan, the Hinglish
story is unlocked on the AMD production backend (no ROCm needed).

- [ ] Build transcribe.cpp with `-DTRANSCRIBE_VULKAN=ON` on coruscant.
- [ ] Load `voxtral-mini-3b-2507-Q4_K_M.gguf` + mmproj.
- [ ] Transcribe a Hinglish clip; assert no heap corruption + accurate transcript.
- [ ] Write up in `docs/spike-1-voxtral-vulkan.md`.
- Delegate the mechanical build to an Ollama coder; keep judgment (accuracy,
  no corruption) for Abhishek/Claude.

## Spike 2 — Rust bindings smoke
- [ ] `transcribe-cpp` Rust crate: load Whisper + Parakeet GGUF, transcribe,
      assert word timestamps. Confirms the binding covers model load +
      streaming + word timings (needed for diarization alignment).

## Port the server from notare-stt
- [ ] Engine adapter: `transcribe-engine` concrete `TranscribeCppEngine`
      (replace the placeholder `transcribe()` bail).
- [ ] Router: port the full route handlers from
      `notare/apps/stt-server/src/router.rs` (424 lines) — CORS allowlist,
      origin check, error envelope, WS handler with bounded writes.
- [ ] State: port `SessionRegistry`, `ModelDownloadManager`, health-check.
- [ ] Auth: port the bearer-token gate.
- [ ] Health: port the RTF health-check (probe-uses-speech).
- [ ] Assets: port + adapt `index.html` (886 lines) + `dashboard.html` (156
      lines) — add model-family/language picker, GPU-util panel, Prometheus
      metrics view.
- [ ] `cargo test --locked --workspace` green.

## Multi-family catalog + download
- [ ] Wire HF download (handy-computer GGUFs) + SHA-256/CRC32 integrity.
- [ ] Expand catalog to the full 16-family matrix (Canary, Canary-Qwen,
      Granite, Cohere, GigaAM, MedASR, MOSS, FunASR, Moonshine Streaming,
      Multitalker Parakeet).
- [ ] Per-model language coverage metadata in the landing page.

## Tailscale toggle (full)
- [ ] Landing-page setup dialog (auth key, hostname, Serve/Funnel).
- [ ] Sidecar `serve.json` generation + sidecar restart via Docker socket
      (compose) or `tailscale serve` command emission (bare metal).

## Dashboard extensions
- [ ] `/api/metrics` Prometheus endpoint (counters, RTF, GPU util).
- [ ] GPU utilization: `amdgpu_top`/`nvtop` sidecar metrics (Vulkan) /
      `nvidia-smi` (CUDA).
- [ ] Request/error counters, WS disconnect reasons.

## Publish + cutover
- [ ] GHCR workflow live; `ghcr.io/abhi-wan-kenobi/stt-server:{cpu,vulkan,cuda}` pullable.
- [ ] Coruscant cutover: swap `notare-stt:vulkan` for `stt-server:vulkan`,
      same Tailscale sidecar + token; soak-verify ≥1 week.
- [ ] Notare backlog: remove `apps/stt-server` + Dockerfiles + workflow from
      the Notare repo (see decoupling plan §6).
- [ ] Notare client docs: point "Custom STT server" at the new GHCR image.