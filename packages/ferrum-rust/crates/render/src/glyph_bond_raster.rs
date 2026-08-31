//! Test-only 8x raster layers for independent glyph-bond alignment measurement.
//!
//! The diagnostic consumes accepted V4 batches through the ordinary private
//! draw stream. It does not expose a product, PyO3, CLI, or Qt API.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use thiserror::Error;
use ttf_parser::Face;

use crate::draw_stream_molecule_v1::lower_molecule_batch;
use crate::draw_stream_v1::{
    DrawEllipseV1, DrawLineCapV1, DrawMetadataV1, DrawPathCommandV1, DrawPathV1, DrawRectV1,
    DrawSinkV1, DrawStreamErrorV1, DrawStyleV1, lower_atom_label_core_run_to_sink_v1,
    lower_molecule_plan_to_sink_v1, scoped_translate,
};
use crate::{
    BatchSpace, FerrumFontEnvironmentV1, FerrumFontId, MoleculeRenderPlanV4, RenderBatchContentV4,
    RenderPaintV3, RenderPoint, RenderTarget, RenderViewportV1,
};

pub(crate) const GLYPH_BOND_RASTER_SCALE: u32 = 8;

pub const RUST_FINAL_INK_CAPTURE_PROFILE_ID: &str = "rust_final_ink_8x_400_square_v1";

