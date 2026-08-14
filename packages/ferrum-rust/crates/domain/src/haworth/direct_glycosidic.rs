//! Validation for the explicit two-ring, exterior-oxygen glycosidic profile.

use std::collections::{BTreeMap, BTreeSet};

use ferrum_core::{BondOrder, Molecule, RecordId, RecordKind, VertexRef};

use crate::haworth::{HaworthError, HaworthTopology, HaworthTopologyBuilder};

type GraphSourceOrders = BTreeMap<RecordId, usize>;

/// One ring and its proven exterior-oxygen attachment in a direct glycosidic profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicRingV1 {
    topology: HaworthTopology,
    attachment_atom: RecordId,
    attachment_bond: RecordId,
}

impl DirectGlycosidicRingV1 {
    /// Return the immutable, canonical topology of this ring.
    #[must_use]
    pub const fn topology(&self) -> &HaworthTopology {
        &self.topology
    }

    /// Return the selected ring atom joined to the exterior oxygen.
    #[must_use]
    pub const fn attachment_atom(&self) -> &RecordId {
        &self.attachment_atom
    }

    /// Return the selected single, non-aromatic bridge bond for this ring.
    #[must_use]
    pub const fn attachment_bond(&self) -> &RecordId {
        &self.attachment_bond
    }
}

/// The validated exterior oxygen and its two canonical bond identities.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicBridgeV1 {
    atom: RecordId,
    bonds: [RecordId; 2],
}

impl DirectGlycosidicBridgeV1 {
    /// Return the exterior oxygen identity.
    #[must_use]
    pub const fn atom(&self) -> &RecordId {
        &self.atom
    }

    /// Return the two bridge bonds in graph source order.
    #[must_use]
    pub const fn bonds(&self) -> &[RecordId; 2] {
        &self.bonds
    }
}

/// Immutable, graph-validated direct glycosidic Haworth topology.
///
/// This profile is exactly two vertex-disjoint C/O Haworth rings connected by
/// one exterior, degree-two oxygen through two explicitly selected single,
/// non-aromatic bonds. It records topology only: it does not choose geometry,
/// stereochemistry, labels, document placement, or mutation behavior.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DirectGlycosidicHaworthTopologyV1 {
    rings: [DirectGlycosidicRingV1; 2],
    bridge: DirectGlycosidicBridgeV1,
    atom_source_orders: GraphSourceOrders,
    bond_source_orders: GraphSourceOrders,
}

