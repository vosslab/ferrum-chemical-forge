//! Tests for descriptor-dispatched atomic interchange document admission.

use super::*;
use std::{fs, io::Cursor};

use ferrum_chemistry::{
    AtomicNumber, ChemistryError, ImportedSdfRecord, KekulizeOptions, SmilesMolecule,
};

const TWO_FRAGMENT_CDXML: &str = r#"<CDXML Name="import"><page HeightPages="1"><fragment id="source-first"><n id="carbon" p="0 0"/></fragment><fragment id="source-second"><n id="oxygen" p="30 0" Element="8"/></fragment></page></CDXML>"#;
const FIXED_SINGLE_PRESENTATION_CDXML: &str = r#"<CDXML><page><fragment id="source-fragment"><n id="a" p="0 0"/><n id="b" p="20 0"/><b B="a" E="b" Display="Wavy"/><n id="c" p="40 0"/><b B="b" E="c" Display="Bold"/><n id="d" p="60 0"/><b B="c" E="d" Display="Dash"/></fragment></page></CDXML>"#;
const VALID_THEN_INVALID_PRESENTATION_CDXML: &str = r#"<CDXML><page><fragment id="first"><n id="a" p="0 0"/><n id="b" p="20 0"/><b B="a" E="b" Display="Wavy"/></fragment><fragment id="later-invalid"><n id="c" p="40 0"/><n id="d" p="60 0"/><b B="c" E="d" Order="2" Display="Dash"/></fragment></page></CDXML>"#;
const CDXML_WITH_LEXICAL_AND_VIEW_LOSSES: &str = r#"<?xml version="1.0" encoding="UTF-8"?><!DOCTYPE CDXML SYSTEM "https://static.chemistry.revvitycloud.com/cdxml/CDXML.dtd"><CDXML CreationProgram="ChemDraw 23.0"><page HeightPages="1"><fragment id="source-fragment"><n id="source-atom" p="0 0"/></fragment></page></CDXML>"#;
const SINGLE_ATOM_CML: &str = r#"<cml xmlns="http://www.xml-cml.org/schema/cml2/core"><molecule><atomArray><atom id="a" elementType="C" x2="0" y2="0"/></atomArray></molecule></cml>"#;
const SINGLE_ATOM_SDF: &str = concat!(
    "Ferrum SDF\n",
    "  Ferrum\n",
    "\n",
    "  1  0  0  0  0  0  0  0  0  0999 V2000\n",
    "    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n",
    "M  END\n",
    "$$$$\n",
);

fn cdxml_descriptor() -> &'static InterchangeFormatDescriptorV1 {
    crate::InterchangeFormatRegistryV1::lookup_input_alias("cdxml").expect("CDXML descriptor")
}

fn cdxml_provenance() -> DocumentInterchangeProvenanceV1 {
    DocumentInterchangeProvenanceV1 {
        format_id: cdxml_descriptor().format_id().to_owned(),
        profile_id: cdxml_descriptor().profile_id().to_owned(),
        source_kind: crate::protocol::DocumentInterchangeSourceKindV1::RequestText,
    }
}

fn descriptor(alias: &str) -> &'static InterchangeFormatDescriptorV1 {
    crate::InterchangeFormatRegistryV1::lookup_input_alias(alias).expect("registered descriptor")
}

fn provenance(alias: &str) -> DocumentInterchangeProvenanceV1 {
    let descriptor = descriptor(alias);
    DocumentInterchangeProvenanceV1 {
        format_id: descriptor.format_id().to_owned(),
        profile_id: descriptor.profile_id().to_owned(),
        source_kind: crate::protocol::DocumentInterchangeSourceKindV1::RequestText,
    }
}

struct InjectedSdfEngine;

