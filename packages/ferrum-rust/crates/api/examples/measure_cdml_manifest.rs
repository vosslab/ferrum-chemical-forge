//! Consented, developer-only measurement of explicitly selected CDML and CD-SVG files.
//!
//! This Cargo example is intentionally outside the normal Ferrum CLI. It records no paths,
//! document content, names, hashes, or parser diagnostics. The normal local-file check below is
//! not a race-free privileged-file claim: a path can still change after `symlink_metadata` and
//! before open, as with the product's existing desktop-file ingress.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::os::fd::OwnedFd;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ferrum_document::{
    CdsvgExtractionError, TypedDocument, XmlInputError, XmlInputMeasurementV1,
    measure_cdsvg_input_v1, measure_xml_input_v1,
};
use rustix::fs::{AtFlags, CWD, FileType, Mode, OFlags, fsync, openat, renameat, statat, unlinkat};
use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Value, json};

const SCHEMA: &str = "ferrum-cdml-measurement-manifest-v1";
const RECEIPT_SCHEMA: &str = "ferrum-cdml-measurement-receipt-v1";
const TEMPORARY_ATTEMPTS: u8 = 16;
static NEXT_TEMPORARY_FILE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug)]
struct Manifest {
    collection_id: String,
    ceiling: usize,
    samples: Vec<Sample>,
}

#[derive(Debug)]
struct Sample {
    alias: String,
    path: PathBuf,
    format: Format,
    stratum: String,
    producer: Option<Producer>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Format {
    Cdml,
    Cdsvg,
}

impl Format {
    fn as_str(self) -> &'static str {
        match self {
            Self::Cdml => "cdml",
            Self::Cdsvg => "cdsvg",
        }
    }
}

#[derive(Debug)]
struct Producer {
    name: String,
    version: Option<String>,
}

#[derive(Debug)]
enum StrictValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<StrictValue>),
    Object(BTreeMap<String, StrictValue>),
}

impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("strict JSON")
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue::Null)
            }
            fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(StrictValue::Null)
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue::Bool(value))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue::Number(value.into()))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue::Number(value.into()))
            }
            fn visit_f64<E: de::Error>(self, value: f64) -> Result<Self::Value, E> {
                serde_json::Number::from_f64(value)
                    .map(StrictValue::Number)
                    .ok_or_else(|| E::custom("invalid number"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue::String(value.to_owned()))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue::String(value))
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(value) = access.next_element()? {
                    values.push(value);
                }
                Ok(StrictValue::Array(values))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = access.next_entry::<String, StrictValue>()? {
                    if values.insert(key.clone(), value).is_some() {
                        return Err(de::Error::custom(format!("duplicate key {key}")));
                    }
                }
                Ok(StrictValue::Object(values))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}

