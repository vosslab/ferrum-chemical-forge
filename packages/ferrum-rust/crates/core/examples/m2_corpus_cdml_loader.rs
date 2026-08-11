//! M2 corpus CDML loader, hosted as a dev target of `ferrum-core`.
//!
//! Reads one corpus CDML file with `xot` and projects every `<molecule>` subtree into
//! `ferrum-core` records, then prints the projection as JSON on stdout in the field
//! shape used by `oracle_molecule_core.rs` and the sibling OASA worker
//! `tests/e2e/oracle/e2e_oasa_corpus_molecule_child.py`, so a comparison harness can
//! consume both.
//!
//! Loading is total, not best effort. Inside a `<molecule>` subtree every element and
//! attribute is either carried into the core model or deferred by a named row of the
//! source-field mapping table in `docs/active_plans/decisions/ferrum_core_model.md`
//! (lines 72-84). Anything else is collected and reported, and the run fails. A loader
//! that silently ignored unrecognized content could pass without loading anything, which
//! is the exact failure the corpus exists to catch.
//!
//! Coordinates are emitted in PostScript points. OASA's `cm_to_float_coord`
//! (`OTHER_REPOS/bkchem-oasa/packages/oasa/oasa/cdml.py` lines 62-68) scales a `cm`
//! suffix by `72/2.54` and passes a bare number through unchanged, so a bare CDML
//! number is already in points.
//!
//! This target is disposable. M8 delivers the typed CDML reader that retires it, and
//! `tests/test_cdml_reader_inventory.py` holds the allowlist that keeps a third reader
//! from appearing unnoticed.

use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

use ferrum_core::{
    Atom, Bond, BondOrder, BondStyle, Identifier, ModelError, Molecule, NonAtomVertex, Position,
    RecordId, RecordKind, VertexRef,
};
use serde_json::{Value, json};
use xot::{Node, Xot};

const CDML_NAMESPACE: &str = "http://www.freesoftware.fsf.org/bkchem/cdml";

/// Points per centimetre, matching OASA's `cm_to_float_coord` factor exactly.
const POINTS_PER_CENTIMETRE: f64 = 72.0 / 2.54;

/// Atom attributes deferred by the "Bond depiction attributes and atom presentation |
/// Dropped" row of the source-field mapping table cited in the module documentation.
/// M6-M8 document storage and M12 rendering own them.
const DROPPED_ATOM_PRESENTATION_ATTRIBUTES: &[&str] = &[
    "show",
    "show_number",
    "hydrogens",
    "number",
    "pos",
    "background-color",
];

/// Atom children deferred by that same "Dropped" row: authored font, formatted text,
/// and mark records are presentation payload rather than carried chemistry facts.
const DROPPED_ATOM_PRESENTATION_CHILDREN: &[&str] = &["font", "ftext", "mark"];

/// Molecule children deferred by the source-field mapping table. Template and
/// fragment records carry IDREF metadata rather than graph members. Display-form and
/// user-data records are preservation-only payload containers owned by M6-M8.
const DROPPED_MOLECULE_CHILDREN: &[&str] = &["template", "fragment", "display-form", "user-data"];

/// Bond attributes deferred by that same "Dropped" row. Every one of these describes
/// how a bond is painted rather than what it connects.
const DROPPED_BOND_DEPICTION_ATTRIBUTES: &[&str] = &[
    "line_width",
    "bond_width",
    "wedge_width",
    "double_ratio",
    "center",
    "distance",
    "width",
    "auto_sign",
    "equithick",
    "simple_double",
    "color",
    "wavy_style",
    "haworth_position",
];

/// One `<bond>` element read before its endpoints have been resolved.
struct RawBond {
    source_id: Option<Identifier>,
    start: String,
    end: String,
    source_type: Option<String>,
    context: String,
}

/// Every molecule-local vertex the core model carries, in source order per kind.
struct MoleculeVertices {
    atoms: Vec<Atom>,
    groups: Vec<NonAtomVertex>,
    texts: Vec<NonAtomVertex>,
    queries: Vec<NonAtomVertex>,
    references: HashMap<String, VertexRef>,
}