const MAX_RASTER_PIXELS: u64 = 64 * 1024 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GlyphBondRasterBondIdentity {
    bond_id: String,
    start_atom: String,
    end_atom: String,
    style: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphBondRasterFixtureIdentity {
    fixture_id: String,
    fixture_cdml_sha256: String,
    expected_relations: Vec<GlyphBondRasterRelation>,
    negative_cases: Vec<GlyphBondRasterRelation>,
}

impl GlyphBondRasterFixtureIdentity {
    pub fn from_cdml(
        fixture_id: impl Into<String>,
        fixture_cdml: &str,
        expected_relations: Vec<GlyphBondRasterRelation>,
        negative_cases: Vec<GlyphBondRasterRelation>,
    ) -> Self {
        Self {
            fixture_id: fixture_id.into(),
            fixture_cdml_sha256: sha256_hex(fixture_cdml.as_bytes()),
            expected_relations,
            negative_cases,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct GlyphBondRasterRelation {
    relation: String,
    subject_id: String,
    object_id: String,
    expectation: String,
}

impl GlyphBondRasterRelation {
    pub fn new(
        relation: impl Into<String>,
        subject_id: impl Into<String>,
        object_id: impl Into<String>,
        expectation: impl Into<String>,
    ) -> Self {
        Self {
            relation: relation.into(),
            subject_id: subject_id.into(),
            object_id: object_id.into(),
            expectation: expectation.into(),
        }
    }
}

impl GlyphBondRasterBondIdentity {
    /// Construct one closed fixture graph bond identity.
    pub fn new(
        bond_id: impl Into<String>,
        start_atom: impl Into<String>,
        end_atom: impl Into<String>,
        style: impl Into<String>,
    ) -> Self {
        Self {
            bond_id: bond_id.into(),
            start_atom: start_atom.into(),
            end_atom: end_atom.into(),
            style: style.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlyphBondRasterSourceMapping {
    atom_target_sources: BTreeMap<String, String>,
    atom_source_elements: BTreeMap<String, String>,
    bond_target_sources: BTreeMap<String, String>,
}

impl GlyphBondRasterSourceMapping {
    pub fn new(
        atom_target_sources: BTreeMap<String, String>,
        bond_target_sources: BTreeMap<String, String>,
    ) -> Self {
        let atom_source_elements = atom_target_sources
            .values()
            .map(|source_id| (source_id.clone(), "unknown".to_owned()))
            .collect();
        Self {
            atom_target_sources,
            atom_source_elements,
            bond_target_sources,
        }
    }

    #[must_use]
    pub fn with_atom_elements(
        atom_target_sources: BTreeMap<String, String>,
        atom_source_elements: BTreeMap<String, String>,
        bond_target_sources: BTreeMap<String, String>,
    ) -> Self {
        Self {
            atom_target_sources,
            atom_source_elements,
            bond_target_sources,
        }
    }
}

#[derive(Debug)]
pub struct GlyphBondRasterLayer {
    pixmap: tiny_skia::Pixmap,
}

impl GlyphBondRasterLayer {
    pub fn width(&self) -> u32 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u32 {
        self.pixmap.height()
    }

    #[must_use]
    pub fn nontransparent_pixels(&self) -> usize {
        self.pixmap
            .pixels()
            .iter()
            .filter(|pixel| pixel.alpha() != 0)
            .count()
    }

    fn write_png(&self, path: &Path) -> Result<(), GlyphBondRasterError> {
        let bytes = encode_png(&self.pixmap)?;
        fs::write(path, bytes).map_err(|source| GlyphBondRasterError::Write {
            path: path.to_owned(),
            source,
        })
    }
}

fn encode_png(pixmap: &tiny_skia::Pixmap) -> Result<Vec<u8>, GlyphBondRasterError> {
    let mut rgba = Vec::new();
    rgba.try_reserve(pixmap.data().len())
        .map_err(|_| GlyphBondRasterError::RasterAllocationFailed)?;
    for pixel in pixmap.pixels() {
        let color = pixel.demultiply();
        rgba.extend_from_slice(&[color.red(), color.green(), color.blue(), color.alpha()]);
    }
    let mut output = Vec::new();
    let mut encoder = png::Encoder::new(&mut output, pixmap.width(), pixmap.height());
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder
        .write_header()
        .map_err(|error| GlyphBondRasterError::Png(error.to_string()))?;
    writer
        .write_image_data(&rgba)
        .map_err(|error| GlyphBondRasterError::Png(error.to_string()))?;
    drop(writer);
    Ok(output)
}

#[derive(Debug)]
pub struct GlyphBondRasterLayers {
    normal_composite: GlyphBondRasterLayer,
    target_core_glyph_masks: BTreeMap<String, GlyphBondRasterLayer>,
    full_label_masks: BTreeMap<String, GlyphBondRasterLayer>,
    target_core_glyph_elements: BTreeMap<String, String>,
    final_bond_footprints: BTreeMap<String, GlyphBondRasterLayer>,
}

impl GlyphBondRasterLayers {
    #[must_use]
    pub fn normal_composite(&self) -> &GlyphBondRasterLayer {
        &self.normal_composite
    }

    #[must_use]
    pub fn target_core_glyph_masks(&self) -> &BTreeMap<String, GlyphBondRasterLayer> {
        &self.target_core_glyph_masks
    }

    #[must_use]
    pub fn full_label_masks(&self) -> &BTreeMap<String, GlyphBondRasterLayer> {
        &self.full_label_masks
    }

    #[must_use]
    pub fn final_bond_footprints(&self) -> &BTreeMap<String, GlyphBondRasterLayer> {
        &self.final_bond_footprints
    }

    pub fn write_measurement_manifest_v2(
        &self,
        directory: &Path,
        fixture: &GlyphBondRasterFixtureIdentity,
        bonds: &[GlyphBondRasterBondIdentity],
    ) -> Result<PathBuf, GlyphBondRasterError> {
        fs::create_dir_all(directory).map_err(|source| GlyphBondRasterError::CreateDirectory {
            path: directory.to_owned(),
            source,
        })?;
        validate_bond_identities(
            &self.target_core_glyph_masks,
            &self.final_bond_footprints,
            bonds,
        )?;
        validate_atom_elements(
            &self.target_core_glyph_masks,
            &self.target_core_glyph_elements,
        )?;
        if self.full_label_masks.len() != self.target_core_glyph_masks.len()
            || self
                .target_core_glyph_masks
                .keys()
                .any(|atom_id| !self.full_label_masks.contains_key(atom_id))
        {
            return Err(GlyphBondRasterError::IncompleteFullLabelMasks);
        }
        if self.normal_composite.width() != 3200 || self.normal_composite.height() != 3200 {
            return Err(GlyphBondRasterError::UnexpectedCaptureProfileExtent {
                width: self.normal_composite.width(),
                height: self.normal_composite.height(),
            });
        }

        let composite_name = "final_composite.png".to_owned();
        let composite = write_hashed_layer(directory, &composite_name, &self.normal_composite)?;
        let core_names = write_layers(directory, "core_glyph", &self.target_core_glyph_masks)?;
        let full_label_names = write_layers(directory, "full_label", &self.full_label_masks)?;
        let footprint_names = write_layers(directory, "final_bond", &self.final_bond_footprints)?;
        let atoms = core_names
            .into_iter()
            .map(|(atom_id, core_glyph_layer)| MeasurementAtomV2 {
                atom_id: atom_id.clone(),
                core_glyph_layer,
                full_label_layer: full_label_names
                    .get(&atom_id)
                    .expect("validated full-label layer")
                    .clone(),
            })
            .collect();
        let bond_layers = bonds
            .iter()
            .map(|bond| MeasurementBondV2 {
                bond_id: bond.bond_id.clone(),
                final_bond_layer: footprint_names
                    .get(&bond.bond_id)
                    .expect("validated bond footprint")
                    .clone(),
            })
            .collect();
        let graph = MeasurementGraphV2 {
            atoms: self
                .target_core_glyph_elements
                .iter()
                .map(|(atom_id, element)| MeasurementGraphAtomV2 {
                    atom_id: atom_id.clone(),
                    element: element.clone(),
                })
                .collect(),
            bonds: bonds
                .iter()
                .map(|identity| MeasurementGraphBondV2 {
                    bond_id: identity.bond_id.clone(),
                    start_atom_id: identity.start_atom.clone(),
                    end_atom_id: identity.end_atom.clone(),
                    style: identity.style.clone(),
                })
                .collect(),
        };
        let manifest = MeasurementManifestV2 {
            schema: "ferrum-measure-stack-raster-layers-v2",
            fixture_id: fixture.fixture_id.clone(),
            fixture_cdml_sha256: fixture.fixture_cdml_sha256.clone(),
            capture_profile: MeasurementCaptureProfileV2::rust_final_ink(),
            graph,
            composite_layer: composite,
            atom_layers: atoms,
            bond_layers,
            expected_relations: fixture.expected_relations.clone(),
            negative_cases: fixture.negative_cases.clone(),
        };
        let path = directory.join("raster_layer_manifest_v2.json");
        let document = serde_json::to_vec_pretty(&manifest)
            .map_err(|error| GlyphBondRasterError::Manifest(error.to_string()))?;
        fs::write(&path, document).map_err(|source| GlyphBondRasterError::Write {
            path: path.clone(),
            source,
        })?;
        Ok(path)
    }
}

#[derive(Debug, Error)]
pub enum GlyphBondRasterError {
    #[error("8x glyph-bond raster dimensions are not representable")]
    RasterDimensionsUnsupported,
    #[error("8x glyph-bond raster needs {required} pixels, over the {limit} pixel limit")]
    RasterAllocationLimit { required: u64, limit: u64 },
    #[error("could not allocate 8x glyph-bond raster")]
    RasterAllocationFailed,
    #[error("glyph-bond raster contains non-finite geometry")]
    NonFiniteGeometry,
    #[error("could not parse verified Telex outline face: {0}")]
    Font(String),
    #[error("required Telex glyph {glyph_index} has no usable outline")]
    MissingGlyphOutline { glyph_index: u32 },
    #[error("could not encode diagnostic PNG: {0}")]
    Png(String),
    #[error("could not create diagnostic directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not write diagnostic artifact {path}: {source}")]
    Write {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("could not serialize diagnostic manifest: {0}")]
    Manifest(String),
    #[error("fixture bond identity {bond_id} is repeated")]
    DuplicateBondIdentity { bond_id: String },
    #[error("fixture bond identity {bond_id} has no rendered footprint")]
    MissingBondFootprint { bond_id: String },
    #[error("fixture bond identity {bond_id} references absent core glyph {atom_id}")]
    MissingCoreGlyph { bond_id: String, atom_id: String },
    #[error("target core glyph {atom_id} has no fixture element identity")]
    MissingCoreElement { atom_id: String },
    #[error("fixture element identity {atom_id} has no target core glyph")]
    StaleCoreElement { atom_id: String },
    #[error("each target core glyph needs one matching full-label mask")]
    IncompleteFullLabelMasks,
    #[error("V2 Rust final-ink capture must use the fixed 3200x3200 profile, got {width}x{height}")]
    UnexpectedCaptureProfileExtent { width: u32, height: u32 },
    #[error("rendered bond footprint {bond_id} has no fixture graph identity")]
    UnidentifiedBondFootprint { bond_id: String },
    #[error("rendered {kind} target {target_id} has no fixture source identity")]
    MissingSourceMapping {
        kind: &'static str,
        target_id: String,
    },
    #[error("fixture {kind} source identity {source_id} is repeated")]
    DuplicateSourceMapping {
        kind: &'static str,
        source_id: String,
    },
    #[error("fixture source mapping contains stale {kind} target {target_id}")]
    StaleSourceMapping {
        kind: &'static str,
        target_id: String,
    },
}

pub fn rasterize_glyph_bond_layers(
    plan: &MoleculeRenderPlanV4,
    viewport: RenderViewportV1,
    source_mapping: &GlyphBondRasterSourceMapping,
) -> Result<GlyphBondRasterLayers, GlyphBondRasterError> {
    let mut normal = RasterSink::new(viewport)?;
    lower_molecule_plan_to_sink_v1(plan, viewport, &mut normal).map_err(map_draw_error)?;

    let environment = FerrumFontEnvironmentV1::load()
        .map_err(|error| GlyphBondRasterError::Font(error.to_string()))?;
    let face = Face::parse(environment.descriptor(FerrumFontId::TelexRegular).data(), 0)
        .map_err(|error| GlyphBondRasterError::Font(error.to_string()))?;
    let mut target_core_glyph_masks = BTreeMap::new();
    let mut full_label_masks = BTreeMap::new();
    let mut final_bond_footprints = BTreeMap::new();
    for batch in plan.batches() {
        let target_id = target_identity(batch.target());
        match batch.content() {
            RenderBatchContentV4::Atom(atom) => {
                let mut core = RasterSink::new(viewport)?;
                core.begin_page(viewport)?;
                core.begin_molecule_target_group(
                    batch.target(),
                    batch.paint_order(),
                    batch.coordinate_space(),
                )?;
                scoped_translate(atom.atom_local_anchor(), &mut core, |sink| {
                    lower_atom_label_core_run_to_sink_v1(atom.label(), &face, sink)
                })
                .map_err(map_draw_error)?;
                core.end_molecule_batch()?;
                core.finish_page()?;
                let source_id = source_mapping
                    .atom_target_sources
                    .get(&target_id)
                    .ok_or_else(|| GlyphBondRasterError::MissingSourceMapping {
                        kind: "atom",
                        target_id: target_id.clone(),
                    })?;
                target_core_glyph_masks.insert(source_id.clone(), core.into_layer());
                let mut full_label = RasterSink::new(viewport)?;
                full_label.begin_page(viewport)?;
                lower_molecule_batch(batch, &face, &mut full_label).map_err(map_draw_error)?;
                full_label.finish_page()?;
                full_label_masks.insert(source_id.clone(), full_label.into_layer());
            }
            RenderBatchContentV4::Bond(_) => {
                let mut footprint = RasterSink::new(viewport)?;
                footprint.begin_page(viewport)?;
                lower_molecule_batch(batch, &face, &mut footprint).map_err(map_draw_error)?;
                footprint.finish_page()?;
                let source_id = source_mapping
                    .bond_target_sources
                    .get(&target_id)
                    .ok_or_else(|| GlyphBondRasterError::MissingSourceMapping {
                        kind: "bond",
                        target_id: target_id.clone(),
                    })?;
                final_bond_footprints.insert(source_id.clone(), footprint.into_layer());
            }
            RenderBatchContentV4::CompactGroup(_) => {}
        }
    }
    validate_source_mapping(
        &source_mapping.atom_target_sources,
        &target_core_glyph_masks,
        plan,
        "atom",
    )?;
    validate_source_mapping(
        &source_mapping.bond_target_sources,
        &final_bond_footprints,
        plan,
        "bond",
    )?;
    Ok(GlyphBondRasterLayers {
        normal_composite: normal.into_layer(),
        target_core_glyph_masks,
        full_label_masks,
        target_core_glyph_elements: source_mapping.atom_source_elements.clone(),
        final_bond_footprints,
    })
}

fn target_identity(target: &RenderTarget) -> String {
    target.document_object_id().as_str().to_owned()
}

fn validate_source_mapping(
    mapping: &BTreeMap<String, String>,
    layers: &BTreeMap<String, GlyphBondRasterLayer>,
    plan: &MoleculeRenderPlanV4,
    kind: &'static str,
) -> Result<(), GlyphBondRasterError> {
    let expected_targets = plan
        .batches()
        .iter()
        .filter(|batch| {
            matches!(
                (kind, batch.content()),
                ("atom", RenderBatchContentV4::Atom(_)) | ("bond", RenderBatchContentV4::Bond(_))
            )
        })
        .map(|batch| target_identity(batch.target()))
        .collect::<BTreeSet<_>>();
    for target_id in &expected_targets {
        if !mapping.contains_key(target_id) {
            return Err(GlyphBondRasterError::MissingSourceMapping {
                kind,
                target_id: target_id.clone(),
            });
        }
    }
    if let Some(target_id) = mapping
        .keys()
        .find(|target_id| !expected_targets.contains(*target_id))
    {
        return Err(GlyphBondRasterError::StaleSourceMapping {
            kind,
            target_id: target_id.clone(),
        });
    }
    let mut source_ids = BTreeSet::new();
    for source_id in mapping.values() {
        if !source_ids.insert(source_id) {
            return Err(GlyphBondRasterError::DuplicateSourceMapping {
                kind,
                source_id: source_id.clone(),
            });
        }
    }
    if layers.len() != source_ids.len() {
        return Err(GlyphBondRasterError::RasterAllocationFailed);
    }
    Ok(())
}

fn map_draw_error(error: DrawStreamErrorV1<GlyphBondRasterError>) -> GlyphBondRasterError {
    match error {
        DrawStreamErrorV1::ResourceExhausted => GlyphBondRasterError::RasterAllocationFailed,
        DrawStreamErrorV1::NonFiniteGeometry => GlyphBondRasterError::NonFiniteGeometry,
        DrawStreamErrorV1::Font(message) => GlyphBondRasterError::Font(message),
        DrawStreamErrorV1::MissingGlyphOutline { glyph_index } => {
            GlyphBondRasterError::MissingGlyphOutline { glyph_index }
        }
        DrawStreamErrorV1::InvalidComposite => GlyphBondRasterError::NonFiniteGeometry,
        DrawStreamErrorV1::Sink(error) => error,
    }
}

fn write_hashed_layer(
    directory: &Path,
    name: &str,
    layer: &GlyphBondRasterLayer,
) -> Result<MeasurementLayerV2, GlyphBondRasterError> {
    let path = directory.join(name);
    layer.write_png(&path)?;
    let bytes = fs::read(&path).map_err(|source| GlyphBondRasterError::Write { path, source })?;
    Ok(MeasurementLayerV2 {
        relative_path: name.to_owned(),
        sha256: sha256_hex(&bytes),
    })
}

fn write_layers(
    directory: &Path,
    prefix: &str,
    layers: &BTreeMap<String, GlyphBondRasterLayer>,
) -> Result<BTreeMap<String, MeasurementLayerV2>, GlyphBondRasterError> {
    let mut names = BTreeMap::new();
    for (index, (identity, layer)) in layers.iter().enumerate() {
        let name = format!("{prefix}_{index:04}.png");
        names.insert(
            identity.clone(),
            write_hashed_layer(directory, &name, layer)?,
        );
    }
    Ok(names)
}

fn validate_bond_identities(
    core_masks: &BTreeMap<String, GlyphBondRasterLayer>,
    footprints: &BTreeMap<String, GlyphBondRasterLayer>,
    bonds: &[GlyphBondRasterBondIdentity],
) -> Result<(), GlyphBondRasterError> {
    let mut identities = BTreeSet::new();
    for bond in bonds {
        if !identities.insert(&bond.bond_id) {
            return Err(GlyphBondRasterError::DuplicateBondIdentity {
                bond_id: bond.bond_id.clone(),
            });
        }
        if !footprints.contains_key(&bond.bond_id) {
            return Err(GlyphBondRasterError::MissingBondFootprint {
                bond_id: bond.bond_id.clone(),
            });
        }
        for atom_id in [&bond.start_atom, &bond.end_atom] {
            if !core_masks.contains_key(atom_id) {
                return Err(GlyphBondRasterError::MissingCoreGlyph {
                    bond_id: bond.bond_id.clone(),
                    atom_id: atom_id.clone(),
                });
            }
        }
    }
    if let Some(identity) = footprints
        .keys()
        .find(|identity| !identities.contains(*identity))
    {
        return Err(GlyphBondRasterError::UnidentifiedBondFootprint {
            bond_id: identity.clone(),
        });
    }
    Ok(())
}

fn validate_atom_elements(
    core_masks: &BTreeMap<String, GlyphBondRasterLayer>,
    elements: &BTreeMap<String, String>,
) -> Result<(), GlyphBondRasterError> {
    for atom_id in core_masks.keys() {
        if !elements.contains_key(atom_id) {
            return Err(GlyphBondRasterError::MissingCoreElement {
                atom_id: atom_id.clone(),
            });
        }
    }
    if let Some(atom_id) = elements
        .keys()
        .find(|atom_id| !core_masks.contains_key(*atom_id))
    {
        return Err(GlyphBondRasterError::StaleCoreElement {
            atom_id: atom_id.clone(),
        });
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementManifestV2 {
    schema: &'static str,
    fixture_id: String,
    fixture_cdml_sha256: String,
    capture_profile: MeasurementCaptureProfileV2,
    graph: MeasurementGraphV2,
    composite_layer: MeasurementLayerV2,
    atom_layers: Vec<MeasurementAtomV2>,
    bond_layers: Vec<MeasurementBondV2>,
    expected_relations: Vec<GlyphBondRasterRelation>,
    negative_cases: Vec<GlyphBondRasterRelation>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementCaptureProfileV2 {
    profile_id: &'static str,
    source_rect: [f64; 4],
    pixel_width: u32,
    pixel_height: u32,
    device_pixel_ratio: f64,
    scene_evaluation: &'static str,
}

impl MeasurementCaptureProfileV2 {
    const fn rust_final_ink() -> Self {
        Self {
            profile_id: RUST_FINAL_INK_CAPTURE_PROFILE_ID,
            source_rect: [-200.0, -200.0, 400.0, 400.0],
            pixel_width: 3200,
            pixel_height: 3200,
            device_pixel_ratio: 8.0,
            scene_evaluation: "raw_final_ink",
        }
    }
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementGraphV2 {
    atoms: Vec<MeasurementGraphAtomV2>,
    bonds: Vec<MeasurementGraphBondV2>,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementGraphAtomV2 {
    atom_id: String,
    element: String,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementGraphBondV2 {
    bond_id: String,
    start_atom_id: String,
    end_atom_id: String,
    style: String,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementAtomV2 {
    atom_id: String,
    core_glyph_layer: MeasurementLayerV2,
    full_label_layer: MeasurementLayerV2,
}

#[derive(Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementBondV2 {
    bond_id: String,
    final_bond_layer: MeasurementLayerV2,
}

#[derive(Clone, Serialize)]
#[serde(deny_unknown_fields)]
struct MeasurementLayerV2 {
    relative_path: String,
    sha256: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

struct RasterSink {
    pixmap: tiny_skia::Pixmap,
    page_transform: tiny_skia::Transform,
    translations: Vec<(f64, f64)>,
    current_translation: (f64, f64),
}

impl RasterSink {
    fn new(viewport: RenderViewportV1) -> Result<Self, GlyphBondRasterError> {
        let width = raster_dimension(viewport.width())?;
        let height = raster_dimension(viewport.height())?;
        let required = u64::from(width) * u64::from(height);
        if required > MAX_RASTER_PIXELS {
            return Err(GlyphBondRasterError::RasterAllocationLimit {
                required,
                limit: MAX_RASTER_PIXELS,
            });
        }
        let pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or(GlyphBondRasterError::RasterAllocationFailed)?;
        let scale = f64::from(GLYPH_BOND_RASTER_SCALE);
        let page_transform = transform(
            scale,
            0.0,
            0.0,
            scale,
            -viewport.x() * scale,
            -viewport.y() * scale,
        )?;
        Ok(Self {
            pixmap,
            page_transform,
            translations: Vec::new(),
            current_translation: (0.0, 0.0),
        })
    }

    fn into_layer(self) -> GlyphBondRasterLayer {
        GlyphBondRasterLayer {
            pixmap: self.pixmap,
        }
    }

    fn current_transform(&self) -> Result<tiny_skia::Transform, GlyphBondRasterError> {
        let (x, y) = self.current_translation;
        Ok(self
            .page_transform
            .pre_concat(transform(1.0, 0.0, 0.0, 1.0, x, y)?))
    }

    fn path(&self, path: &DrawPathV1) -> Result<tiny_skia::Path, GlyphBondRasterError> {
        let mut builder = tiny_skia::PathBuilder::new();
        for command in &path.commands {
            match *command {
                DrawPathCommandV1::MoveTo(point) => {
                    builder.move_to(f32_value(point.x())?, f32_value(point.y())?)
                }
                DrawPathCommandV1::LineTo(point) => {
                    builder.line_to(f32_value(point.x())?, f32_value(point.y())?)
                }
                DrawPathCommandV1::QuadraticTo { control, end } => builder.quad_to(
                    f32_value(control.x())?,
                    f32_value(control.y())?,
                    f32_value(end.x())?,
                    f32_value(end.y())?,
                ),
                DrawPathCommandV1::CubicTo {
                    control_1,
                    control_2,
                    end,
                } => builder.cubic_to(
                    f32_value(control_1.x())?,
                    f32_value(control_1.y())?,
                    f32_value(control_2.x())?,
                    f32_value(control_2.y())?,
                    f32_value(end.x())?,
                    f32_value(end.y())?,
                ),
                DrawPathCommandV1::Close => builder.close(),
            }
        }
        builder
            .finish()
            .ok_or(GlyphBondRasterError::NonFiniteGeometry)
    }

    fn draw_style(
        &mut self,
        path: &tiny_skia::Path,
        style: DrawStyleV1<'_>,
        transform: tiny_skia::Transform,
    ) -> Result<(), GlyphBondRasterError> {
        if style.fill.is_some() {
            self.pixmap.fill_path(
                path,
                &mask_paint(),
                tiny_skia::FillRule::EvenOdd,
                transform,
                None,
            );
        }
        if let Some(stroke) = style.stroke {
            let stroke = tiny_skia::Stroke {
                width: f32_value(stroke.width.get())?,
                miter_limit: f32_value(stroke.miter_limit)?,
                line_cap: match stroke.line_cap {
                    DrawLineCapV1::Butt => tiny_skia::LineCap::Butt,
                    DrawLineCapV1::Round => tiny_skia::LineCap::Round,
                },
                line_join: tiny_skia::LineJoin::Miter,
                dash: None,
            };
            self.pixmap
                .stroke_path(path, &mask_paint(), &stroke, transform, None);
        }
        Ok(())
    }
}

impl DrawSinkV1 for RasterSink {
    type Error = GlyphBondRasterError;

    fn begin_page(&mut self, _: RenderViewportV1) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_root(
        &mut self,
        _: u32,
        _: &ferrum_document_projection::DocumentObjectIdV1,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_root(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_molecule_batch(&mut self, _: u32, _: BatchSpace) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_molecule_batch(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_document_text(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
    fn begin_text_operation(&mut self, _: i32, _: &RenderPaintV3) -> Result<(), Self::Error> {
        Ok(())
    }
    fn end_text_operation(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    fn save(&mut self) -> Result<(), Self::Error> {
        self.translations.push(self.current_translation);
        Ok(())
    }

    fn concat_translate(&mut self, anchor: RenderPoint) -> Result<(), Self::Error> {
        self.current_translation.0 += anchor.x();
        self.current_translation.1 += anchor.y();
        if !self.current_translation.0.is_finite() || !self.current_translation.1.is_finite() {
            return Err(GlyphBondRasterError::NonFiniteGeometry);
        }
        Ok(())
    }

    fn restore(&mut self) -> Result<(), Self::Error> {
        self.current_translation = self
            .translations
            .pop()
            .ok_or(GlyphBondRasterError::NonFiniteGeometry)?;
        Ok(())
    }

    fn fill_rect(
        &mut self,
        rect: DrawRectV1,
        _: &RenderPaintV3,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        let rect = tiny_skia::Rect::from_xywh(
            f32_value(rect.origin.x())?,
            f32_value(rect.origin.y())?,
            f32_value(rect.width.get())?,
            f32_value(rect.height.get())?,
        )
        .ok_or(GlyphBondRasterError::NonFiniteGeometry)?;
        self.pixmap
            .fill_rect(rect, &mask_paint(), self.current_transform()?, None);
        Ok(())
    }

    fn draw_path(
        &mut self,
        path: &DrawPathV1,
        style: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        let path = self.path(path)?;
        self.draw_style(&path, style, self.current_transform()?)
    }

    fn draw_ellipse(
        &mut self,
        ellipse: DrawEllipseV1,
        style: DrawStyleV1<'_>,
        _: DrawMetadataV1,
    ) -> Result<(), Self::Error> {
        let mut builder = tiny_skia::PathBuilder::new();
        let rect = tiny_skia::Rect::from_xywh(
            f32_value(ellipse.center.x() - ellipse.radius_x.get())?,
            f32_value(ellipse.center.y() - ellipse.radius_y.get())?,
            f32_value(ellipse.radius_x.get() * 2.0)?,
            f32_value(ellipse.radius_y.get() * 2.0)?,
        )
        .ok_or(GlyphBondRasterError::NonFiniteGeometry)?;
        builder.push_oval(rect);
        let path = builder
            .finish()
            .ok_or(GlyphBondRasterError::NonFiniteGeometry)?;
        let rotation = tiny_skia::Transform::from_rotate_at(
            f32_value(ellipse.rotation_degrees)?,
            f32_value(ellipse.center.x())?,
            f32_value(ellipse.center.y())?,
        );
        self.draw_style(&path, style, self.current_transform()?.pre_concat(rotation))
    }

    fn finish_page(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn raster_dimension(extent: f64) -> Result<u32, GlyphBondRasterError> {
    let pixels = extent * f64::from(GLYPH_BOND_RASTER_SCALE);
    if !pixels.is_finite() || pixels <= 0.0 || pixels.ceil() > f64::from(u32::MAX) {
        return Err(GlyphBondRasterError::RasterDimensionsUnsupported);
    }
    Ok(pixels.ceil() as u32)
}

fn transform(
    sx: f64,
    kx: f64,
    ky: f64,
    sy: f64,
    tx: f64,
    ty: f64,
) -> Result<tiny_skia::Transform, GlyphBondRasterError> {
    Ok(tiny_skia::Transform::from_row(
        f32_value(sx)?,
        f32_value(ky)?,
        f32_value(kx)?,
        f32_value(sy)?,
        f32_value(tx)?,
        f32_value(ty)?,
    ))
}

fn f32_value(value: f64) -> Result<f32, GlyphBondRasterError> {
    let converted = value as f32;
    if converted.is_finite() && (value == 0.0 || converted != 0.0) {
        Ok(converted)
    } else {
        Err(GlyphBondRasterError::NonFiniteGeometry)
    }
}
fn mask_paint() -> tiny_skia::Paint<'static> {
    tiny_skia::Paint {
        shader: tiny_skia::Shader::SolidColor(tiny_skia::Color::from_rgba8(0, 0, 0, 255)),
        blend_mode: tiny_skia::BlendMode::SourceOver,
        anti_alias: true,
        colorspace: tiny_skia::ColorSpace::Linear,
        force_hq_pipeline: false,
    }
}
#[cfg(test)]
#[path = "glyph_bond_raster_tests.rs"]
mod tests;
