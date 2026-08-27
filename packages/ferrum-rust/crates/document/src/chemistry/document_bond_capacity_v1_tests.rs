use crate::{DocumentSession, SessionDocumentObservationV1};

use super::{
    DocumentBondCapacityOutcomeV1, DocumentBondCapacityRequestV1, inspect_document_bond_capacity_v1,
};

fn observation(source: &str) -> SessionDocumentObservationV1 {
    DocumentSession::load(source)
        .expect("inline source loads")
        .observe(0)
        .expect("inline source projects")
}

fn request(
    observation: &SessionDocumentObservationV1,
    indices: &[usize],
) -> DocumentBondCapacityRequestV1 {
    let roots = indices
        .iter()
        .map(|index| {
            observation.projection().molecules()[*index]
                .document_object_id()
                .clone()
        })
        .collect();
    DocumentBondCapacityRequestV1::new(
        observation.snapshot().revision(),
        *observation.snapshot().digest(),
        roots,
    )
    .expect("nonduplicated selection")
}

#[test]
fn mixed_receipt_is_document_ordered_and_does_not_change_the_observation() {
    let source = r#"
<cdml xmlns="urn:ferrum:cdml" version="26.08">
 <molecule id="within">
  <atom id="c1" name="C" explicit_hydrogens="4"><point x="0" y="0"/></atom>
 </molecule>
 <molecule id="exceeds">
  <atom id="c2" name="C" explicit_hydrogens="4"><point x="2" y="0"/></atom>
  <atom id="o2" name="O"><point x="3" y="0"/></atom>
  <bond id="b2" start="c2" end="o2" type="n1"/>
 </molecule>
</cdml>
"#;
    let observation = observation(source);
    let before = observation.clone();

    let receipt = inspect_document_bond_capacity_v1(&observation, &request(&observation, &[1, 0]))
        .expect("in-profile roots evaluate");

    assert_eq!(
        receipt
            .records()
            .iter()
            .map(|record| record.source().source_id())
            .collect::<Vec<_>>(),
        vec!["within", "exceeds"]
    );
    assert!(matches!(
        receipt.records()[1].outcome(),
        DocumentBondCapacityOutcomeV1::ExceedsCapacity { atoms }
            if atoms[0].source_id.as_deref() == Some("c2")
    ));
    assert_eq!(observation, before);
}

#[test]
fn excluded_complete_root_returns_no_partial_atom_receipt() {
    let source = r#"
<cdml xmlns="urn:ferrum:cdml" version="26.08"><molecule id="unsupported">
 <atom id="c1" name="C" multiplicity="2"><point x="0" y="0"/></atom>
</molecule></cdml>
"#;
    let observation = observation(source);

    let receipt = inspect_document_bond_capacity_v1(&observation, &request(&observation, &[0]))
        .expect("unsupported profile is a successful receipt");

    assert!(matches!(
        receipt.records()[0].outcome(),
        DocumentBondCapacityOutcomeV1::NotChecked { .. }
    ));
}