impl DirectGlycosidicHaworthTopologyV1 {
    /// Classify and canonicalize one explicitly selected direct glycosidic profile.
    ///
    /// The molecule is borrowed only while validation runs. The returned value
    /// owns every selected fact, so it remains an immutable receipt for that
    /// exact graph snapshot rather than a live view of document state.
    pub fn classify(
        molecule: &Molecule,
        rings: [HaworthTopology; 2],
        bridge_atom: RecordId,
        bridge_bonds: [RecordId; 2],
    ) -> Result<Self, HaworthError> {
        let first = validate_ring_snapshot(molecule, rings[0].clone())?;
        let second = validate_ring_snapshot(molecule, rings[1].clone())?;
        ensure_vertex_disjoint(&first, &second)?;

        if bridge_atom.kind() != RecordKind::Atom {
            return Err(HaworthError::InvalidSpec("bridge atom must be an atom"));
        }
        if [first.vertices(), second.vertices()]
            .into_iter()
            .flatten()
            .any(|vertex| vertex.atom == bridge_atom)
        {
            return Err(HaworthError::UnsupportedTopology(
                "bridge oxygen must be exterior to selected rings",
            ));
        }
        if bridge_bonds
            .iter()
            .any(|bond| bond.kind() != RecordKind::Bond)
        {
            return Err(HaworthError::InvalidSpec("bridge bonds must be bonds"));
        }
        if bridge_bonds[0] == bridge_bonds[1] {
            return Err(HaworthError::InvalidSpec("bridge bonds must be distinct"));
        }
        let bridge_atom_record = molecule
            .atoms()
            .iter()
            .find(|atom| atom.identity() == &bridge_atom)
            .ok_or(HaworthError::StaleTopology("bridge atom is absent"))?;
        if bridge_atom_record.element() != Some("O") {
            return Err(HaworthError::UnsupportedTopology(
                "bridge atom must be an exterior oxygen",
            ));
        }

        let incident: Vec<_> = molecule
            .bonds()
            .iter()
            .filter(|bond| {
                has_atom_endpoint(bond.start(), &bridge_atom)
                    || has_atom_endpoint(bond.end(), &bridge_atom)
            })
            .collect();
        if incident.len() != 2 {
            return Err(HaworthError::UnsupportedTopology(
                "bridge oxygen must have degree two",
            ));
        }
        let selected: BTreeSet<_> = bridge_bonds.iter().collect();
        if incident
            .iter()
            .any(|bond| !selected.contains(bond.identity()))
        {
            return Err(HaworthError::UnsupportedTopology(
                "bridge oxygen has an unselected attachment",
            ));
        }

        let attachments = [
            bridge_attachment(molecule, &bridge_atom, &bridge_bonds[0], &first, &second)?,
            bridge_attachment(molecule, &bridge_atom, &bridge_bonds[1], &first, &second)?,
        ];
        if attachments[0].0 == attachments[1].0 {
            return Err(HaworthError::UnsupportedTopology(
                "bridge bonds must attach to different rings",
            ));
        }

        let mut ring_facts = [
            DirectGlycosidicRingV1 {
                topology: first,
                attachment_atom: bridge_atom.clone(),
                attachment_bond: bridge_bonds[0].clone(),
            },
            DirectGlycosidicRingV1 {
                topology: second,
                attachment_atom: bridge_atom.clone(),
                attachment_bond: bridge_bonds[1].clone(),
            },
        ];
        for (ring_index, attachment_atom, attachment_bond) in attachments {
            ring_facts[ring_index].attachment_atom = attachment_atom;
            ring_facts[ring_index].attachment_bond = attachment_bond;
        }
        if ring_sort_key(&ring_facts[1]) < ring_sort_key(&ring_facts[0]) {
            ring_facts.swap(0, 1);
        }
        let mut canonical_bridge_bonds = bridge_bonds;
        canonical_bridge_bonds.sort_by_key(|bond_id| source_order(molecule.bonds(), bond_id));

        let (atom_source_orders, bond_source_orders) =
            source_orders(molecule, &ring_facts, &bridge_atom, &canonical_bridge_bonds)?;
        Ok(Self {
            rings: ring_facts,
            bridge: DirectGlycosidicBridgeV1 {
                atom: bridge_atom,
                bonds: canonical_bridge_bonds,
            },
            atom_source_orders,
            bond_source_orders,
        })
    }

    /// Return the two canonical ring facts, independent of caller order.
    #[must_use]
    pub const fn rings(&self) -> &[DirectGlycosidicRingV1; 2] {
        &self.rings
    }

    /// Return the validated exterior-oxygen bridge facts.
    #[must_use]
    pub const fn bridge(&self) -> &DirectGlycosidicBridgeV1 {
        &self.bridge
    }

    /// Return source positions for all selected atom identities in this snapshot.
    #[must_use]
    pub const fn atom_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.atom_source_orders
    }

    /// Return source positions for all selected bond identities in this snapshot.
    #[must_use]
    pub const fn bond_source_orders(&self) -> &BTreeMap<RecordId, usize> {
        &self.bond_source_orders
    }
}

fn validate_ring_snapshot(
    molecule: &Molecule,
    topology: HaworthTopology,
) -> Result<HaworthTopology, HaworthError> {
    let anomeric_atom = topology
        .vertices()
        .last()
        .ok_or(HaworthError::StaleTopology("selected ring has no vertices"))?
        .atom
        .clone();
    let revalidated = HaworthTopologyBuilder::new(
        topology.ring_form(),
        anomeric_atom,
        topology.vertices().to_vec(),
    )
    .build(molecule)
    .map_err(|_| HaworthError::StaleTopology("selected ring does not match molecule snapshot"))?;
    if revalidated != topology {
        return Err(HaworthError::StaleTopology(
            "selected ring does not match molecule snapshot",
        ));
    }
    Ok(topology)
}

fn ensure_vertex_disjoint(
    first: &HaworthTopology,
    second: &HaworthTopology,
) -> Result<(), HaworthError> {
    let first_vertices: BTreeSet<_> = first.vertices().iter().map(|vertex| &vertex.atom).collect();
    if second
        .vertices()
        .iter()
        .any(|vertex| first_vertices.contains(&vertex.atom))
    {
        return Err(HaworthError::UnsupportedTopology(
            "direct glycosidic rings must be vertex-disjoint",
        ));
    }
    Ok(())
}

