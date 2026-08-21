//! Compiler-derived public-boundary proof for retired M4b SMARTS authority.
//!
//! Rustdoc JSON includes hidden items, so this test detects a forbidden item
//! even when it leaks through an alias, glob import, macro, or `#[doc(hidden)]`
//! re-export. Renderer crates have a closed no-SMARTS public boundary; the API
//! exposes only the closed stateless protocol DTO boundary.

use std::{
    collections::{BTreeSet, HashSet},
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

use serde_json::{Map, Value, json};

static TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

const API_SMARTS_DTO_NAMES: &[&str] = &[
    "DocumentSmartsQueryDocumentV1",
    "DocumentSmartsQueryInputV1",
    "DocumentSmartsQueryLimitsV1",
    "DocumentSmartsQueryRequestV1",
    "DocumentSmartsQuerySummaryV1",
    "DocumentSmartsQueryTraversalSummaryV1",
    "DocumentSmartsQueryMoleculeSummaryV1",
];

#[test]
fn retired_m4b_authority_is_not_publicly_reachable() {
    for crate_spec in [
        CrateSpec::new("ferrum-render", "ferrum_render"),
        CrateSpec::new("ferrum-document-render", "ferrum_document_render"),
        CrateSpec::new("ferrum-api", "ferrum_api"),
    ] {
        let inventory = rustdoc_inventory(crate_spec);
        match crate_spec.package {
            "ferrum-render" | "ferrum-document-render" => {
                assert_renderer_exposes_no_smarts_authority(crate_spec, &inventory);
            }
            "ferrum-api" => {
                assert_api_exports_only_protocol_smarts_dtos(&inventory);
                assert_no_prohibited_authority_categories(crate_spec, &inventory);
            }
            _ => unreachable!("the public-surface oracle owns exactly three crates"),
        }
    }
}

#[derive(Clone, Copy)]
struct CrateSpec {
    package: &'static str,
    crate_name: &'static str,
}

impl CrateSpec {
    const fn new(package: &'static str, crate_name: &'static str) -> Self {
        Self {
            package,
            crate_name,
        }
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct PublicItem {
    exported_path: String,
    target_name: String,
}

fn assert_renderer_exposes_no_smarts_authority(
    crate_spec: CrateSpec,
    inventory: &BTreeSet<PublicItem>,
) {
    for item in inventory {
        assert!(
            !item.is_smarts_specific(),
            "{} publicly reaches retired M4b SMARTS authority via path `{}` targeting `{}`",
            crate_spec.package,
            item.exported_path,
            item.target_name,
        );
    }
}

fn assert_api_exports_only_protocol_smarts_dtos(inventory: &BTreeSet<PublicItem>) {
    let allowed = API_SMARTS_DTO_NAMES
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    let leaked = inventory
        .iter()
        .filter(|item| item.is_smarts_specific())
        .filter(|item| {
            !allowed.contains(item.exported_terminal_name())
                || !allowed.contains(item.target_name.as_str())
        })
        .collect::<Vec<_>>();

    assert!(
        leaked.is_empty(),
        "ferrum-api may expose only the seven stateless protocol SMARTS DTOs; leaked private authority: {leaked:?}"
    );
}

fn assert_no_prohibited_authority_categories(
    crate_spec: CrateSpec,
    inventory: &BTreeSet<PublicItem>,
) {
    for item in inventory {
        let searchable = item.normalized_names();
        if let Some(category) = prohibited_authority_category(&searchable) {
            panic!(
                "{} publicly reaches prohibited M4b {category} via path `{}` targeting `{}`",
                crate_spec.package, item.exported_path, item.target_name,
            );
        }
    }
}

fn prohibited_authority_category(name: &str) -> Option<&'static str> {
    let smarts_prepared_surface = name.contains("smarts")
        && ["prepared", "target", "proof", "overlay"]
            .iter()
            .any(|term| name.contains(term));
    if smarts_prepared_surface {
        return Some("prepared target/proof/overlay authority");
    }
    if name.contains("targetdescriptor") {
        return Some("target descriptor authority");
    }
    let smarts_raw_accessor = name.contains("smarts")
        && ["rawmatch", "rawprojection", "rawanchor", "rawgeneration"]
            .iter()
            .any(|term| name.contains(term));
    if smarts_raw_accessor {
        return Some("raw match/projection/anchor/generation accessor");
    }
    let live_authority = [
        "livereceipt",
        "livebridge",
        "livesnapshot",
        "liverun",
        "liveledger",
        "liveredemption",
        "liveredeem",
    ]
    .iter()
    .any(|term| name.contains(term));
    live_authority.then_some("live receipt/bridge/snapshot/run/ledger/redemption authority")
}

impl PublicItem {
    fn exported_terminal_name(&self) -> &str {
        self.exported_path
            .rsplit("::")
            .next()
            .expect("public exported path has a terminal name")
    }

    fn is_smarts_specific(&self) -> bool {
        self.normalized_names().contains("smarts")
    }

    fn normalized_names(&self) -> String {
        format!("{}{}", self.exported_path, self.target_name)
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .flat_map(char::to_lowercase)
            .collect()
    }
}

/// Builds the pinned nightly Rustdoc JSON item tree with hidden items. The
/// compiler's module/import graph, not source-text search, establishes whether
/// a banned name is externally reachable.
fn rustdoc_inventory(crate_spec: CrateSpec) -> BTreeSet<PublicItem> {
    let root = TemporaryWorkRoot::new(crate_spec.package);
    let crate_root = workspace_root().join("crates").join(
        crate_spec
            .package
            .strip_prefix("ferrum-")
            .expect("Ferrum package has a ferrum- prefix"),
    );
    let output = Command::new("rustup")
        .args([
            "run",
            "nightly",
            "cargo",
            "rustdoc",
            "--quiet",
            "--manifest-path",
        ])
        .arg(crate_root.join("Cargo.toml"))
        .args(["--lib", "--target-dir"])
        .arg(root.path())
        .args([
            "--",
            "-Z",
            "unstable-options",
            "--output-format",
            "json",
            "--document-hidden-items",
        ])
        .output()
        .expect("run nightly Rustdoc JSON for the M4b public-surface oracle");
    assert_success(&output, crate_spec.package);

    let index_path = root
        .path()
        .join("doc")
        .join(format!("{}.json", crate_spec.crate_name));
    let document = fs::read_to_string(&index_path).unwrap_or_else(|error| {
        panic!(
            "read Rustdoc JSON inventory {}: {error}",
            index_path.display()
        )
    });
    let inventory = parse_public_inventory(&document);
    inventory
}

fn parse_public_inventory(document: &str) -> BTreeSet<PublicItem> {
    let document: Value = serde_json::from_str(document).expect("Rustdoc JSON inventory is JSON");
    let index = document["index"]
        .as_object()
        .expect("Rustdoc JSON has an item index");
    let root_id = numeric_id(&document["root"], "Rustdoc JSON has a root item ID");
    let mut inventory = BTreeSet::new();
    let mut traversed = HashSet::new();
    let mut referenced_type_targets = HashSet::new();
    collect_public_module_items(
        index,
        &root_id,
        "",
        &mut traversed,
        &mut referenced_type_targets,
        &mut inventory,
    );
    inventory
}

#[test]
fn public_associated_items_and_type_references_keep_their_owner_in_exported_paths() {
    let document = json!({
        "root": 0,
        "index": {
            "0": {"visibility": "public", "name": "fixture", "inner": {"module": {"items": [1, 5, 8, 12, 16]}}},
            "1": {"visibility": "public", "name": "NeutralApi", "inner": {"struct": {"impls": [2], "fields": [3]}}},
            "2": {"visibility": "default", "name": null, "inner": {"impl": {"items": [4, 20]}}},
            "3": {"visibility": "public", "name": "value", "inner": {"struct_field": {"type": {"resolved_path": {"id": 13}}}}},
            "4": {"visibility": "public", "name": "redeem_live_smarts_receipt_v1", "inner": {"function": {}}},
            "5": {"visibility": "public", "name": "NeutralBridge", "inner": {"trait": {"impls": [], "items": [6, 7], "fields": []}}},
            "6": {"visibility": "default", "name": "prepare_smarts_target", "inner": {"function": {}}},
            "7": {"visibility": "default", "name": "SmartsAnchor", "inner": {"assoc_type": {}}},
            "8": {"visibility": "public", "name": "NeutralAlias", "inner": {"type_alias": {"type": {"resolved_path": {"id": 9}}}}},
            "9": {"visibility": "default", "name": "InternalCarrier", "inner": {"struct": {"impls": [10], "fields": []}}},
            "10": {"visibility": "default", "name": null, "inner": {"impl": {"items": [11]}}},
            "11": {"visibility": "public", "name": "redeem_live_smarts_receipt_v1", "inner": {"function": {}}},
            "12": {"visibility": "public", "name": "PublicFunction", "inner": {"function": {"decl": {"inputs": [["value", {"resolved_path": {"id": 14}}]], "output": {"resolved_path": {"id": 15}}}}}},
            "13": {"visibility": "default", "name": "HiddenSmartsField", "inner": {"struct": {"impls": [], "fields": []}}},
            "14": {"visibility": "default", "name": "HiddenSmartsParameter", "inner": {"struct": {"impls": [], "fields": []}}},
            "15": {"visibility": "default", "name": "HiddenSmartsReturn", "inner": {"struct": {"impls": [], "fields": []}}},
            "16": {"visibility": "public", "name": "PublicChoice", "inner": {"enum": {"impls": [], "variants": [17], "fields": []}}},
            "17": {"visibility": "public", "name": "Payload", "inner": {"variant": {"kind": {"plain": {"fields": [18]}}}}},
            "18": {"visibility": "public", "name": "entry", "inner": {"struct_field": {"type": {"resolved_path": {"id": 19}}}}},
            "19": {"visibility": "default", "name": "HiddenSmartsPayload", "inner": {"struct": {"impls": [], "fields": []}}},
            "20": {"visibility": "public", "name": "TOKEN", "inner": {"assoc_const": {"type": {"resolved_path": {"id": 21}}}}},
            "21": {"visibility": "default", "name": "HiddenLiveSmartsReceipt", "inner": {"struct": {"impls": [], "fields": []}}}
        }
    });

    let inventory = parse_public_inventory(&document.to_string());
    let paths = inventory
        .iter()
        .map(|item| item.exported_path.as_str())
        .collect::<BTreeSet<_>>();

    assert!(paths.contains("NeutralApi::redeem_live_smarts_receipt_v1"));
    assert!(paths.contains("NeutralApi::TOKEN"));
    assert!(paths.contains("NeutralApi::value"));
    assert!(paths.contains("NeutralBridge::prepare_smarts_target"));
    assert!(paths.contains("NeutralBridge::SmartsAnchor"));
    assert!(paths.contains("NeutralAlias::redeem_live_smarts_receipt_v1"));
    assert!(paths.contains("PublicFunction"));
    assert!(paths.contains("PublicChoice::Payload::entry"));
    assert!(inventory.iter().any(|item| {
        item.exported_path == "NeutralApi::TOKEN" && item.target_name == "HiddenLiveSmartsReceipt"
    }));
    assert!(inventory.iter().any(|item| {
        item.exported_path == "NeutralApi::value" && item.target_name == "HiddenSmartsField"
    }));
    assert!(inventory.iter().any(|item| {
        item.exported_path == "PublicFunction" && item.target_name == "HiddenSmartsParameter"
    }));
    assert!(inventory.iter().any(|item| {
        item.exported_path == "PublicFunction" && item.target_name == "HiddenSmartsReturn"
    }));
    assert!(inventory.iter().any(|item| {
        item.exported_path == "PublicChoice::Payload::entry"
            && item.target_name == "HiddenSmartsPayload"
    }));
    assert!(
        std::panic::catch_unwind(|| {
            assert_no_prohibited_authority_categories(
                CrateSpec::new("fixture", "fixture"),
                &inventory,
            );
        })
        .is_err()
    );
}

fn collect_public_module_items(
    index: &Map<String, Value>,
    module_id: &str,
    prefix: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    if !traversed.insert((module_id.to_owned(), prefix.to_owned())) {
        return;
    }
    for child_id in module_items(index, module_id) {
        let child = index
            .get(child_id)
            .expect("Rustdoc module child resolves in item index");
        if child["visibility"] != "public" {
            continue;
        }
        let import = child["inner"].get("use");
        if let Some(import) = import {
            collect_public_import(
                index,
                import,
                prefix,
                traversed,
                referenced_type_targets,
                inventory,
            );
            continue;
        }
        let name = item_name(child).expect("public Rustdoc item has a name");
        let exported_path = join_path(prefix, name);
        collect_public_item(
            index,
            child_id,
            &exported_path,
            traversed,
            referenced_type_targets,
            inventory,
        );
    }
}

/// Records one public export and its public associated API. Rustdoc stores
/// methods, trait members, variants, and fields below their owning item rather
/// than in a module's item list, so their externally callable path must retain
/// that owner's exported path.
fn collect_public_item(
    index: &Map<String, Value>,
    item_id: &str,
    exported_path: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    let item = index
        .get(item_id)
        .expect("Rustdoc public item resolves in item index");
    let name = item_name(item).expect("public Rustdoc item has a name");
    inventory.insert(PublicItem {
        exported_path: exported_path.to_owned(),
        target_name: name.to_owned(),
    });
    collect_exposed_type_references(
        index,
        item,
        exported_path,
        traversed,
        referenced_type_targets,
        inventory,
    );
    match item_kind(item) {
        "module" => collect_public_module_items(
            index,
            item_id,
            exported_path,
            traversed,
            referenced_type_targets,
            inventory,
        ),
        "struct" | "union" | "enum" | "trait" => {
            collect_public_associated_items(
                index,
                item,
                exported_path,
                traversed,
                referenced_type_targets,
                inventory,
            );
        }
        "variant" => collect_variant_fields(
            index,
            item,
            exported_path,
            traversed,
            referenced_type_targets,
            inventory,
        ),
        _ => {}
    }
}

fn collect_public_associated_items(
    index: &Map<String, Value>,
    owner: &Value,
    owner_path: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    let inner = owner["inner"]
        .as_object()
        .expect("Rustdoc public item has an inner object");
    let kind = item_kind(owner);
    let definition = inner
        .get(kind)
        .expect("Rustdoc public item has its declared kind");
    collect_associated_ids(
        index,
        definition["impls"].as_array(),
        owner_path,
        false,
        traversed,
        referenced_type_targets,
        inventory,
    );
    collect_associated_ids(
        index,
        definition["items"].as_array(),
        owner_path,
        kind == "trait",
        traversed,
        referenced_type_targets,
        inventory,
    );
    collect_record_fields(
        index,
        definition,
        owner_path,
        traversed,
        referenced_type_targets,
        inventory,
    );
    if kind == "enum" {
        collect_associated_ids(
            index,
            definition["variants"].as_array(),
            owner_path,
            true,
            traversed,
            referenced_type_targets,
            inventory,
        );
    }
}

fn collect_record_fields(
    index: &Map<String, Value>,
    definition: &Value,
    owner_path: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    for fields in [
        definition["fields"].as_array(),
        definition["kind"]["plain"]["fields"].as_array(),
        definition["kind"]["struct"]["fields"].as_array(),
        definition["struct"]["fields"].as_array(),
    ] {
        collect_associated_ids(
            index,
            fields,
            owner_path,
            false,
            traversed,
            referenced_type_targets,
            inventory,
        );
    }
}

fn collect_variant_fields(
    index: &Map<String, Value>,
    variant: &Value,
    variant_path: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    let kind = &variant["inner"]["variant"]["kind"];
    collect_associated_ids(
        index,
        kind["tuple"].as_array(),
        variant_path,
        false,
        traversed,
        referenced_type_targets,
        inventory,
    );
    collect_record_fields(
        index,
        kind,
        variant_path,
        traversed,
        referenced_type_targets,
        inventory,
    );
}

fn collect_associated_ids(
    index: &Map<String, Value>,
    item_ids: Option<&Vec<Value>>,
    owner_path: &str,
    trait_member: bool,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    for item_id in item_ids.into_iter().flatten() {
        let item_id = numeric_id(item_id, "Rustdoc associated item ID is numeric");
        let item = index
            .get(&item_id)
            .expect("Rustdoc associated item resolves in item index");
        if item_kind(item) == "impl" {
            collect_associated_ids(
                index,
                item["inner"]["impl"]["items"].as_array(),
                owner_path,
                false,
                traversed,
                referenced_type_targets,
                inventory,
            );
            continue;
        }
        if !trait_member && item["visibility"] != "public" {
            continue;
        }
        let name = item_name(item).expect("public associated Rustdoc item has a name");
        let exported_path = join_path(owner_path, name);
        collect_public_item(
            index,
            &item_id,
            &exported_path,
            traversed,
            referenced_type_targets,
            inventory,
        );
    }
}

fn collect_public_import(
    index: &Map<String, Value>,
    import: &Value,
    prefix: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    let target_id = numeric_id(&import["id"], "public Rustdoc import has a target item ID");
    let target = index
        .get(&target_id)
        .expect("public Rustdoc import target resolves in item index");
    if import["glob"].as_bool().unwrap_or(false) {
        if item_kind(target) == "module" {
            collect_public_module_items(
                index,
                &target_id,
                prefix,
                traversed,
                referenced_type_targets,
                inventory,
            );
        }
        return;
    }
    let export_name = import["name"]
        .as_str()
        .expect("non-glob public Rustdoc import has an export name");
    let exported_path = join_path(prefix, export_name);
    collect_public_item(
        index,
        &target_id,
        &exported_path,
        traversed,
        referenced_type_targets,
        inventory,
    );
}

/// Descends only local Rustdoc IDs found in public API type positions. This
/// intentionally ignores primitive, external, and standard-library details;
/// the oracle needs only named local types that can carry retired authority.
fn collect_exposed_type_references(
    index: &Map<String, Value>,
    item: &Value,
    exported_path: &str,
    traversed: &mut HashSet<(String, String)>,
    referenced_type_targets: &mut HashSet<String>,
    inventory: &mut BTreeSet<PublicItem>,
) {
    let type_position = match item_kind(item) {
        "type_alias" => item["inner"]["type_alias"].get("type"),
        "struct_field" => item["inner"]["struct_field"].get("type"),
        "assoc_type" => item["inner"]["assoc_type"].get("type"),
        "assoc_const" => item["inner"]["assoc_const"].get("type"),
        "constant" => item["inner"]["constant"].get("type"),
        "static" => item["inner"]["static"].get("type"),
        "function" => item["inner"]["function"].get("decl"),
        _ => None,
    };
    let Some(type_position) = type_position else {
        return;
    };
    let mut targets = BTreeSet::new();
    collect_resolved_path_ids(type_position, &mut targets);
    for target_id in targets {
        if !referenced_type_targets.insert(target_id.clone()) || !index.contains_key(&target_id) {
            continue;
        }
        collect_public_item(
            index,
            &target_id,
            exported_path,
            traversed,
            referenced_type_targets,
            inventory,
        );
    }
}

fn collect_resolved_path_ids(value: &Value, targets: &mut BTreeSet<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_resolved_path_ids(value, targets);
            }
        }
        Value::Object(fields) => {
            if let Some(id) = fields.get("resolved_path").and_then(|path| path.get("id")) {
                targets.insert(numeric_id(
                    id,
                    "Rustdoc resolved path has a numeric item ID",
                ));
            }
            for value in fields.values() {
                collect_resolved_path_ids(value, targets);
            }
        }
        _ => {}
    }
}

