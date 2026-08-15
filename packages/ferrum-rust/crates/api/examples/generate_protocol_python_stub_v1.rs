//! Regenerate only the marked Ferrum operation protocol V1 stub section.

use std::{fs, path::PathBuf};

const BEGIN: &str = "# BEGIN GENERATED FERRUM OPERATION PROTOCOL V1";
const END: &str = "# END GENERATED FERRUM OPERATION PROTOCOL V1";
const SECTION: &str = r#"# BEGIN GENERATED FERRUM OPERATION PROTOCOL V1
# Generated from the checked-in Rust-owned operation protocol V1 schema.
class OperationProtocolErrorV1(FerrumError):
	category: str


def execute_operation_v1(request_json: str) -> str: ...
def operation_protocol_schema_v1() -> str: ...
# END GENERATED FERRUM OPERATION PROTOCOL V1"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let schema: serde_json::Value =
        serde_json::from_str(ferrum_api::operation_protocol_schema_v1())?;
    if schema.get("x-ferrum-roots").is_none() {
        return Err("operation protocol schema lacks its generated roots".into());
    }
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let destination = manifest.join("wheel_metadata/ferrum_chem.pyi");
    let current = fs::read_to_string(&destination)?;
    let start = current
        .find(BEGIN)
        .ok_or("protocol stub begin marker is missing")?;
    let end = current[start..]
        .find(END)
        .map(|offset| start + offset + END.len())
        .ok_or("protocol stub end marker is missing")?;
    let mut updated = String::with_capacity(current.len() - (end - start) + SECTION.len());
    updated.push_str(&current[..start]);
    updated.push_str(SECTION);
    updated.push_str(&current[end..]);
    fs::write(destination, updated)?;
    Ok(())
}
