#[doc(hidden)]
pub struct HiddenRawAdapter;

#[doc(hidden)]
#[macro_export]
macro_rules! hidden_raw_adapter_macro {
    () => {};
}

include!("generated_public_raw.rs");

mod raw_adapter {
    pub struct ReexportedRawBuffer;
}

pub use raw_adapter::ReexportedRawBuffer;