fn main() {
    let mut arguments = env::args().skip(1);
    let Some(corpus_path) = arguments.next() else {
        eprintln!("usage: m2_corpus_cdml_loader <corpus.cdml>");
        process::exit(2);
    };
    let source = match fs::read_to_string(&corpus_path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("cannot read {corpus_path}: {error}");
            process::exit(2);
        }
    };
    match load_corpus(&source, &corpus_path) {
        Ok(result) => println!(
            "{}",
            serde_json::to_string(&result).expect("serialize corpus projection")
        ),
        Err(error) => {
            eprintln!("{error}");
            process::exit(1);
        }
    }
}

/// Parse the corpus document and project every molecule it declares.
fn load_corpus(source: &str, corpus_path: &str) -> Result<Value, String> {
    let mut tree = Xot::new();
    let document = tree.parse(source).map_err(|error| error.to_string())?;
    let root = tree
        .document_element(document)
        .map_err(|error| error.to_string())?;
    let (root_name, root_namespace) =
        element_name(&tree, root).ok_or_else(|| "document element is not an element".to_owned())?;
    if root_name != "cdml" || root_namespace != CDML_NAMESPACE {
        return Err(format!(
            "document element is {{{root_namespace}}}{root_name}, not a CDML root"
        ));
    }
    // Amendment B is scoped to molecule subtrees, so other direct children of <cdml>
    // stay the document layer's business and are not inventoried here.
    let mut unhandled = Vec::new();
    let mut used_identities = HashSet::new();
    let mut projected_molecules = Vec::new();
    let mut molecule_index = 0_usize;
    for child in tree.children(root) {
        let Some((local_name, namespace)) = element_name(&tree, child) else {
            continue;
        };
        if namespace != CDML_NAMESPACE || local_name != "molecule" {
            continue;
        }
        let molecule = load_molecule(
            &tree,
            child,
            molecule_index,
            &mut used_identities,
            &mut unhandled,
        )?;
        projected_molecules.push(project_molecule(&molecule, molecule_index)?);
        molecule_index += 1;
    }
    if !unhandled.is_empty() {
        unhandled.sort();
        unhandled.dedup();
        let listing = unhandled.join("\n  ");
        return Err(format!(
            "{corpus_path}: molecule content is neither carried by the core model nor \
deferred by the source-field mapping table in \
docs/active_plans/decisions/ferrum_core_model.md lines 72-84:\n  {listing}"
        ));
    }
    let projection = json!({
        "capability": "corpus-molecule-core",
        "corpus_path": corpus_path,
        "coordinate_unit": "postscript_point",
        "molecules": projected_molecules,
    });
    let result = json!({
        "engine": "ferrum-core",
        "facts": {
            "crate_version": env!("CARGO_PKG_VERSION"),
            "rust_version_floor": env!("CARGO_PKG_RUST_VERSION"),
        },
        "projection": projection,
    });
    Ok(result)
}

/// Build one validated `Molecule` from a `<molecule>` element.
fn load_molecule(
    tree: &Xot,
    node: Node,
    molecule_index: usize,
    used_identities: &mut HashSet<RecordId>,
    unhandled: &mut Vec<String>,
) -> Result<Molecule, String> {
    let context = format!("molecule[{molecule_index}]");
    let mut source_id = None;
    let mut name = None;
    for (namespace, local_name, value) in attributes(tree, node) {
        match (namespace.as_str(), local_name.as_str()) {
            ("", "id") => source_id = Some(identifier(&value)?),
            ("", "name") => name = Some(value),
            // Foreign attributes are vendor payload retained by M8's unknown-
            // attribute bag; they cannot add a core graph member.
            (namespace, _) if !namespace.is_empty() => {}
            _ => unhandled.push(attribute_context(&context, &namespace, &local_name)),
        }
    }
    let vertices = load_vertices(tree, node, &context, used_identities, unhandled)?;
    let raw_bonds = read_raw_bonds(tree, node, &context, unhandled)?;
    let mut bonds = Vec::new();
    for raw_bond in &raw_bonds {
        let bond = build_bond(raw_bond, &vertices.references, used_identities)?;
        used_identities.insert(bond.identity().clone());
        bonds.push(bond);
    }
    let molecule = build_record(used_identities, Molecule::identity, |occurrence| {
        Molecule::new(
            source_id.clone(),
            name.clone(),
            vertices.atoms.clone(),
            vertices.groups.clone(),
            vertices.texts.clone(),
            vertices.queries.clone(),
            bonds.clone(),
            occurrence,
        )
    })
    .map_err(|error| format!("{context}: {error}"))?;
    used_identities.insert(molecule.identity().clone());
    Ok(molecule)
}

