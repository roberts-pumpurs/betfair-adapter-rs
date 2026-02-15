# Memory

## Project: betfair-adapter-rs
- Rust workspace, edition 2024, MSRV 1.91, pre-1.0 (v0.6.8)
- `cargo xtask check` runs clippy + fmt; `cargo xtask test` runs nextest
- `bytes` crate added as workspace dep (v1) for zero-copy raw message support
- `StreamAPIClientCodec::Decoder::Item` is `(bytes::Bytes, ResponseMessage)` — raw bytes preserved via `BytesMut::freeze()`
- `MessageProcessor` trait has `on_raw_message(&mut self, raw: &[u8])` with default no-op