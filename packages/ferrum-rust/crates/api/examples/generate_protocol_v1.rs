//! Regenerate the checked-in Ferrum operation protocol V1 schema.

use std::{fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let destination = manifest.join("protocol/ferrum-operation-v1.schema.json");
    let schema = ferrum_api::generated_operation_protocol_schema_v1();
    let rendered = serde_json::to_string_pretty(&schema)?;
    fs::write(destination, format!("{rendered}\n"))?;
    Ok(())
}
