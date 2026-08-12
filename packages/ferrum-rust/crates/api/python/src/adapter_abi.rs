use std::fmt;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct UnsupportedAdapterAbiVersion {
    found: u32,
    supported: u32,
}

impl fmt::Display for UnsupportedAdapterAbiVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Ferrum-Chem adapter ABI {} is unsupported; this Ferrum API requires ABI {}",
            self.found, self.supported
        )
    }
}

pub(crate) fn ensure_supported_adapter_abi_version(
    found: u32,
    supported: u32,
) -> Result<(), UnsupportedAdapterAbiVersion> {
    if found == supported {
        Ok(())
    } else {
        Err(UnsupportedAdapterAbiVersion { found, supported })
    }
}