fn main() {
    let result = run();
    if result.is_err() {
        eprintln!("CDML measurement did not complete");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ()> {
    let (manifest_path, receipt_path) = parse_args()?;
    let manifest_bytes = fs::read(&manifest_path).map_err(|_| ())?;
    let manifest = parse_manifest(&manifest_bytes).map_err(|_| ())?;
    reject_output_aliases(&manifest_path, &receipt_path, &manifest.samples).map_err(|_| ())?;
    let mut entries = Vec::new();
    for sample in &manifest.samples {
        entries.push(measure_sample(sample, manifest.ceiling));
    }
    let any_failed = entries.iter().any(|entry| entry["outcome"] != "measured");
    let receipt = receipt(&manifest, entries);
    let encoded = serde_json::to_vec_pretty(&receipt).map_err(|_| ())?;
    publish_receipt(&receipt_path, &encoded).map_err(|_| ())?;
    if any_failed { Err(()) } else { Ok(()) }
}

fn parse_args() -> Result<(PathBuf, PathBuf), ()> {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.len() != 4 || args[0] != "--manifest" || args[2] != "--receipt" {
        return Err(());
    }
    Ok((PathBuf::from(&args[1]), PathBuf::from(&args[3])))
}

fn parse_manifest(bytes: &[u8]) -> Result<Manifest, ()> {
    // This developer configuration is intentionally read as a small explicit local file; it is
    // not a product document ingress and selects no production allocation limit.
    let strict: StrictValue = serde_json::from_slice(bytes).map_err(|_| ())?;
    let root = object(&strict)?;
    exact_keys(
        root,
        &[
            "schema",
            "consented",
            "collection_id",
            "collection_read_ceiling_bytes",
            "samples",
        ],
    )?;
    if string(required(root, "schema")?)? != SCHEMA || !boolean(required(root, "consented")?)? {
        return Err(());
    }
    let collection_id = safe_identifier(string(required(root, "collection_id")?)?)?;
    let ceiling = positive_usize(required(root, "collection_read_ceiling_bytes")?)?;
    let values = array(required(root, "samples")?)?;
    if values.is_empty() {
        return Err(());
    }
    let mut aliases = BTreeSet::new();
    let mut samples = Vec::with_capacity(values.len());
    for value in values {
        let sample = parse_sample(value)?;
        if !aliases.insert(sample.alias.clone()) {
            return Err(());
        }
        samples.push(sample);
    }
    Ok(Manifest {
        collection_id,
        ceiling,
        samples,
    })
}

fn parse_sample(value: &StrictValue) -> Result<Sample, ()> {
    let object = object(value)?;
    exact_keys(object, &["alias", "path", "format", "stratum", "producer"])?;
    let alias = safe_alias(string(required(object, "alias")?)?)?;
    let path = PathBuf::from(string(required(object, "path")?)?);
    if !path.is_absolute() {
        return Err(());
    }
    let format = match string(required(object, "format")?)? {
        "cdml" => Format::Cdml,
        "cdsvg" => Format::Cdsvg,
        _ => return Err(()),
    };
    let stratum = match string(required(object, "stratum")?)? {
        "typical" | "largest" | "legacy" | "legacy_opaque" | "cdsvg" => {
            safe_identifier(string(required(object, "stratum")?)?)?
        }
        _ => return Err(()),
    };
    let producer = match object.get("producer") {
        None => None,
        Some(value) => Some(parse_producer(value)?),
    };
    Ok(Sample {
        alias,
        path,
        format,
        stratum,
        producer,
    })
}

fn parse_producer(value: &StrictValue) -> Result<Producer, ()> {
    let object = object(value)?;
    exact_keys(object, &["name", "version"])?;
    let name = safe_text(string(required(object, "name")?)?)?;
    let version = object
        .get("version")
        .map(|value| safe_text(string(value)?))
        .transpose()?;
    Ok(Producer { name, version })
}

fn measure_sample(sample: &Sample, ceiling: usize) -> Value {
    let base = json!({
        "alias": sample.alias,
        "format": sample.format.as_str(),
        "stratum": sample.stratum,
        "producer": producer_json(sample.producer.as_ref()),
    });
    let source = match read_regular_through_sentinel(&sample.path, ceiling) {
        Ok(Some(source)) => source,
        Ok(None) => return failed(base, "source_exceeds_collection_ceiling"),
        Err(code) => return failed(base, code),
    };
    let source = match String::from_utf8(source) {
        Ok(source) => source,
        Err(_) => return failed(base, "invalid_utf8"),
    };
    match sample.format {
        Format::Cdml => match measure_xml_input_v1(&source) {
            Ok(metrics) => match TypedDocument::parse(&source) {
                Ok(_) => measured(base, json!({"metrics": metrics_json(metrics)})),
                Err(_) => failed(base, "invalid_cdml_payload"),
            },
            Err(error) => failed(base, xml_code(&error)),
        },
        Format::Cdsvg => match measure_cdsvg_input_v1(&source) {
            Ok(metrics) => measured(
                base,
                json!({
                    "wrapper_metrics": metrics_json(metrics.wrapper),
                    "normalized_payload_metrics": metrics_json(metrics.normalized_payload),
                }),
            ),
            Err(error) => failed(base, cdsvg_code(&error)),
        },
    }
}

fn read_regular_through_sentinel(
    path: &Path,
    ceiling: usize,
) -> Result<Option<Vec<u8>>, &'static str> {
    let sentinel = ceiling
        .checked_add(1)
        .ok_or("read_ceiling_unrepresentable")?;
    let before = fs::symlink_metadata(path).map_err(|_| "read_error")?;
    if before.file_type().is_symlink() {
        return Err("symlink");
    }
    let mut file = File::open(path).map_err(|_| "read_error")?;
    if !file
        .metadata()
        .map_err(|_| "read_error")?
        .file_type()
        .is_file()
    {
        return Err("non_regular_file");
    }
    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 8192];
    while bytes.len() < sentinel {
        let count = file
            .read(&mut chunk[..(sentinel - bytes.len()).min(8192)])
            .map_err(|_| "read_error")?;
        if count == 0 {
            break;
        }
        bytes.extend_from_slice(&chunk[..count]);
    }
    Ok((bytes.len() < sentinel).then_some(bytes))
}

