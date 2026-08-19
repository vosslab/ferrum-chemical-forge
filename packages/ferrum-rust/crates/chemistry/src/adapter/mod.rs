//! Deliberate native-adapter admission for callers that opt into dynamic loading.

mod explicit;

pub use explicit::{ExplicitAdapterError, load_explicit_adapter};