/// Read every typed vertex a molecule declares, in source order per kind.
fn load_vertices(
    tree: &Xot,
    node: Node,
    context: &str,
    used_identities: &mut HashSet<RecordId>,
    unhandled: &mut Vec<String>,
) -> Result<MoleculeVertices, String> {
    let mut vertices = MoleculeVertices {
        atoms: Vec::new(),
        groups: Vec::new(),
        texts: Vec::new(),
        queries: Vec::new(),
        references: HashMap::new(),
    };
    let mut atom_index = 0_usize;
    for child in children_of(tree, node, context, unhandled) {
        let (local_name, namespace, child_node) = child;
        if namespace != CDML_NAMESPACE {
            unhandled.push(element_context(context, &namespace, &local_name));
            continue;
        }
        match local_name.as_str() {
            "atom" => {
                let child_context = format!("{context}/atom[{atom_index}]");
                atom_index += 1;
                let atom = load_atom(tree, child_node, &child_context, used_identities, unhandled)?;
                used_identities.insert(atom.identity().clone());
                register_reference(
                    &mut vertices.references,
                    atom.source_id(),
                    VertexRef::Atom(atom.identity().clone()),
                    &child_context,
                )?;
                vertices.atoms.push(atom);
            }
            "group" | "text" | "query" => {
                let kind = vertex_kind(&local_name);
                let index = match kind {
                    RecordKind::Group => vertices.groups.len(),
                    RecordKind::Text => vertices.texts.len(),
                    _ => vertices.queries.len(),
                };
                let child_context = format!("{context}/{local_name}[{index}]");
                let vertex =
                    load_non_atom_vertex(tree, child_node, kind, &child_context, used_identities)?;
                used_identities.insert(vertex.identity().clone());
                register_reference(
                    &mut vertices.references,
                    vertex.source_id(),
                    vertex_reference(kind, vertex.identity().clone()),
                    &child_context,
                )?;
                match kind {
                    RecordKind::Group => vertices.groups.push(vertex),
                    RecordKind::Text => vertices.texts.push(vertex),
                    _ => vertices.queries.push(vertex),
                }
            }
            // Bonds are read separately, once every vertex identity exists to resolve
            // their endpoints against.
            "bond" => {}
            other if DROPPED_MOLECULE_CHILDREN.contains(&other) => {}
            _ => unhandled.push(element_context(context, &namespace, &local_name)),
        }
    }
    Ok(vertices)
}

/// Build one validated `Atom` from an `<atom>` element.
fn load_atom(
    tree: &Xot,
    node: Node,
    context: &str,
    used_identities: &HashSet<RecordId>,
    unhandled: &mut Vec<String>,
) -> Result<Atom, String> {
    let mut source_id = None;
    let mut element = None;
    let mut formal_charge = None;
    let mut isotope = None;
    let mut explicit_hydrogens = None;
    let mut valence = None;
    let mut multiplicity = None;
    let mut free_sites = None;
    for (namespace, local_name, value) in attributes(tree, node) {
        if !namespace.is_empty() {
            // Foreign attributes are vendor payload retained by M8.
            continue;
        }
        match local_name.as_str() {
            "id" => source_id = Some(identifier(&value)?),
            "name" => element = Some(value),
            "charge" => formal_charge = Some(parse_scalar(&value, context, "charge")?),
            "isotope" => isotope = Some(parse_scalar(&value, context, "isotope")?),
            "explicit_hydrogens" => {
                explicit_hydrogens = Some(parse_scalar(&value, context, "explicit_hydrogens")?);
            }
            "valency" => valence = Some(parse_scalar(&value, context, "valency")?),
            "multiplicity" => multiplicity = Some(parse_scalar(&value, context, "multiplicity")?),
            "free_sites" => free_sites = Some(parse_scalar(&value, context, "free_sites")?),
            other if DROPPED_ATOM_PRESENTATION_ATTRIBUTES.contains(&other) => {}
            // The accepted M8 assignment uses this corpus attribute to prove that
            // an unfamiliar attribute leaves the atom typed.
            "local_extension" => {}
            _ => unhandled.push(attribute_context(context, &namespace, &local_name)),
        }
    }
    let mut position = None;
    for (local_name, namespace, child_node) in children_of(tree, node, context, unhandled) {
        if namespace != CDML_NAMESPACE {
            unhandled.push(element_context(context, &namespace, &local_name));
            continue;
        }
        match local_name.as_str() {
            "point" => position = Some(load_position(tree, child_node, context, unhandled)?),
            other if DROPPED_ATOM_PRESENTATION_CHILDREN.contains(&other) => {}
            _ => unhandled.push(element_context(context, &namespace, &local_name)),
        }
    }
    let position =
        position.ok_or_else(|| format!("{context}: the core atom shape requires one point"))?;
    let atom = build_record(used_identities, Atom::identity, |occurrence| {
        Atom::new(
            source_id.clone(),
            element.clone(),
            position,
            formal_charge,
            isotope,
            explicit_hydrogens,
            valence,
            multiplicity,
            free_sites,
            occurrence,
        )
    })
    .map_err(|error| format!("{context}: {error}"))?;
    Ok(atom)
}

