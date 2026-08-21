//! Native adapter implementation of the safe chemistry engine.
//!
//! This facade keeps the native engine in one Rust module while separating its
//! public adapter surface from response decoding.

include!("native_engine/engine_api.rs");
include!("native_engine/engine_codec.rs");