fn receipt(manifest: &Manifest, samples: Vec<Value>) -> Value {
    let mut formats = BTreeSet::new();
    for sample in &manifest.samples {
        formats.insert(sample.format.as_str());
    }
    let aggregates = aggregate_receipt(&samples);
    json!({
        "schema": RECEIPT_SCHEMA,
        "manifest_schema": SCHEMA,
        "collection_id": manifest.collection_id,
        "producer_metadata_policy": "supplier_declared_constrained",
        "measurement": {
            "tool": "ferrum-api CDML manifest measurement example",
            "tool_version": env!("CARGO_PKG_VERSION"),
            "target": format!("{}-{}", std::env::consts::ARCH, std::env::consts::OS),
            "xml_accounting": "XmlInputBudgetV1 tokenizer semantics v1",
            "formats": formats,
            "collection_read_ceiling_bytes": manifest.ceiling,
        },
        "samples": samples,
        "aggregates": aggregates,
    })
}

fn aggregate_receipt(samples: &[Value]) -> Value {
    let mut result = Map::new();
    for kind in ["cdml", "cdsvg_wrapper", "cdsvg_normalized_payload"] {
        let format = if kind == "cdml" { "cdml" } else { "cdsvg" };
        let metric_key = match kind {
            "cdml" => "metrics",
            "cdsvg_wrapper" => "wrapper_metrics",
            _ => "normalized_payload_metrics",
        };
        let selected: Vec<&Value> = samples
            .iter()
            .filter(|sample| sample["format"] == format)
            .collect();
        let measured: Vec<&Value> = selected
            .iter()
            .copied()
            .filter(|sample| sample["outcome"] == "measured")
            .collect();
        let maximum = maximum_metrics(&measured, metric_key);
        result.insert(
            kind.into(),
            json!({
                "measured_sample_count": measured.len(),
                "failed_sample_count": selected.len() - measured.len(),
                "maximum": maximum,
            }),
        );
    }
    let mut coverage: BTreeMap<String, usize> = BTreeMap::new();
    for sample in samples {
        let key = format!(
            "{}:{}",
            sample["format"].as_str().unwrap_or(""),
            sample["stratum"].as_str().unwrap_or("")
        );
        *coverage.entry(key).or_default() += 1;
    }
    result.insert("format_stratum_sample_counts".into(), json!(coverage));
    Value::Object(result)
}

fn maximum_metrics(samples: &[&Value], key: &str) -> Value {
    if samples.is_empty() {
        return Value::Null;
    }
    let metric_names = [
        "utf8_bytes",
        "elements",
        "max_depth",
        "attributes",
        "lexical_text_utf8_bytes",
    ];
    let mut maximum = Map::new();
    for metric in metric_names {
        let value = samples
            .iter()
            .filter_map(|sample| sample[key][metric].as_u64())
            .max()
            .expect("measured samples carry complete metrics");
        maximum.insert(metric.into(), json!(value));
    }
    Value::Object(maximum)
}