/// Build one group, molecule-local text, or query vertex.
///
/// The "Group, text, query identity | Carried minimally" row of the source-field mapping
/// table gives the core only the identity and assigns the payload to the document layer,
/// so this vertex's remaining attributes and children are deferred by that row.
fn load_non_atom_vertex(
    tree: &Xot,
    node: Node,
    kind: RecordKind,
    context: &str,
    used_identities: &HashSet<RecordId>,
) -> Result<NonAtomVertex, String> {
    let mut source_id = None;
    for (namespace, local_name, value) in attributes(tree, node) {
        if namespace.is_empty() && local_name == "id" {
            source_id = Some(identifier(&value)?);
        }
    }
    let vertex = build_record(used_identities, NonAtomVertex::identity, |occurrence| {
        NonAtomVertex::new(kind, source_id.clone(), occurrence)
    })
    .map_err(|error| format!("{context}: {error}"))?;
    Ok(vertex)
}

/// Read every `<bond>` element without resolving its endpoints yet.
fn read_raw_bonds(
    tree: &Xot,
    node: Node,
    context: &str,
    unhandled: &mut Vec<String>,
) -> Result<Vec<RawBond>, String> {
    let mut raw_bonds = Vec::new();
    let mut bond_index = 0_usize;
    for (local_name, namespace, child_node) in children_of(tree, node, context, unhandled) {
        if namespace != CDML_NAMESPACE || local_name != "bond" {
            continue;
        }
        let bond_context = format!("{context}/bond[{bond_index}]");
        bond_index += 1;
        let mut source_id = None;
        let mut start = None;
        let mut end = None;
        let mut source_type = None;
        for (attribute_namespace, attribute_name, value) in attributes(tree, child_node) {
            if !attribute_namespace.is_empty() {
                // Foreign attributes are vendor payload retained by M8.
                continue;
            }
            match attribute_name.as_str() {
                "id" => source_id = Some(identifier(&value)?),
                "start" => start = Some(value),
                "end" => end = Some(value),
                "type" => source_type = Some(value),
                other if DROPPED_BOND_DEPICTION_ATTRIBUTES.contains(&other) => {}
                _ => unhandled.push(attribute_context(
                    &bond_context,
                    &attribute_namespace,
                    &attribute_name,
                )),
            }
        }
        // A bond child element would be unrecognized content; the CDML bond record is
        // attribute-only in every corpus form.
        for (child_name, child_namespace, _) in
            children_of(tree, child_node, &bond_context, unhandled)
        {
            unhandled.push(element_context(
                &bond_context,
                &child_namespace,
                &child_name,
            ));
        }
        let start = start.ok_or_else(|| format!("{bond_context}: bond requires a start"))?;
        let end = end.ok_or_else(|| format!("{bond_context}: bond requires an end"))?;
        raw_bonds.push(RawBond {
            source_id,
            start,
            end,
            source_type,
            context: bond_context,
        });
    }
    Ok(raw_bonds)
}

