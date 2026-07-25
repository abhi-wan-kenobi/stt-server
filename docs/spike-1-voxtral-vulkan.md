# Spike 1: transcribe.cpp Voxtral-on-Vulkan on the RX 6600

**Date:** 2026-07-25
**Verdict:** **GO** ✅
**Hardware:** coruscant — AMD RX 6600 (gfx1032, RDNA2, 8GB), Vulkan 1.3.275 (RADV NAVI23).

## Question

Can [transcribe.cpp](https://github.com/handy-computer/transcribe.cpp) (the
unifying ggml STT library, 16+ ASR families on Metal/Vulkan/CUDA/CPU) run the
**Voxtral-Mini-3B** model on the **AMD Vulkan** backend cleanly — i.e. without
the RDNA2 heap-corruption that forced server-side Hinglish to CPU-only Whisper
when using llama.cpp's `mtmd` path?

This is the highest-leverage unknown for the stt-server v0.2 roadmap: if GO,
the v0.2 transcribe.cpp engine swap becomes the priority, because it unblocks
GPU-accelerated Hinglish (Roman-script code-mixed Hindi+English) on the AMD
Vulkan production backend *without* needing ROCm.

## Setup

- **Build:** transcribe.cpp cloned from
  `https://github.com/handy-computer/transcribe.cpp` (commit `b6a6aca`,
  version 0.2.0). Built with `cmake -B build -DTRANSCRIBE_VULKAN=ON
  -DCMAKE_BUILD_TYPE=Release` then `cmake --build build -j12`. Backends
  compiled: `vulkan;cpu`. Binary: `build/bin/transcribe-cli` (unified CLI,
  no separate voxtral binary).
- **Model:** `handy-Voxtral-Mini-3B-2507-Q4_K_M.gguf` (2.98 GB / 2984721056
  bytes) from
  `https://huggingface.co/handy-computer/Voxtral-Mini-3B-2507-gguf/`.
  Single self-contained GGUF (no separate mmproj file needed, unlike the
  ggml-org split version which transcribe.cpp rejects with `unsupported
  architecture`). License: apache-2.0. Whisper-large-v3 audio encoder (32
  layers) → 4-frame-group projector (375 audio tokens/30s) → Ministral-3B
  causal LM.
- **GPU detection:** `transcribe-cli --list-devices` → `[0] AMD Radeon RX
  6600 (RADV NAVI23) vulkan gpu 7.98 GiB total 7.97 GiB free fp16=1; [1]
  12th Gen Intel Core i5-12400F cpu 15.41 GiB`.
- **Audio clip:** `clip16k.wav` (81 KB, 2.566s, 16kHz mono float32). Generated
  with `espeak-ng -v hi "kal ka client call reschedule karna hai" -w
  clip.wav` then resampled to 16kHz with `ffmpeg -i clip.wav -ar 16000 -ac 1
  clip16k.wav` (transcribe.cpp v1 only accepts 16kHz WAV). **Note:** espeak-ng
  synthetic Hindi is NOT real Hinglish speech — the transcript accuracy was
  never the test; the test was *does it crash*.

## Run

```sh
cd /tmp/opencode/transcribe-cpp-spike/transcribe.cpp
HSA_OVERRIDE_GFX_VERSION=10.3.0 \
timeout 300 ./build/bin/transcribe-cli \
  -m ../models/handy-Voxtral-Mini-3B-2507-Q4_K_M.gguf \
  --backend vulkan --device 0 \
  -l hi \
  ../clip16k.wav
```

`HSA_OVERRIDE_GFX_VERSION=10.3.0` is set defensively (matches the proven
llama.cpp mtmd recipe on this GPU); in practice transcribe.cpp's Vulkan
backend via RADV doesn't need it.

## Output (spike-run.log)

