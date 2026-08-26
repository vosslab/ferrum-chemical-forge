"""Durable molecule addresses for the current document selection."""

# Standard Library
import dataclasses

# local repo modules
import ferrum_qt.ferrum.document_tab_errors as document_tab_errors
import ferrum_qt.ferrum.engine as engine


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeInspectionAddress:
	"""One exact Rust-owned molecule identity selected through a structure target."""

	molecule_id: str


#============================================
def selected_durable_molecule_addresses(
		tab: object) -> tuple[FerrumNativeMoleculeInspectionAddress, ...] | None:
	"""Resolve selected structural targets through their durable owner IDs."""
	if getattr(tab, "requires_refresh", True):
		return None
	try:
		targets = tab.selected_structure_targets()
	except document_tab_errors.FerrumNativeDocumentTabError:
		return None
	if type(targets) is not tuple or not targets:
		return None
	addresses: dict[str, FerrumNativeMoleculeInspectionAddress] = {}
	object_ids: set[str] = set()
	for target in targets:
		if (
			target.kind not in (
				engine.StructureTargetKindV1.atom,
				engine.StructureTargetKindV1.bond,
			)
			or type(target.molecule_object_id) is not str
			or not target.molecule_object_id
			or type(target.object_id) is not str
			or not target.object_id
		):
			return None
		if target.object_id in object_ids:
			return None
		object_ids.add(target.object_id)
		address = FerrumNativeMoleculeInspectionAddress(
			target.molecule_object_id,
		)
		previous = addresses.setdefault(address.molecule_id, address)
		if previous != address:
			return None
	return tuple(addresses.values())


#============================================
def selected_durable_molecule_address(
		tab: object) -> FerrumNativeMoleculeInspectionAddress | None:
	"""Return the sole root only when the complete selection maps to one root."""
	addresses = selected_durable_molecule_addresses(tab)
	return None if addresses is None or len(addresses) != 1 else addresses[0]
