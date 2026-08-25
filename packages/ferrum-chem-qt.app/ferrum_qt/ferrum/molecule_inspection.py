"""Durable molecule addresses for the current document selection."""

# Standard Library
import dataclasses


#============================================
@dataclasses.dataclass(frozen=True, slots=True)
class FerrumNativeMoleculeInspectionAddress:
	"""One direct molecule root issued by the current render observation."""

	molecule_id: str


#============================================
def selected_durable_molecule_addresses(
		tab: object) -> tuple[FerrumNativeMoleculeInspectionAddress, ...] | None:
	"""Resolve selected structural targets through their durable owner IDs."""
	if getattr(tab, "requires_refresh", True):
		return None
	targets = tab.selected_molecule_information_targets()
	if type(targets) is not tuple or not targets:
		return None
	addresses = {}
	for target in targets:
		if (
			target.kind not in ("atom", "bond")
			or type(target.durable_object_id) is not str
			or not target.durable_object_id
			or type(target.durable_molecule_object_id) is not str
			or not target.durable_molecule_object_id
		):
			return None
		address = FerrumNativeMoleculeInspectionAddress(
			target.durable_molecule_object_id,
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