impl ChemEngine for InjectedSdfEngine {
    fn smiles_to_molecule(&self, _: &str) -> Result<SmilesMolecule, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "smiles_to_molecule",
        })
    }

    fn generate_2d_coordinates(&self, _: &MolGraph) -> Result<Coordinates, ChemistryError> {
        Err(ChemistryError::OperationUnavailable {
            operation: "generate_2d_coordinates",
        })
    }

    fn sdf_to_records(&self, _: &str) -> Result<Vec<ImportedSdfRecord>, ChemistryError> {
        let graph = MolGraph::new(
            vec![
                MolAtom::new(
                    AtomicNumber::try_from(6).expect("carbon"),
                    Some(0),
                    None,
                    None,
                    false,
                )
                .expect("carbon atom"),
            ],
            Vec::new(),
            Some(Coordinates::new(vec![
                ChemistryPoint2::new(0.0, 0.0).expect("finite point"),
            ])),
        )
        .expect("one-atom graph");
        Ok(vec![ImportedSdfRecord::new(
            SmilesMolecule::new("C", graph).expect("complete molecule"),
            "injected SDF record".to_owned(),
            Vec::new(),
        )])
    }

    fn kekulize(
        &self,
        molecule: &MolGraph,
        _: KekulizeOptions,
    ) -> Result<MolGraph, ChemistryError> {
        Ok(molecule.clone())
    }
}

struct InjectedSdfRuntime(InjectedSdfEngine);

impl crate::protocol::runtime::ChemistryRuntimeV1 for InjectedSdfRuntime {
    fn with_engine<T>(
        &self,
        operation: impl FnOnce(
            &dyn ChemEngine,
        ) -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1>,
    ) -> Result<T, crate::protocol::runtime::ChemistryRuntimeErrorV1> {
        operation(&self.0)
    }
}

fn temporary_path(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!(
        "ferrum-interchange-{name}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("monotonic wall clock")
            .as_nanos(),
    ))
}

#[test]
fn every_registered_descriptor_refuses_sources_above_its_limit() {
    enum SourceKind {
        RequestText,
        StandardInput,
        RegularFile,
    }

    for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
        let over_limit = vec![b'x'; descriptor.limits().max_source_bytes() + 1];
        let path = temporary_path(descriptor.format_id());
        fs::write(&path, &over_limit).expect("write bounded source");
        for source_kind in [
            SourceKind::RequestText,
            SourceKind::StandardInput,
            SourceKind::RegularFile,
        ] {
            let result = match source_kind {
                SourceKind::RequestText => admit_interchange_source_v1(
                    descriptor,
                    InterchangeSourceInputV1::RequestText(&over_limit),
                ),
                SourceKind::StandardInput => {
                    let mut stdin = Cursor::new(&over_limit);
                    admit_interchange_source_v1(
                        descriptor,
                        InterchangeSourceInputV1::StandardInput(&mut stdin),
                    )
                }
                SourceKind::RegularFile => admit_interchange_source_v1(
                    descriptor,
                    InterchangeSourceInputV1::RegularFile(&path),
                ),
            };
            assert!(matches!(
                result,
                Err(refusal) if refusal == InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InputBytesLimit)
            ));
        }
        fs::remove_file(path).expect("remove bounded source");
    }
}

#[test]
fn every_registered_descriptor_preserves_admitted_bytes_and_provenance() {
    for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
        let bytes = format!("{} source bytes", descriptor.format_id()).into_bytes();
        assert!(bytes.len() <= descriptor.limits().max_source_bytes());
        assert!(!descriptor.profile_id().is_empty());

        let request =
            admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RequestText(&bytes))
                .expect("request source should be admitted");
        assert_eq!(request.bytes(), bytes);
        assert_eq!(
            request.source_kind(),
            crate::protocol::DocumentInterchangeSourceKindV1::RequestText
        );

        let mut stdin = Cursor::new(bytes.clone());
        let standard_input = admit_interchange_source_v1(
            descriptor,
            InterchangeSourceInputV1::StandardInput(&mut stdin),
        )
        .expect("standard input should be admitted");
        assert_eq!(standard_input.bytes(), bytes);
        assert_eq!(
            standard_input.source_kind(),
            crate::protocol::DocumentInterchangeSourceKindV1::StandardInput
        );

        let path = temporary_path(descriptor.format_id());
        fs::write(&path, &bytes).expect("write regular source");
        let regular_file =
            admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(&path))
                .expect("regular file should be admitted");
        assert_eq!(regular_file.bytes(), bytes);
        assert_eq!(
            regular_file.source_kind(),
            crate::protocol::DocumentInterchangeSourceKindV1::RegularFile
        );
        assert!(regular_file.retained_source().is_some());
        drop(regular_file);
        fs::remove_file(path).expect("remove regular source");
    }
}