/// Resolve one raw bond's typed endpoints and build the validated record.
fn build_bond(
    raw_bond: &RawBond,
    references: &HashMap<String, VertexRef>,
    used_identities: &HashSet<RecordId>,
) -> Result<Bond, String> {
    let context = &raw_bond.context;
    let start = references
        .get(&raw_bond.start)
        .ok_or_else(|| format!("{context}: start {} names no local vertex", raw_bond.start))?;
    let end = references
        .get(&raw_bond.end)
        .ok_or_else(|| format!("{context}: end {} names no local vertex", raw_bond.end))?;
    let order = raw_bond.source_type.as_deref().and_then(bond_order);
    let style = raw_bond.source_type.as_deref().and_then(bond_style);
    let bond = build_record(used_identities, Bond::identity, |occurrence| {
        Bond::new(
            raw_bond.source_id.clone(),
            start.clone(),
            end.clone(),
            raw_bond.source_type.clone(),
            order,
            style.clone(),
            // No corpus CDML form states an aromatic flag, and the model forbids
            // normalizing an absent source fact into an authored default.
            None,
            occurrence,
        )
    })
    .map_err(|error| format!("{context}: {error}"))?;
    Ok(bond)
}

/// Build a record, assigning the lowest occurrence that yields a fresh identity.
///
/// An idless record adds an occurrence only among records with an identical canonical
/// fingerprint, exactly as `ferrum_core_model.md` line 45 requires of the reader. A
/// source-backed record takes no occurrence, so the first attempt is the only one.
fn build_record<T, B, I>(
    used_identities: &HashSet<RecordId>,
    identity: I,
    build: B,
) -> Result<T, ModelError>
where
    B: Fn(Option<u32>) -> Result<T, ModelError>,
    I: Fn(&T) -> &RecordId,
{
    let source_backed = build(None);
    if let Ok(record) = source_backed {
        return Ok(record);
    }
    let mut occurrence = 0_u32;
    loop {
        let record = build(Some(occurrence))?;
        if !used_identities.contains(identity(&record)) {
            return Ok(record);
        }
        occurrence += 1;
    }
}

/// Read one `<point>` into a validated position, in PostScript points.
fn load_position(
    tree: &Xot,
    node: Node,
    context: &str,
    unhandled: &mut Vec<String>,
) -> Result<Position, String> {
    let mut x = None;
    let mut y = None;
    // CDML omits z on a two-dimensional point; that absence means zero rather than an
    // unknown value, which is also what OASA's reader records.
    let mut z = 0.0_f64;
    for (namespace, local_name, value) in attributes(tree, node) {
        if !namespace.is_empty() {
            unhandled.push(attribute_context(context, &namespace, &local_name));
            continue;
        }
        match local_name.as_str() {
            "x" => x = Some(coordinate(&value, context)?),
            "y" => y = Some(coordinate(&value, context)?),
            "z" => z = coordinate(&value, context)?,
            _ => unhandled.push(attribute_context(context, &namespace, &local_name)),
        }
    }
    let x = x.ok_or_else(|| format!("{context}: point requires an x coordinate"))?;
    let y = y.ok_or_else(|| format!("{context}: point requires a y coordinate"))?;
    let position = Position::new(x, y, z).map_err(|error| format!("{context}: {error}"))?;
    Ok(position)
}

/// Convert one CDML coordinate into PostScript points.
fn coordinate(value: &str, context: &str) -> Result<f64, String> {
    if let Some(centimetres) = value.strip_suffix("cm") {
        let parsed: f64 = centimetres
            .parse()
            .map_err(|_| format!("{context}: {value} is not a centimetre coordinate"))?;
        return Ok(parsed * POINTS_PER_CENTIMETRE);
    }
    let parsed: f64 = value
        .parse()
        .map_err(|_| format!("{context}: {value} is not a point coordinate"))?;
    Ok(parsed)
}

/// Read the observed bond order from an explicit trailing digit only.
fn bond_order(source_type: &str) -> Option<BondOrder> {
    let digits = source_type.get(1..)?;
    let value: u8 = digits.parse().ok()?;
    let order = match value {
        1 => BondOrder::Single,
        2 => BondOrder::Double,
        3 => BondOrder::Triple,
        4 => BondOrder::Aromatic,
        other => BondOrder::Other(other),
    };
    Some(order)
}

/// Read the observed bond style from the leading CDML type character.
fn bond_style(source_type: &str) -> Option<BondStyle> {
    let character = source_type.chars().next()?;
    let style = match character {
        'n' => BondStyle::Normal,
        'w' => BondStyle::Wedge,
        'h' => BondStyle::Hashed,
        'a' => BondStyle::Adder,
        'b' => BondStyle::Bold,
        'd' => BondStyle::Dashed,
        'o' => BondStyle::Dotted,
        other => BondStyle::Other(other.to_string()),
    };
    Some(style)
}

