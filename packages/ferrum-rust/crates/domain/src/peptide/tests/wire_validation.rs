use crate::peptide::PeptideSequence;

#[test]
fn deserialization_rejects_an_empty_sequence() {
    let error = serde_json::from_str::<PeptideSequence>(r#"{"residues":[]}"#)
        .expect_err("serialized empty residues must not bypass the sequence invariant");
    assert!(
        error
            .to_string()
            .contains("must contain at least one residue")
    );
}
