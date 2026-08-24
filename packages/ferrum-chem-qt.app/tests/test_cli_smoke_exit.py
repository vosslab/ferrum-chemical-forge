"""Focused command-line validation for the Qt bundle smoke switch."""

# Standard Library
import sys
import pathlib

# PIP3 modules
import pytest

# local repo modules
import ferrum_qt.cli


#============================================
def test_cli_accepts_a_positive_smoke_exit_duration(monkeypatch: pytest.MonkeyPatch) -> None:
	"""A controlled bundle smoke can request ordinary timer-backed application exit."""
	monkeypatch.setattr(sys, "argv", ["ferrum-qt", "--smoke-exit", "2"])
	args = ferrum_qt.cli.parse_args()

	assert args.smoke_exit == 2.0


#============================================
def test_cli_help_advertises_the_documented_public_command(
		monkeypatch: pytest.MonkeyPatch, capsys: pytest.CaptureFixture[str],
		) -> None:
	"""Module launch help retains the public Ferrum command identity."""
	monkeypatch.setattr(sys, "argv", ["__main__.py", "--help"])

	with pytest.raises(SystemExit):
		ferrum_qt.cli.parse_args()

	assert "usage: ferrum-qt " in capsys.readouterr().out


#============================================
def test_cli_accepts_a_receipt_only_with_a_positive_smoke_timer(
		monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""A completion receipt is an explicit companion to normal timer exit."""
	receipt_path = tmp_path / "completion.json"
	monkeypatch.setattr(
		sys, "argv", ["ferrum-qt", "--smoke-exit", "2", "--smoke-receipt", str(receipt_path)],
	)
	args = ferrum_qt.cli.parse_args()

	assert args.smoke_receipt == str(receipt_path)


#============================================
def test_cli_rejects_a_receipt_without_a_smoke_timer(
		monkeypatch: pytest.MonkeyPatch, tmp_path: pathlib.Path,
		) -> None:
	"""A receipt cannot claim lifecycle completion for ordinary app startup."""
	monkeypatch.setattr(sys, "argv", ["ferrum-qt", "--smoke-receipt", str(tmp_path / "completion.json")])

	with pytest.raises(SystemExit):
		ferrum_qt.cli.parse_args()


#============================================
@pytest.mark.parametrize("duration", ("0", "-1", "nan", "inf"))
def test_cli_rejects_nonpositive_or_nonfinite_smoke_duration(
		monkeypatch: pytest.MonkeyPatch, duration: str,
		) -> None:
	"""The public timer-exit switch rejects values that cannot complete normally."""
	monkeypatch.setattr(sys, "argv", ["ferrum-qt", "--smoke-exit", duration])

	with pytest.raises(SystemExit):
		ferrum_qt.cli.parse_args()
