"""Installed-wheel checks for explicit, no-default CDML/CD-SVG admission."""

from __future__ import annotations

from pathlib import Path
import sys

import pytest

import ferrum_chem


CDML = b'<cdml version="1.0"/>'
CDSVG = (
    b'<svg xmlns="http://www.w3.org/2000/svg">'
    b'<cdml xmlns="http://www.freesoftware.fsf.org/bkchem/cdml" version="1.0"/>'
    b'</svg>'
)


def budget(byte_limit: int = 10_000) -> ferrum_chem.XmlInputBudgetV1:
    """A test-only complete policy; production callers must select their own limits."""
    return ferrum_chem.XmlInputBudgetV1(byte_limit, 100, 20, 100, 1_000)


def assert_input_error_shape(
    error: ferrum_chem.DocumentInputError,
    *,
    origin: str,
    stage: str,
    limit: int | None,
    actual: int | None,
    observed_at_least: int | None,
) -> None:
    """Every typed ingress rejection exposes the same inspectable attribute shape."""
    assert error.origin == origin
    assert error.stage == stage
    assert error.limit == limit
    assert error.actual == actual
    assert error.observed_at_least == observed_at_least


def test_budget_is_frozen_exact_and_extension_owned() -> None:
    value = budget()

    assert value.__class__.__module__ == "ferrum_chem"
    assert value.max_utf8_bytes == 10_000
    with pytest.raises(AttributeError):
        value.max_depth = 1
    for invalid in (True, -1, 2**200):
        with pytest.raises(TypeError):
            ferrum_chem.XmlInputBudgetV1(invalid, 1, 1, 1, 1)


def test_cdml_bytes_are_exact_and_ordinary_load_stays_available() -> None:
    session = ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(CDML, budget())
    assert session.snapshot().revision == 0
    assert ferrum_chem.DocumentSession.load(CDML.decode("ascii")).snapshot().revision == 0
    assert "unbounded compatibility" in ferrum_chem.DocumentSession.load.__doc__

    class BytesSubclass(bytes):
        pass

    for invalid in (bytearray(CDML), memoryview(CDML), BytesSubclass(CDML), CDML.decode("ascii")):
        with pytest.raises(TypeError):
            ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(invalid, budget())
    with pytest.raises(TypeError):
        ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(CDML, object())


def test_over_budget_utf8_and_dtd_fail_as_typed_input_without_session() -> None:
    with pytest.raises(ferrum_chem.DocumentInputError) as over_budget:
        ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(CDML, budget(len(CDML) - 1))
    assert_input_error_shape(
        over_budget.value,
        origin="bytes",
        stage="bytes",
        limit=len(CDML) - 1,
        actual=None,
        observed_at_least=len(CDML),
    )
    assert CDML.decode("ascii") not in str(over_budget.value)

    with pytest.raises(ferrum_chem.DocumentInputError) as invalid_utf8:
        ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(b"\xff", budget(1))
    assert_input_error_shape(
        invalid_utf8.value,
        origin="bytes",
        stage="utf8",
        limit=None,
        actual=None,
        observed_at_least=None,
    )

    dtd = b'<!DOCTYPE cdml><cdml version="1.0"/>'
    with pytest.raises(ferrum_chem.DocumentInputError) as dtd_error:
        ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(dtd, budget(len(dtd)))
    assert_input_error_shape(
        dtd_error.value,
        origin="bytes",
        stage="cdml",
        limit=None,
        actual=None,
        observed_at_least=None,
    )