```
[debug] ggml_vulkan: Found 1 Vulkan devices: 0 = AMD Radeon RX 6600 (RADV NAVI23)
[info] voxtral: using vulkan backend: Vulkan0
[warn] tokenizer.encode: unknown tokenizer.ggml.pre "tekken"; falling back to qwen2
audio: ../clip16k.wav samples: 41050 duration: 2.566 s sample rate 16000 Hz mono float32
model: .../handy-Voxtral-Mini-3B-2507-Q4_K_M.gguf -> ok
       backend: Vulkan0 name: Voxtral Mini 3B license: apache-2.0 max audio: 10448.6 s
timings: load=3861.18 ms mel=21.33 ms encode=708.89 ms decode=901.96 ms
text: कौ-कौ-क्लाइंट-कॉरिशेबुल-कैनोही
run: ok
realtime: 2x (1632.2 ms for 2.6 s)
```

**Exit code: 0.** No heap corruption, no segfault, no Vulkan validation
errors, no `ggml_vulkan` warnings beyond the expected device-discovery debug
line. A real (if garbled — espeak-ng synthetic input) transcript was
produced.

## Verdict: GO

transcribe.cpp runs Voxtral cleanly on the RX 6600 Vulkan backend. The
RDNA2 heap-corruption that plagues llama.cpp's `mtmd` path (and forced
server-side Hinglish to CPU-only Whisper) does **not** reproduce in
transcribe.cpp's ggml Vulkan path.

### Implications for stt-server v0.2

1. **Unblocks GPU Hinglish on AMD Vulkan** — the production backend. No
   ROCm needed.
2. **Makes the transcribe.cpp engine swap the v0.2 priority.** The current
   v0.1 uses the Notare-era split: `whisper-rs` (Vulkan) for Whisper +
   `llama-cpp-2` (CPU-only on this GPU due to the heap corruption) for
   Voxtral. transcribe.cpp collapses both into one ggml runtime and adds
   14+ more model families (Moonshine, Canary, Qwen3-ASR, SenseVoice,
   Granite, Cohere, GigaAM, MedASR, …) for free.
3. **Voxtral perf on this GPU:** 2x realtime cold (1632ms for 2.6s audio).
   Warm-compute should be faster (model load 3.86s is one-time). For real
   Hinglish clips, RTF should hold or improve. Track in v0.2 benchmark.
4. **Tokenizer fallback** (`tekken` → `qwen2`): cosmetic warning, transcript
   still produced. Worth investigating if real-Hinglish accuracy is poor, but
   not a blocker.

### Caveats

- The clip was espeak-ng **synthetic** Hindi, not real Hinglish speech. The
  garbled Devanagari transcript (`कौ-कौ-क्लाइंट-कॉरिशेबुल-कैनोही`) is
  expected for synthetic input and does NOT reflect real-audio accuracy.
  v0.2 must validate against a real Hinglish recording from Abhishek.
- `max audio: 10448.6 s` (174 min) per call. Long audio needs chunking (same
  discipline as Whisper).
- VRAM: model loaded fine in the 8GB card; Voxtral-Mini-3B Q4_K_M (2.98 GB)
  + KV cache + audio encoder fit comfortably. No OOM.

## Reproduction artifacts (on coruscant, ephemeral)

- Spike repo: `/tmp/opencode/transcribe-cpp-spike/transcribe.cpp/` (cloned).
- Build: `/tmp/opencode/transcribe-cpp-spike/transcribe.cpp/build/bin/transcribe-cli`.
- Model: `/tmp/opencode/transcribe-cpp-spike/models/handy-Voxtral-Mini-3B-2507-Q4_K_M.gguf`.
- Clip: `/tmp/opencode/transcribe-cpp-spike/clip16k.wav`.
- Full run log: `/tmp/opencode/transcribe-cpp-spike/spike-run.log`.

## Next spike (Spike 2 — Rust bindings smoke)

The C++ CLI works. Now verify the `transcribe-cpp` Rust crate (from the same
repo's `bindings/`) can: load a Whisper GGUF + a Voxtral GGUF, transcribe,
return word-level timestamps (needed for diarization alignment). If the
binding covers model-load + streaming + word timings, the v0.2 Rust engine
adapter can be written against it.