#[test]
fn non_regular_file_is_redacted_at_shared_source_boundary() {
    let path = temporary_path("directory");
    fs::create_dir(&path).expect("make temporary directory");
    for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
        assert!(matches!(
            admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(&path)),
            Err(refusal) if refusal == generic_source_refusal()
        ));
    }
    fs::remove_dir(path).expect("remove temporary directory");
}

#[cfg(unix)]
#[test]
fn symlink_is_refused_before_opening_at_shared_source_boundary() {
    use std::os::unix::fs::symlink;

    let target = temporary_path("target");
    let link = temporary_path("link");
    fs::write(&target, b"source").expect("write target");
    symlink(&target, &link).expect("create symlink");
    for descriptor in crate::InterchangeFormatRegistryV1::descriptors() {
        assert!(matches!(
            admit_interchange_source_v1(descriptor, InterchangeSourceInputV1::RegularFile(&link)),
            Err(refusal) if refusal == generic_source_refusal()
        ));
    }
    fs::remove_file(link).expect("remove symlink");
    fs::remove_file(target).expect("remove target");
}

#[test]
fn cdxml_commit_retains_fragment_order_in_public_document_observation() {
    let source = admit_interchange_source_v1(
        cdxml_descriptor(),
        InterchangeSourceInputV1::RequestText(TWO_FRAGMENT_CDXML.as_bytes()),
    )
    .expect("admitted CDXML source");
    let prepared = prepare_interchange_new_document_v1(
        cdxml_descriptor(),
        &source,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        cdxml_provenance(),
    )
    .expect("atomic preparation");
    let (session, receipt) = prepared.commit_and_take_session().expect("one commit");

    assert_eq!(receipt.imported_record_count, 2);
    assert_eq!(
        receipt.loss_report,
        DocumentInterchangeImportLossReportV1 {
            source_identifiers_reallocated: true,
            dropped_categories: vec![DocumentInterchangeLossCategoryV1::DocumentViewMetadata],
        }
    );
    let observation = session.observe(1).expect("committed document observation");
    assert_eq!(
        observation
            .projection()
            .molecules()
            .iter()
            .map(|molecule| molecule.atoms()[0].element())
            .collect::<Vec<_>>(),
        [Some("C"), Some("O")]
    );
    let page = observation.projection().paper_layout().page();
    let positions = observation
        .projection()
        .molecules()
        .iter()
        .map(|molecule| molecule.atoms()[0].position())
        .collect::<Vec<_>>();
    assert_eq!(
        (positions[0].x() + positions[1].x()) / 2.0,
        (page.scene_left() + page.scene_right()) / 2.0,
    );
    assert_eq!(
        (positions[0].y() + positions[1].y()) / 2.0,
        (page.scene_top() + page.scene_bottom()) / 2.0,
    );
}

#[test]
fn cdxml_receipt_reports_declared_lexical_and_view_losses_in_order() {
    let source = admit_interchange_source_v1(
        cdxml_descriptor(),
        InterchangeSourceInputV1::RequestText(CDXML_WITH_LEXICAL_AND_VIEW_LOSSES.as_bytes()),
    )
    .expect("admitted CDXML source");
    let prepared = prepare_interchange_new_document_v1(
        cdxml_descriptor(),
        &source,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        cdxml_provenance(),
    )
    .expect("atomic preparation");
    let (_, receipt) = prepared.commit_and_take_session().expect("one commit");

    assert_eq!(
        receipt.loss_report.dropped_categories,
        vec![
            DocumentInterchangeLossCategoryV1::LexicalSyntax,
            DocumentInterchangeLossCategoryV1::DocumentViewMetadata,
        ]
    );
}