fn module_items<'a>(index: &'a Map<String, Value>, module_id: &str) -> Vec<&'a str> {
    index[module_id]["inner"]["module"]["items"]
        .as_array()
        .expect("Rustdoc public module has item IDs")
        .iter()
        .map(|item_id| {
            item_id
                .as_u64()
                .expect("Rustdoc module item ID is numeric")
                .to_string()
        })
        .map(|item_id| {
            index
                .get_key_value(&item_id)
                .expect("Rustdoc module item resolves")
                .0
                .as_str()
        })
        .collect()
}

fn numeric_id(value: &Value, message: &str) -> String {
    value.as_u64().expect(message).to_string()
}

fn item_name(item: &Value) -> Option<&str> {
    item["name"].as_str()
}

fn item_kind(item: &Value) -> &str {
    item["inner"]
        .as_object()
        .expect("Rustdoc item has an inner kind")
        .keys()
        .next()
        .expect("Rustdoc item has one inner kind")
}

fn join_path(prefix: &str, name: &str) -> String {
    if prefix.is_empty() {
        name.to_owned()
    } else {
        format!("{prefix}::{name}")
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("API manifest is nested under the workspace")
        .to_path_buf()
}

struct TemporaryWorkRoot(PathBuf);

impl TemporaryWorkRoot {
    fn new(package: &str) -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        Self(std::env::temp_dir().join(format!(
            "ferrum-m4b-public-surface-{}-{package}-{sequence}",
            std::process::id()
        )))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryWorkRoot {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "remove Rustdoc JSON work directory {}: {error}",
                    self.0.display()
                );
            }
        }
    }
}

fn assert_success(output: &Output, package: &str) {
    assert!(
        output.status.success(),
        "build Rustdoc JSON inventory for {package}:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
