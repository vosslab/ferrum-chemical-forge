"""Validated PubChem SMILES used by the Ferrum GUI documentation tour."""


# Standard Library
import pathlib

# PIP3 modules
import yaml


_REPO_ROOT = pathlib.Path(__file__).resolve().parents[1]
_PUBCHEM_DATA_PATH = _REPO_ROOT / "data" / "pubchem_molecules_data.yml"
_REQUIRED_CIDS = (190, 1135, 5988)

# The user supplied this non-stereospecific CID 65146 source specifically for
# the documentation capture. It is intentionally distinct from CID 94190.
DISTEAROYLPHOSPHATIDYLCHOLINE_SMILES = (
	"CCCCCCCCCCCCCCCCCC(=O)OCC(COP(=O)([O-])OCC[N+](C)(C)C)"
	"OC(=O)CCCCCCCCCCCCCCCCC"
)


#============================================
def _pubchem_smiles_by_cid() -> dict[int, str]:
	"""Load only the bounded production records used by documentation capture."""
	loaded = yaml.safe_load(_PUBCHEM_DATA_PATH.read_text(encoding="utf-8"))
	if type(loaded) is not dict:
		raise RuntimeError("PubChem molecule data must be a mapping")
	records = loaded["cid_to_data"]
	if type(records) is not dict:
		raise RuntimeError("PubChem cid_to_data must be a mapping")
	result: dict[int, str] = {}
	for cid in _REQUIRED_CIDS:
		record = records[cid]
		if type(record) is not dict or record["CID"] != cid:
			raise RuntimeError(f"PubChem CID {cid} record is malformed")
		smiles = record["SMILES"]
		if type(smiles) is not str or not smiles or len(smiles) > 4096:
			raise RuntimeError(f"PubChem CID {cid} SMILES is malformed")
		result[cid] = smiles
	return result


_PUBCHEM_SMILES_BY_CID = _pubchem_smiles_by_cid()
ADENINE_SMILES = _PUBCHEM_SMILES_BY_CID[190]
THYMINE_SMILES = _PUBCHEM_SMILES_BY_CID[1135]
SUCROSE_SMILES = _PUBCHEM_SMILES_BY_CID[5988]