fn metrics_json(metrics: XmlInputMeasurementV1) -> Value {
    json!({
        "utf8_bytes": metrics.utf8_bytes,
        "elements": metrics.elements,
        "max_depth": metrics.max_depth,
        "attributes": metrics.attributes,
        "lexical_text_utf8_bytes": metrics.lexical_text_utf8_bytes,
    })
}
fn producer_json(producer: Option<&Producer>) -> Value {
    producer
        .map(|value| json!({"name": value.name, "version": value.version}))
        .unwrap_or(Value::Null)
}
fn measured(mut base: Value, extra: Value) -> Value {
    base.as_object_mut()
        .expect("base object")
        .insert("outcome".into(), json!("measured"));
    base.as_object_mut()
        .expect("base object")
        .extend(extra.as_object().expect("extra object").clone());
    base
}
fn failed(mut base: Value, code: &str) -> Value {
    let object = base.as_object_mut().expect("base object");
    object.insert("outcome".into(), json!("not_measured"));
    object.insert("failure_code".into(), json!(code));
    base
}
fn xml_code(error: &XmlInputError) -> &'static str {
    match error {
        XmlInputError::DtdForbidden => "dtd_forbidden",
        XmlInputError::Preflight(_) | XmlInputError::Xml(_) => "malformed_xml",
        XmlInputError::Budget(_) => "invalid_cdml_payload",
    }
}
fn cdsvg_code(error: &CdsvgExtractionError) -> &'static str {
    match error {
        CdsvgExtractionError::WrapperInput(error) | CdsvgExtractionError::PayloadInput(error) => {
            xml_code(error)
        }
        CdsvgExtractionError::NotSvgRoot { .. } => "not_svg_root",
        CdsvgExtractionError::MissingCdmlPayload => "missing_cdml_payload",
        CdsvgExtractionError::MultipleCdmlPayload { .. } => "multiple_cdml_payload",
        CdsvgExtractionError::PayloadSerialization(_) => "payload_serialization_error",
        CdsvgExtractionError::Typed(_) => "invalid_cdml_payload",
        CdsvgExtractionError::Xml(_) => "malformed_xml",
    }
}

fn reject_output_aliases(manifest: &Path, receipt: &Path, samples: &[Sample]) -> Result<(), ()> {
    if same_destination(manifest, receipt)?
        || samples
            .iter()
            .any(|sample| same_destination(&sample.path, receipt).unwrap_or(false))
    {
        return Err(());
    }
    let mut inputs = vec![identity(manifest)?];
    for sample in samples {
        // A missing, unreadable, symbolic-link, or nonregular sample remains a recorded per-sample
        // outcome. Only a file we can identify participates in the pre-read hard-link check.
        if let Some(identity) = optional_regular_identity(&sample.path) {
            inputs.push(identity);
        }
    }
    if receipt.exists() {
        let output = identity(receipt)?;
        if inputs.contains(&output) {
            return Err(());
        }
    }
    Ok(())
}

fn same_destination(left: &Path, right: &Path) -> Result<bool, ()> {
    if lexical_absolute(left)? == lexical_absolute(right)? {
        return Ok(true);
    }
    let (left_parent, left_name) = resolved_parent_identity(left)?;
    let (right_parent, right_name) = resolved_parent_identity(right)?;
    Ok(left_parent == right_parent && left_name == right_name)
}

fn resolved_parent_identity(path: &Path) -> Result<((u64, u64), std::ffi::OsString), ()> {
    let parent = path.parent().ok_or(())?;
    let parent = fs::canonicalize(parent).map_err(|_| ())?;
    let metadata = fs::metadata(parent).map_err(|_| ())?;
    if !metadata.is_dir() {
        return Err(());
    }
    Ok((
        (metadata.dev(), metadata.ino()),
        path.file_name().ok_or(())?.to_owned(),
    ))
}