def test_cdsvg_wrapper_and_payload_budgets_remain_independent() -> None:
    session = ferrum_chem.DocumentSession.load_cdsvg_utf8_bytes_with_budget(
        CDSVG,
        budget(len(CDSVG)),
        budget(10_000),
    )
    assert session.snapshot().revision == 0

    with pytest.raises(ferrum_chem.DocumentInputError) as wrapper_error:
        ferrum_chem.DocumentSession.load_cdsvg_utf8_bytes_with_budget(
            CDSVG,
            budget(len(CDSVG) - 1),
            budget(10_000),
        )
    assert_input_error_shape(
        wrapper_error.value,
        origin="bytes",
        stage="bytes",
        limit=len(CDSVG) - 1,
        actual=None,
        observed_at_least=len(CDSVG),
    )

    with pytest.raises(ferrum_chem.DocumentInputError) as payload_error:
        ferrum_chem.DocumentSession.load_cdsvg_utf8_bytes_with_budget(
            CDSVG,
            budget(len(CDSVG)),
            budget(1),
        )
    assert payload_error.value.origin == "bytes"
    assert payload_error.value.stage == "cdsvg_payload"
    assert payload_error.value.limit == 1
    assert isinstance(payload_error.value.actual, int)
    assert payload_error.value.actual > payload_error.value.limit
    assert payload_error.value.observed_at_least is None


def test_file_admission_accepts_exact_string_and_rejects_directory_and_symlink(
    tmp_path: Path,
) -> None:
    source = tmp_path / "document.cdml"
    source.write_bytes(CDML)
    assert ferrum_chem.DocumentSession.load_file_with_budget(
        str(source), budget(),
    ).snapshot().revision == 0

    with pytest.raises(TypeError):
        ferrum_chem.DocumentSession.load_file_with_budget(source, budget())
    with pytest.raises(ferrum_chem.DocumentInputError) as directory_error:
        ferrum_chem.DocumentSession.load_file_with_budget(str(tmp_path), budget())
    assert_input_error_shape(
        directory_error.value,
        origin="file",
        stage="source_policy",
        limit=None,
        actual=None,
        observed_at_least=None,
    )

    link = tmp_path / "document-link.cdml"
    link.symlink_to(source)
    with pytest.raises(ferrum_chem.DocumentInputError) as link_error:
        ferrum_chem.DocumentSession.load_file_with_budget(str(link), budget())
    assert_input_error_shape(
        link_error.value,
        origin="file",
        stage="source_policy",
        limit=None,
        actual=None,
        observed_at_least=None,
    )


def test_local_profile_prepares_one_worker_safe_session(
		tmp_path: Path,
		) -> None:
	"""The named profile owns admission before one UI-thread session handoff."""
	source = tmp_path / "product-open.cdml"
	source.write_bytes(CDML)
	prepared = ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(source))
	session, observation = prepared.take_admission_v1()
	assert session.snapshot().revision == 0 and not session.snapshot().is_dirty
	assert observation.document.snapshot.digest == session.snapshot().digest
	with pytest.raises(ferrum_chem.PreparedOperationConsumedError):
		prepared.take_admission_v1()


def test_local_profile_rejects_symlink_before_preparing_a_session(tmp_path: Path) -> None:
	"""The named profile preserves the ordinary local-file source policy."""
	source = tmp_path / "product-open.cdml"
	source.write_bytes(CDML)
	link = tmp_path / "product-open-link.cdml"
	link.symlink_to(source)
	with pytest.raises(ferrum_chem.DocumentInputError) as link_error:
		ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(str(link))
	assert link_error.value.origin == "file" and link_error.value.stage == "source_policy"


def test_local_profile_maps_unpaired_surrogate_to_typed_path_error() -> None:
	"""Python text encoding failure cannot bypass the product input taxonomy."""
	invalid_path = "\ud800"
	with pytest.raises(ferrum_chem.DocumentInputError) as path_error:
		ferrum_chem.DocumentSession.prepare_local_cdml_file_v1(invalid_path)
	assert path_error.value.origin == "file" and path_error.value.stage == "path"


def test_maximum_usize_budget_is_a_typed_source_policy_failure() -> None:
    # This is not a deployment limit. It exercises the impossible sentinel configuration.
    maximum = (sys.maxsize * 2) + 1
    with pytest.raises(ferrum_chem.DocumentInputError) as failure:
        ferrum_chem.DocumentSession.load_utf8_bytes_with_budget(CDML, budget(maximum))
    assert_input_error_shape(
        failure.value,
        origin="bytes",
        stage="source_policy",
        limit=maximum,
        actual=None,
        observed_at_least=None,
    )