fn bridge_attachment(
    molecule: &Molecule,
    bridge_atom: &RecordId,
    bond_id: &RecordId,
    first: &HaworthTopology,
    second: &HaworthTopology,
) -> Result<(usize, RecordId, RecordId), HaworthError> {
    let bond = molecule
        .bonds()
        .iter()
        .find(|bond| bond.identity() == bond_id)
        .ok_or(HaworthError::StaleTopology(
            "selected bridge bond is absent",
        ))?;
    if bond.order() != Some(BondOrder::Single) || bond.aromatic() != Some(false) {
        return Err(HaworthError::UnsupportedTopology(
            "bridge bonds must be non-aromatic single bonds",
        ));
    }
    let attachment = other_atom_endpoint(bond.start(), bond.end(), bridge_atom).ok_or(
        HaworthError::StaleTopology("selected bridge bond does not meet bridge oxygen"),
    )?;
    let ring_index = [first, second]
        .iter()
        .position(|ring| {
            ring.vertices()
                .iter()
                .any(|vertex| vertex.atom == attachment)
        })
        .ok_or(HaworthError::UnsupportedTopology(
            "bridge bonds must attach to selected ring atoms",
        ))?;
    let attachment_record = molecule
        .atoms()
        .iter()
        .find(|atom| atom.identity() == &attachment)
        .ok_or(HaworthError::StaleTopology(
            "selected bridge attachment atom is absent",
        ))?;
    if attachment_record.element() != Some("C") {
        return Err(HaworthError::UnsupportedTopology(
            "bridge bonds must attach to selected ring carbons",
        ));
    }
    Ok((ring_index, attachment, bond_id.clone()))
}

fn has_atom_endpoint(endpoint: &VertexRef, atom: &RecordId) -> bool {
    endpoint == &VertexRef::Atom(atom.clone())
}

fn other_atom_endpoint(
    start: &VertexRef,
    end: &VertexRef,
    bridge_atom: &RecordId,
) -> Option<RecordId> {
    match (start, end) {
        (VertexRef::Atom(start), VertexRef::Atom(end)) if start == bridge_atom => Some(end.clone()),
        (VertexRef::Atom(start), VertexRef::Atom(end)) if end == bridge_atom => Some(start.clone()),
        _ => None,
    }
}

fn ring_sort_key(ring: &DirectGlycosidicRingV1) -> (Vec<RecordId>, Vec<RecordId>) {
    (
        ring.topology
            .vertices()
            .iter()
            .map(|vertex| vertex.atom.clone())
            .collect(),
        ring.topology.bond_ids().to_vec(),
    )
}

fn source_orders(
    molecule: &Molecule,
    rings: &[DirectGlycosidicRingV1; 2],
    bridge_atom: &RecordId,
    bridge_bonds: &[RecordId; 2],
) -> Result<(GraphSourceOrders, GraphSourceOrders), HaworthError> {
    let selected_atoms: BTreeSet<_> = rings
        .iter()
        .flat_map(|ring| {
            ring.topology
                .vertices()
                .iter()
                .map(|vertex| vertex.atom.clone())
        })
        .chain(std::iter::once(bridge_atom.clone()))
        .collect();
    let selected_bonds: BTreeSet<_> = rings
        .iter()
        .flat_map(|ring| ring.topology.bond_ids().iter().cloned())
        .chain(bridge_bonds.iter().cloned())
        .collect();
    let atom_source_orders: BTreeMap<_, _> = molecule
        .atoms()
        .iter()
        .enumerate()
        .filter(|(_, atom)| selected_atoms.contains(atom.identity()))
        .map(|(order, atom)| (atom.identity().clone(), order))
        .collect();
    let bond_source_orders: BTreeMap<_, _> = molecule
        .bonds()
        .iter()
        .enumerate()
        .filter(|(_, bond)| selected_bonds.contains(bond.identity()))
        .map(|(order, bond)| (bond.identity().clone(), order))
        .collect();
    if atom_source_orders.len() != selected_atoms.len()
        || bond_source_orders.len() != selected_bonds.len()
    {
        return Err(HaworthError::StaleTopology(
            "selected topology records are absent from molecule snapshot",
        ));
    }
    Ok((atom_source_orders, bond_source_orders))
}

fn source_order(records: &[ferrum_core::Bond], record_id: &RecordId) -> usize {
    records
        .iter()
        .position(|record| record.identity() == record_id)
        .expect("validated bridge bond must remain in molecule")
}