fn lexical_absolute(path: &Path) -> Result<PathBuf, ()> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        env::current_dir().map_err(|_| ())?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::RootDir => normalized.push(Path::new("/")),
            Component::Normal(value) => normalized.push(value),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err(());
                }
            }
            Component::CurDir => {}
            Component::Prefix(_) => return Err(()),
        }
    }
    Ok(normalized)
}
fn optional_regular_identity(path: &Path) -> Option<(u64, u64)> {
    let metadata = fs::symlink_metadata(path).ok()?;
    (!metadata.file_type().is_symlink() && metadata.file_type().is_file())
        .then(|| (metadata.dev(), metadata.ino()))
}
fn identity(path: &Path) -> Result<(u64, u64), ()> {
    let metadata = fs::symlink_metadata(path).map_err(|_| ())?;
    if metadata.file_type().is_symlink() || !metadata.file_type().is_file() {
        return Err(());
    }
    Ok((metadata.dev(), metadata.ino()))
}

/// Publish through a retained descriptor.
///
/// A failed directory sync reports an uncertain completed replacement.
fn publish_receipt(path: &Path, bytes: &[u8]) -> Result<(), ()> {
    let name = path.file_name().ok_or(())?;
    let directory = open_parent(path)?;
    validate_destination(&directory, name)?;
    let (temporary, fd) = reserve_temporary(&directory, name)?;
    let mut file = File::from(fd);
    if file.write_all(bytes).and_then(|_| file.sync_all()).is_err() {
        let _ = unlinkat(&directory, &temporary, AtFlags::empty());
        return Err(());
    }
    drop(file);
    validate_destination(&directory, name)?;
    renameat(&directory, &temporary, &directory, name).map_err(|_| ())?;
    fsync(&directory).map_err(|_| ())
}
fn open_parent(path: &Path) -> Result<OwnedFd, ()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = openat(
        CWD,
        if parent.is_absolute() {
            Path::new("/")
        } else {
            Path::new(".")
        },
        flags,
        Mode::empty(),
    )
    .map_err(|_| ())?;
    for component in parent.components() {
        let part = match component {
            Component::Normal(part) => part,
            Component::ParentDir => std::ffi::OsStr::new(".."),
            Component::RootDir | Component::CurDir | Component::Prefix(_) => continue,
        };
        directory = openat(&directory, part, flags, Mode::empty()).map_err(|_| ())?;
    }
    Ok(directory)
}
fn validate_destination(directory: &OwnedFd, name: &std::ffi::OsStr) -> Result<(), ()> {
    match statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
        Ok(metadata) if FileType::from_raw_mode(metadata.st_mode).is_file() => Ok(()),
        Err(rustix::io::Errno::NOENT) => Ok(()),
        _ => Err(()),
    }
}
fn reserve_temporary(
    directory: &OwnedFd,
    name: &std::ffi::OsStr,
) -> Result<(std::ffi::OsString, OwnedFd), ()> {
    for _ in 0..TEMPORARY_ATTEMPTS {
        let sequence = NEXT_TEMPORARY_FILE.fetch_add(1, Ordering::Relaxed);
        let temporary = format!(
            ".{}.ferrum-measure-{sequence:016x}.tmp",
            name.to_string_lossy()
        )
        .into();
        match openat(
            directory,
            &temporary,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::CLOEXEC,
            Mode::RUSR | Mode::WUSR,
        ) {
            Ok(fd) => return Ok((temporary, fd)),
            Err(rustix::io::Errno::EXIST) => continue,
            Err(_) => return Err(()),
        }
    }
    Err(())
}

