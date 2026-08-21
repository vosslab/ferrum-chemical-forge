//! Public-surface regression for molecule-report graph authority.

#[test]
fn document_public_surface_does_not_export_report_graph_execution_authority() {
    let chemistry_surface = include_str!("../src/chemistry/mod.rs");
    for forbidden in [
        "PreparedDocumentMoleculeReportV1",
        "execute_prepared_document_molecule_report_v1",
        "prepare_document_molecule_report_v1",
        "document_molecule_report_v1",
    ] {
        assert!(
            !chemistry_surface.contains(forbidden),
            "document must not export report graph authority: {forbidden}"
        );
    }
}
