//! `stt-server` — standalone, GPU-accelerated, multilingual STT server.
//!
//! Decoupled from the Notare monorepo (2026-07-24). Engine: `transcribe.cpp`
//! (ggml) — 16+ ASR model families on Metal/Vulkan/CUDA/CPU. API: the
//! `/v1/listen` (batch + WebSocket + SSE) contract the Notare desktop client
//! already speaks, unchanged.

pub mod admin;
pub mod assets;
pub mod auth;
pub mod config;
pub mod health;
pub mod probe;
pub mod router;
pub mod state;