#[test]
fn cdxml_fixed_single_presentations_commit_as_durable_tokens_with_clean_rendering() {
    let source = admit_interchange_source_v1(
        cdxml_descriptor(),
        InterchangeSourceInputV1::RequestText(FIXED_SINGLE_PRESENTATION_CDXML.as_bytes()),
    )
    .expect("admitted CDXML source");
    let prepared = prepare_interchange_new_document_v1(
        cdxml_descriptor(),
        &source,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        cdxml_provenance(),
    )
    .expect("presentation-bearing CDXML preparation");
    let (session, receipt) = prepared
        .commit_and_take_session()
        .expect("clean presentation-bearing commit");

    assert_eq!((receipt.imported_record_count, receipt.bond_count), (1, 3));
    let snapshot = session.snapshot().expect("committed snapshot");
    for token in ["s1", "b1", "d1"] {
        assert!(
            snapshot.cdml().contains(&format!("type=\"{token}\"")),
            "CDML must retain {token}",
        );
    }
    let rendered = session
        .observe_render_v2(snapshot.revision())
        .expect("exact committed render observation");
    assert!(rendered.resolved().suppression().is_none());
    assert!(
        rendered
            .resolved()
            .molecule_plans()
            .iter()
            .all(|plan| { plan.issues().is_empty() && plan.member_issues().is_empty() })
    );
}

#[test]
fn clean_render_publication_gate_admits_cml_and_sdf_through_the_same_transaction() {
    let cml_descriptor = descriptor("cml");
    let cml_source = admit_interchange_source_v1(
        cml_descriptor,
        InterchangeSourceInputV1::RequestText(SINGLE_ATOM_CML.as_bytes()),
    )
    .expect("admitted CML source");
    let cml = prepare_interchange_new_document_v1(
        cml_descriptor,
        &cml_source,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        provenance("cml"),
    )
    .expect("CML candidate prepares")
    .commit_and_take_session()
    .expect("CML candidate publishes only after a clean render");
    assert_eq!(
        (cml.1.format_id.as_str(), cml.1.imported_record_count),
        (cml_descriptor.format_id(), 1)
    );

    let sdf_descriptor = descriptor("sdf");
    let sdf_source = admit_interchange_source_v1(
        sdf_descriptor,
        InterchangeSourceInputV1::RequestText(SINGLE_ATOM_SDF.as_bytes()),
    )
    .expect("admitted SDF source");
    let sdf = prepare_interchange_new_document_v1(
        sdf_descriptor,
        &sdf_source,
        &InjectedSdfRuntime(InjectedSdfEngine),
        provenance("sdf"),
    )
    .expect("SDF candidate prepares")
    .commit_and_take_session()
    .expect("SDF candidate publishes only after a clean render");
    assert_eq!(
        (sdf.1.format_id.as_str(), sdf.1.imported_record_count),
        (sdf_descriptor.format_id(), 1)
    );
}

#[test]
fn later_invalid_cdxml_fragment_refuses_the_whole_import_before_publication() {
    let source = admit_interchange_source_v1(
        cdxml_descriptor(),
        InterchangeSourceInputV1::RequestText(VALID_THEN_INVALID_PRESENTATION_CDXML.as_bytes()),
    )
    .expect("bounded CDXML source");
    let result = prepare_interchange_new_document_v1(
        cdxml_descriptor(),
        &source,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        cdxml_provenance(),
    );

    let Err(refusal) = result else {
        panic!("a later invalid fragment cannot publish a partial document");
    };
    assert_eq!(
        refusal,
        InterchangeImportRefusalV1::for_reason(InterchangeImportRefusalReasonV1::InvalidScalar),
    );
}

#[test]
fn cdxml_refusal_is_mapped_and_redacted_before_preparation() {
    let source = b"<CDXML><page><fragment id=\"source-fragment\"><n id=\"source-atom\" p=\"0 0\" Radical=\"1\"/></fragment></page></CDXML>";
    let admitted = admit_interchange_source_v1(
        cdxml_descriptor(),
        InterchangeSourceInputV1::RequestText(source),
    )
    .expect("bounded source admission");
    let result = prepare_interchange_new_document_v1(
        cdxml_descriptor(),
        &admitted,
        &crate::protocol::runtime::NoChemistryRuntimeV1,
        cdxml_provenance(),
    );
    let Err(refusal) = result else {
        panic!("unsupported CDXML attribute must refuse before a commit");
    };

    assert_eq!(
        refusal,
        InterchangeImportRefusalV1::for_reason(
            InterchangeImportRefusalReasonV1::AttributeUnsupported
        )
    );
    assert!(
        !serde_json::to_string(&refusal)
            .expect("redacted refusal JSON")
            .contains("source-fragment")
    );
}
