# stt-server

A standalone, GPU-accelerated **Speech-to-Text server** — point any
Deepgram-compatible `/v1/listen` client at it and transcribe. AMD (Vulkan),
NVIDIA (CUDA), and CPU images, a built-in Tailscale sidecar for tailnet
exposure, and a web dashboard. Published to GHCR.

> Decoupled from [Notare](https://github.com/abhi-wan-kenobi/notare)'s
> `apps/stt-server` (issue #14) into this standalone project on 2026-07-24.
> See `docs/architecture.md` for the rationale and the decoupling plan.

## Quickstart

Pick the image that matches your hardware:

| Image | Backend | Hardware |
|---|---|---|
| `ghcr.io/abhi-wan-kenobi/stt-server:cpu` | tinyBLAS CPU | Any (no GPU) |
| `ghcr.io/abhi-wan-kenobi/stt-server:vulkan` | ggml Vulkan | AMD RDNA2/3, Intel Arc, NVIDIA (via Vulkan) |
| `ghcr.io/abhi-wan-kenobi/stt-server:cuda` | ggml CUDA | NVIDIA |

```sh
# AMD / Intel / NVIDIA-via-Vulkan (default)
docker run --rm -p 8383:8383 --device /dev/dri --group-add video \
  -v stt-models:/data/models \
  ghcr.io/abhi-wan-kenobi/stt-server:vulkan

# NVIDIA CUDA (needs NVIDIA Container Toolkit)
docker run --rm -p 8383:8383 --gpus all \
  -v stt-models:/data/models \
  ghcr.io/abhi-wan-kenobi/stt-server:cuda

# CPU only
docker run --rm -p 8383:8383 -v stt-models:/data/models \
  ghcr.io/abhi-wan-kenobi/stt-server:cpu
```

Then open `http://localhost:8383/` for the dashboard, or hit the API:

```sh
curl http://127.0.0.1:8383/health           # ok
curl http://127.0.0.1:8383/api/status | jq  # engine + GPU backend + loaded model
curl http://127.0.0.1:8383/api/models | jq  # catalog + on-disk integrity
```

## Tailscale exposure (tailnet-only by default)

For non-loopback access (the Notare desktop client requires HTTPS/wss on
custom servers), use the reference compose with a Tailscale sidecar:

```sh
cp docker/.env.example docker/.env  # fill TS_AUTHKEY + STT_TOKEN
docker compose -f docker/docker-compose.tailscale.yml up -d
```

Result: `https://stt.<tailnet>.ts.net/v1` — real ts.net cert, tailnet-only
(Serve, **not** Funnel). Funnel (public internet) is an opt-in toggle on the
landing page with a warning. See `docs/architecture.md` § "Tailscale sidecar".

## Config (flags / env vars)

| Flag | Env var | Default | Notes |
|---|---|---|---|
| `--host` | `STT_HOST` | `0.0.0.0` | `127.0.0.1` for loopback only. |
| `--port` | `STT_PORT` | `8383` | |
| `--model-dir` | `STT_MODEL_DIR` | `./data/models` | Model catalog root. |
| `--model` | `STT_MODEL` | `QuantizedSmall` | Whisper catalog id. |
| `--require-gpu` | `STT_REQUIRE_GPU` | `false` | Refuse to start without a verified GPU backend. `true` on GPU images. |
| `--token` | `STT_TOKEN` | unset | Optional bearer gate on `/v1/listen` + mutation routes. |
| `--health-interval-secs` | `STT_HEALTH_INTERVAL_SECS` | `300` | Periodic RTF health monitor. |
| `--health-min-rtf` | `STT_HEALTH_MIN_RTF` | `5.0` | Min realtime factor to stay healthy. |
| `--health-fail-streak` | `STT_HEALTH_FAIL_STREAK` | `2` | Consecutive low probes before 503/exit. |
| `--health-autorestart` | `STT_HEALTH_AUTORESTART` | `true` | Exit on sustained degradation so the supervisor restarts. |

## Endpoints

- `GET /` — embedded web dashboard (server status, GPU backend, model catalog + download/activate, Tailscale toggle).
- `GET /health` — `"ok"` (503 on sustained RTF degradation).
- `POST /v1/listen?channels=&sample_rate=` — batch transcription (`Accept: text/event-stream` for SSE progress).
- `GET /v1/listen?channels=&sample_rate=` (WebSocket) — live streaming.
- `GET /api/status` — version, engine, loaded model, GPU backends, offload state, RTF, uptime.
- `GET /api/models` — catalog + per-model integrity + download progress.
- `POST /api/models/{id}/download` · `GET /api/models/{id}/progress` · `POST /api/models/{id}/cancel` · `DELETE /api/models/{id}` · `POST /api/models/{id}/activate`.

Full route docs + curl examples: `apps/stt-server/README.md`.

## Security

Plaintext HTTP, no auth by default — **keep it on a trusted LAN or tailnet**.
Never port-forward it. Set `STT_TOKEN` for a shared-secret gate; use the
Tailscale sidecar for remote access. See `apps/stt-server/README.md` § Security.

## Build from source

```sh
cargo check -p stt-server
cargo run -p stt-server -- --model-dir ./data/models
```

Docker images: `apps/stt-server/Dockerfile.{cpu,vulkan,cuda}`.

## License

MIT — see `LICENSE`. Decoupled from Notare (also MIT).