/// Project one molecule into the shared comparison field set.
fn project_molecule(molecule: &Molecule, molecule_index: usize) -> Result<Value, String> {
    let mut atom_index_by_identity = HashMap::new();
    let mut projected_atoms = Vec::new();
    for (atom_index, atom) in molecule.atoms().iter().enumerate() {
        atom_index_by_identity.insert(atom.identity(), atom_index);
        projected_atoms.push(project_atom(atom, atom_index));
    }
    let mut vertex_position = HashMap::new();
    collect_vertex_positions(&mut vertex_position, molecule.groups(), "group");
    collect_vertex_positions(&mut vertex_position, molecule.texts(), "text");
    collect_vertex_positions(&mut vertex_position, molecule.queries(), "query");
    for (atom_index, atom) in molecule.atoms().iter().enumerate() {
        vertex_position.insert(atom.identity(), ("atom", atom_index));
    }
    let mut projected_bonds = Vec::new();
    for bond in molecule.bonds() {
        projected_bonds.push(project_bond(bond, &vertex_position)?);
    }
    let projected = json!({
        "index": molecule_index,
        "id": molecule.source_id().map(Identifier::as_str),
        "name": molecule.name(),
        "atoms": projected_atoms,
        "bonds": projected_bonds,
        "groups": project_non_atom_vertices(molecule.groups()),
        "texts": project_non_atom_vertices(molecule.texts()),
        "queries": project_non_atom_vertices(molecule.queries()),
    });
    Ok(projected)
}

/// Project one atom into the field set the OASA corpus worker also emits.
fn project_atom(atom: &Atom, atom_index: usize) -> Value {
    let position = atom.position();
    json!({
        "index": atom_index,
        "id": atom.source_id().map(Identifier::as_str),
        "symbol": atom.element(),
        "element": atom.element(),
        "formal_charge": atom.formal_charge(),
        "explicit_hydrogens": atom.explicit_hydrogens(),
        "isotope": atom.isotope(),
        "valence": atom.valence(),
        "multiplicity": atom.multiplicity(),
        "free_sites": atom.free_sites(),
        "x": round_coordinate(position.x()),
        "y": round_coordinate(position.y()),
        "z": round_coordinate(position.z()),
    })
}

/// Project one bond, naming each endpoint by its kind and its index within that kind.
fn project_bond(
    bond: &Bond,
    vertex_position: &HashMap<&RecordId, (&str, usize)>,
) -> Result<Value, String> {
    let (start_kind, start_index) = endpoint_position(bond.start(), vertex_position)?;
    let (end_kind, end_index) = endpoint_position(bond.end(), vertex_position)?;
    let order = bond.order().map(|order| match order {
        BondOrder::Single => 1,
        BondOrder::Double => 2,
        BondOrder::Triple => 3,
        BondOrder::Aromatic => 4,
        BondOrder::Other(value) => value,
    });
    let projected = json!({
        "id": bond.source_id().map(Identifier::as_str),
        "start": start_index,
        "end": end_index,
        "start_kind": start_kind,
        "end_kind": end_kind,
        "type": bond.source_type(),
        "order": order,
        "style": bond.style().map(bond_style_name),
        "aromatic": bond.aromatic(),
    });
    Ok(projected)
}

/// Return the kind name and per-kind index of one typed endpoint.
fn endpoint_position<'a>(
    endpoint: &VertexRef,
    vertex_position: &HashMap<&RecordId, (&'a str, usize)>,
) -> Result<(&'a str, usize), String> {
    let record_id = match endpoint {
        VertexRef::Atom(id) | VertexRef::Group(id) | VertexRef::Text(id) | VertexRef::Query(id) => {
            id
        }
    };
    let position = vertex_position
        .get(record_id)
        .ok_or_else(|| "bond endpoint resolves to no projected vertex".to_owned())?;
    Ok(*position)
}

/// Record where each non-atom vertex sits within its own kind list.
fn collect_vertex_positions<'a>(
    vertex_position: &mut HashMap<&'a RecordId, (&'static str, usize)>,
    vertices: &'a [NonAtomVertex],
    kind_name: &'static str,
) {
    for (index, vertex) in vertices.iter().enumerate() {
        vertex_position.insert(vertex.identity(), (kind_name, index));
    }
}

/// Project the minimal identity the core carries for non-atom vertices.
fn project_non_atom_vertices(vertices: &[NonAtomVertex]) -> Vec<Value> {
    vertices
        .iter()
        .enumerate()
        .map(|(index, vertex)| {
            json!({"index": index, "id": vertex.source_id().map(Identifier::as_str)})
        })
        .collect()
}

