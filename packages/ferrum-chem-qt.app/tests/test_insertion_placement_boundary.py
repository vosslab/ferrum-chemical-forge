"""Behavior coverage for Rust-owned molecule-insertion placement capture."""

# Standard Library
import math

# PIP3 modules
import ferrum_chem
import pytest

# local repo modules
import ferrum_qt.bridge.insertion_placement


#============================================
def test_capture_insertion_placement_returns_finite_plain_worker_values() -> None:
	"""The default detached capture is validated by the compiled Ferrum boundary."""
	bond_length, anchor = ferrum_qt.bridge.insertion_placement.capture_insertion_placement(
		object(),
	)

	assert bond_length > 0.0 and all(math.isfinite(value) for value in anchor)


#============================================
def test_geometry_public_types_belong_to_the_compiled_extension() -> None:
	"""The frozen V1 DTO and typed error retain direct extension provenance."""
	assert (
		ferrum_chem.GeometryError.__module__,
		ferrum_chem.InsertionPlacementV1.__module__,
	) == ("ferrum_chem", "ferrum_chem")


#============================================
def test_insertion_placement_rejects_nonfinite_anchor_at_the_rust_boundary() -> None:
	"""The public Ferrum DTO reports invalid placement with its exact error type."""
	with pytest.raises(ferrum_chem.GeometryError, match="insertion anchor"):
		ferrum_chem.validate_insertion_placement_v1(30.0, math.inf, 0.0)
