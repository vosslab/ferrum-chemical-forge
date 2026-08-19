#!/usr/bin/env python3
"""Read the frozen Python-RDKit layout-orientation evidence receipt.

The one-time reference measurement selected ``canonOrient=True`` before the
Ferrum layout route existed. Its isolated Python-RDKit worker was retired: this
tool deliberately reads the accepted receipt rather than presenting historical
measurement output as a current implementation or CI check.
"""

# Standard Library
import argparse
import json
import pathlib


REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
DEFAULT_REPORT = (
	REPO_ROOT / "docs" / "active_plans" / "reports" / "rdkit_layout_orientation.json"
)


#============================================
def _receipt(path: pathlib.Path) -> dict:
	"""Load the accepted reference receipt without interpreting it as a live run."""
	try:
		value = json.loads(path.read_text(encoding="utf-8"))
	except (OSError, json.JSONDecodeError) as error:
		raise RuntimeError("layout-orientation receipt is unreadable: " + str(path)) from error
	if not isinstance(value, dict) or value.get("capability") != "rdkit-layout-orientation-v1":
		raise RuntimeError("layout-orientation receipt has an unexpected schema")
	return value


#============================================
def main() -> None:
	"""Print a receipt-backed archival summary or the complete frozen receipt."""
	parser = argparse.ArgumentParser(description=__doc__)
	parser.add_argument("--report", default=DEFAULT_REPORT, type=pathlib.Path)
	parser.add_argument(
		"--show-receipt", action="store_true",
		help="print the complete frozen receipt instead of its archival summary",
	)
	arguments = parser.parse_args()
	receipt = _receipt(arguments.report)
	if arguments.show_receipt:
		print(json.dumps(receipt, indent=2, sort_keys=True))
		return
	print(json.dumps({
		"capability": receipt["capability"],
		"decision": receipt.get("decision"),
		"receipt": str(arguments.report.resolve()),
		"recorded_status": receipt.get("status"),
		"status": "archived-reference-evidence",
	}, sort_keys=True))


if __name__ == "__main__":
	main()