/// Return a stable JSON name for one bond style.
fn bond_style_name(style: &BondStyle) -> String {
    let name = match style {
        BondStyle::Normal => "normal",
        BondStyle::Wedge => "wedge",
        BondStyle::Hashed => "hashed",
        BondStyle::Adder => "adder",
        BondStyle::Bold => "bold",
        BondStyle::Dashed => "dashed",
        BondStyle::Dotted => "dotted",
        BondStyle::Wavy => "wavy",
        BondStyle::HaworthFront => "haworth_front",
        BondStyle::Other(value) => value.as_str(),
    };
    name.to_owned()
}

/// Round to the same six decimal places the OASA corpus worker emits.
fn round_coordinate(value: f64) -> f64 {
    (value * 1_000_000.0).round() / 1_000_000.0
}

/// Return the element children of a node, inventorying every other node kind.
fn children_of(
    tree: &Xot,
    node: Node,
    context: &str,
    unhandled: &mut Vec<String>,
) -> Vec<(String, String, Node)> {
    let mut elements = Vec::new();
    for child in tree.children(node) {
        if let Some((local_name, namespace)) = element_name(tree, child) {
            elements.push((local_name, namespace, child));
            continue;
        }
        if let Some(text) = tree.text_str(child) {
            if text.trim().is_empty() {
                continue;
            }
            unhandled.push(format!("{context}/#text"));
            continue;
        }
        unhandled.push(format!("{context}/#node"));
    }
    elements
}

/// Return the namespace, local name, and value of every attribute on a node.
fn attributes(tree: &Xot, node: Node) -> Vec<(String, String, String)> {
    tree.attributes(node)
        .iter()
        .map(|(name, value)| {
            let (local_name, namespace) = tree.name_ns_str(name);
            (namespace.to_owned(), local_name.to_owned(), value.clone())
        })
        .collect()
}

/// Return the local name and namespace of a node when it is an element.
fn element_name(tree: &Xot, node: Node) -> Option<(String, String)> {
    let element = tree.element(node)?;
    let (local_name, namespace) = tree.name_ns_str(element.name());
    Some((local_name.to_owned(), namespace.to_owned()))
}

/// Return the record kind named by a non-atom vertex element.
fn vertex_kind(local_name: &str) -> RecordKind {
    match local_name {
        "group" => RecordKind::Group,
        "text" => RecordKind::Text,
        _ => RecordKind::Query,
    }
}

/// Return the typed endpoint reference for one non-atom vertex kind.
fn vertex_reference(kind: RecordKind, identity: RecordId) -> VertexRef {
    match kind {
        RecordKind::Group => VertexRef::Group(identity),
        RecordKind::Text => VertexRef::Text(identity),
        _ => VertexRef::Query(identity),
    }
}

/// Record the source ID a bond endpoint may name.
fn register_reference(
    references: &mut HashMap<String, VertexRef>,
    source_id: Option<&Identifier>,
    reference: VertexRef,
    context: &str,
) -> Result<(), String> {
    let Some(source_id) = source_id else {
        return Ok(());
    };
    if references
        .insert(source_id.as_str().to_owned(), reference)
        .is_some()
    {
        return Err(format!(
            "{context}: {} repeats a molecule-local source ID",
            source_id.as_str()
        ));
    }
    Ok(())
}

/// Validate one exact source identifier.
fn identifier(value: &str) -> Result<Identifier, String> {
    Identifier::new(value).map_err(|error| error.to_string())
}

/// Parse one carried numeric attribute without supplying a default.
fn parse_scalar<T: std::str::FromStr>(
    value: &str,
    context: &str,
    field: &str,
) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{context}: {field} value {value} is out of range"))
}

/// Format one attribute for the unhandled-content inventory.
fn attribute_context(context: &str, namespace: &str, local_name: &str) -> String {
    if namespace.is_empty() {
        return format!("{context}@{local_name}");
    }
    format!("{context}@{{{namespace}}}{local_name}")
}

/// Format one element for the unhandled-content inventory.
fn element_context(context: &str, namespace: &str, local_name: &str) -> String {
    if namespace.is_empty() {
        return format!("{context}/{local_name}");
    }
    format!("{context}/{{{namespace}}}{local_name}")
}