fn object(value: &StrictValue) -> Result<&BTreeMap<String, StrictValue>, ()> {
    if let StrictValue::Object(value) = value {
        Ok(value)
    } else {
        Err(())
    }
}
fn array(value: &StrictValue) -> Result<&[StrictValue], ()> {
    if let StrictValue::Array(value) = value {
        Ok(value)
    } else {
        Err(())
    }
}
fn string(value: &StrictValue) -> Result<&str, ()> {
    if let StrictValue::String(value) = value {
        Ok(value)
    } else {
        Err(())
    }
}
fn boolean(value: &StrictValue) -> Result<bool, ()> {
    if let StrictValue::Bool(value) = value {
        Ok(*value)
    } else {
        Err(())
    }
}
fn positive_usize(value: &StrictValue) -> Result<usize, ()> {
    if let StrictValue::Number(value) = value {
        value
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .filter(|value| *value > 0)
            .ok_or(())
    } else {
        Err(())
    }
}
fn required<'a>(
    object: &'a BTreeMap<String, StrictValue>,
    key: &str,
) -> Result<&'a StrictValue, ()> {
    object.get(key).ok_or(())
}
fn exact_keys(object: &BTreeMap<String, StrictValue>, allowed: &[&str]) -> Result<(), ()> {
    if object.keys().all(|key| allowed.contains(&key.as_str())) {
        Ok(())
    } else {
        Err(())
    }
}
fn safe_alias(value: &str) -> Result<String, ()> {
    if value.len() > 64
        || value.is_empty()
        || !value.as_bytes()[0].is_ascii_lowercase()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_' || byte == b'-'
        })
    {
        return Err(());
    }
    Ok(value.to_owned())
}
fn safe_identifier(value: &str) -> Result<String, ()> {
    safe_alias(value)
}
fn safe_text(value: &str) -> Result<String, ()> {
    if value.is_empty()
        || value.len() > 80
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_graphic() && byte != b'/' && byte != b'\\')
    {
        return Err(());
    }
    Ok(value.to_owned())
}

#[cfg(test)]
mod tests {
    use super::{parse_manifest, read_regular_through_sentinel, same_destination};
    use std::io::Write;
    #[test]
    fn strict_manifest_rejects_unknown_and_duplicate_consent() {
        let duplicate = concat!(
            r#"{"schema":"ferrum-cdml-measurement-manifest-v1","consented":true,"#,
            r#""consented":false,"collection_id":"local","#,
            r#""collection_read_ceiling_bytes":1,"samples":[{"alias":"a","#,
            r#""path":"/tmp/a","format":"cdml","stratum":"typical"}]}"#,
        );
        let unknown = concat!(
            r#"{"schema":"ferrum-cdml-measurement-manifest-v1","consented":true,"#,
            r#""collection_id":"local","collection_read_ceiling_bytes":1,"extra":1,"#,
            r#""samples":[{"alias":"a","path":"/tmp/a","format":"cdml","#,
            r#""stratum":"typical"}]}"#,
        );
        assert!(parse_manifest(duplicate.as_bytes()).is_err());
        assert!(parse_manifest(unknown.as_bytes()).is_err());
    }
    #[test]
    fn sentinel_is_a_lower_bound() {
        let path = std::env::temp_dir().join(format!("ferrum-measure-{}", std::process::id()));
        let mut file = std::fs::File::create(&path).expect("create");
        file.write_all(b"ab").expect("write");
        assert_eq!(read_regular_through_sentinel(&path, 1).expect("read"), None);
        std::fs::remove_file(path).expect("remove");
    }

    #[test]
    fn missing_sample_under_a_symlinked_parent_aliases_the_receipt_destination() {
        let root =
            std::env::temp_dir().join(format!("ferrum-measure-parent-{}", std::process::id()));
        let real = root.join("real");
        let linked = root.join("linked");
        std::fs::create_dir_all(&real).expect("create real parent");
        std::os::unix::fs::symlink(&real, &linked).expect("link parent");
        assert!(
            same_destination(&linked.join("future.cdml"), &real.join("future.cdml"))
                .expect("existing parent identities compare")
        );
        std::fs::remove_dir_all(root).expect("remove test tree");
    }
}
