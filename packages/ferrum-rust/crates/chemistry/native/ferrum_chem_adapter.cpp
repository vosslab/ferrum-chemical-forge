#include "ferrum_chem_adapter.h"
#include "ferrum_chem_smarts_match.h"
#include "ferrum_chem_molecule_response.h"
#include "ferrum_chem_utf8.h"

#include <GraphMol/Atom.h>
#include <GraphMol/Bond.h>
#ifdef FERRUM_CHEM_ENABLE_DEPICTOR
#include <GraphMol/Chirality.h>
#include <GraphMol/Conformer.h>
#include <GraphMol/Depictor/RDDepictor.h>
#endif
#include <GraphMol/MolOps.h>
#include <GraphMol/RWMol.h>
#include <GraphMol/SanitException.h>
#include <GraphMol/SmilesParse/SmilesParse.h>

#include <algorithm>
#ifdef FERRUM_CHEM_ENABLE_DEPICTOR
#include <bit>
#include <cmath>
#endif
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <exception>
#include <iterator>
#include <limits>
#include <memory>
#include <new>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_set>
#include <vector>

namespace {

constexpr uint8_t kRequestMagic[] = {'F', 'C', 'K', '1'};
constexpr uint8_t kResponseMagic[] = {'F', 'C', 'R', '1'};
constexpr uint32_t kWireVersion = FERRUM_CHEM_KEKULIZE_WIRE_VERSION;
constexpr uint32_t kKnownOptionBits =
	FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS |
	FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL;
constexpr uint16_t kFormalChargePresent = FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE;
constexpr uint16_t kIsotopePresent = FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE;
constexpr uint16_t kExplicitHydrogensPresent =
	FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS;
constexpr uint16_t kKnownAtomPresenceBits = kFormalChargePresent | kIsotopePresent |
	kExplicitHydrogensPresent;
constexpr uint32_t kMaximumBacktracks = FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS;
constexpr uint32_t kMaximumAtoms = FERRUM_CHEM_KEKULIZE_MAX_ATOMS;
constexpr uint32_t kMaximumBonds = FERRUM_CHEM_KEKULIZE_MAX_BONDS;
constexpr uint32_t kMaximumDetailBytes = FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES;
constexpr uint64_t kRequestHeaderBytes = FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES;
constexpr uint64_t kResponseHeaderBytes = FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES;
constexpr uint64_t kAtomBytes = FERRUM_CHEM_KEKULIZE_ATOM_BYTES;
constexpr uint64_t kBondBytes = FERRUM_CHEM_KEKULIZE_BOND_BYTES;
constexpr uint64_t kMaximumSmilesBytes = FERRUM_CHEM_SMILES_MAX_BYTES;
constexpr uint64_t kMaximumResponseBytes = FERRUM_CHEM_MAX_RESPONSE_BYTES;

struct KekulizeOptions {
	bool clear_aromatic_flags;
	bool canonical;
	uint32_t max_backtracks;
};

struct WireAtom {
	uint8_t atomic_number;
	bool aromatic;
	uint16_t presence_flags;
	int32_t formal_charge;
	uint16_t isotope;
	uint16_t explicit_hydrogens;
};

struct WireBond {
	uint32_t begin_atom;
	uint32_t end_atom;
	RDKit::Bond::BondType type;
	bool aromatic;
};

struct WireMolecule {
	KekulizeOptions options;
	std::vector<WireAtom> atoms;
	std::vector<WireBond> bonds;
};

uint16_t read_u16(const uint8_t *bytes) {
	return static_cast<uint16_t>(bytes[0]) | (static_cast<uint16_t>(bytes[1]) << 8U);
}

uint32_t read_u32(const uint8_t *bytes) {
	return static_cast<uint32_t>(bytes[0]) |
		(static_cast<uint32_t>(bytes[1]) << 8U) |
		(static_cast<uint32_t>(bytes[2]) << 16U) |
		(static_cast<uint32_t>(bytes[3]) << 24U);
}

int32_t read_i32(const uint8_t *bytes) {
	return static_cast<int32_t>(read_u32(bytes));
}

void append_u16(std::vector<uint8_t> &bytes, uint16_t value) {
	bytes.push_back(static_cast<uint8_t>(value));
	bytes.push_back(static_cast<uint8_t>(value >> 8U));
}

void append_u32(std::vector<uint8_t> &bytes, uint32_t value) {
	bytes.push_back(static_cast<uint8_t>(value));
	bytes.push_back(static_cast<uint8_t>(value >> 8U));
	bytes.push_back(static_cast<uint8_t>(value >> 16U));
	bytes.push_back(static_cast<uint8_t>(value >> 24U));
}

void append_i32(std::vector<uint8_t> &bytes, int32_t value) {
	append_u32(bytes, static_cast<uint32_t>(value));
}

bool has_record_bytes(uint32_t count, uint64_t record_bytes, uint64_t remaining) {
	return static_cast<uint64_t>(count) <= remaining / record_bytes;
}

bool decode_bond_type(uint8_t wire_type, RDKit::Bond::BondType *bond_type) {
	switch (wire_type) {
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE:
		*bond_type = RDKit::Bond::SINGLE;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE:
		*bond_type = RDKit::Bond::DOUBLE;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE:
		*bond_type = RDKit::Bond::TRIPLE;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC:
		*bond_type = RDKit::Bond::AROMATIC;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED:
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE:
		return false;
	default:
		return false;
	}
}

uint8_t encode_bond_type(RDKit::Bond::BondType bond_type) {
	switch (bond_type) {
	case RDKit::Bond::SINGLE:
		return FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE;
	case RDKit::Bond::DOUBLE:
		return FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE;
	case RDKit::Bond::TRIPLE:
		return FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE;
	case RDKit::Bond::AROMATIC:
		return FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC;
	case RDKit::Bond::QUADRUPLE:
		return FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED;
	default:
		return FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED;
	}
}

bool parse_request(const uint8_t *request, uint64_t request_len, bool kekulization,
		WireMolecule *molecule, std::string *error) {
	if (request == nullptr || request_len < kRequestHeaderBytes) {
		*error = "request is missing or shorter than its header";
		return false;
	}
	if (std::memcmp(request, kRequestMagic, sizeof(kRequestMagic)) != 0) {
		*error = "request magic is not FCK1";
		return false;
	}
	if (read_u32(request + 4) != kWireVersion) {
		*error = "request wire version is unsupported";
		return false;
	}
	const uint32_t option_bits = read_u32(request + 8);
	const uint32_t max_backtracks = read_u32(request + 12);
	const uint32_t atom_count = read_u32(request + 16);
	const uint32_t bond_count = read_u32(request + 20);
	if (kekulization && ((option_bits & ~kKnownOptionBits) != 0 || max_backtracks == 0 ||
		max_backtracks > kMaximumBacktracks)) {
		*error = "Kekulize request has reserved options or exceeds the backtrack limit";
		return false;
	}
	if (!kekulization && (option_bits != 0 || max_backtracks != 1)) {
		*error = "depiction request has non-deterministic or Kekulize-only controls";
		return false;
	}
	if (atom_count > kMaximumAtoms || bond_count > kMaximumBonds) {
		*error = "request exceeds molecule size limits";
		return false;
	}
	const uint64_t payload_len = request_len - kRequestHeaderBytes;
	if (!has_record_bytes(atom_count, kAtomBytes, payload_len)) {
		*error = "request atom records are truncated";
		return false;
	}
	const uint64_t atom_len = static_cast<uint64_t>(atom_count) * kAtomBytes;
	if (!has_record_bytes(bond_count, kBondBytes, payload_len - atom_len) ||
		payload_len != atom_len + static_cast<uint64_t>(bond_count) * kBondBytes) {
		*error = "request bond records are truncated or have trailing bytes";
		return false;
	}
	molecule->options = {
		(option_bits & FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS) != 0,
		(option_bits & FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL) != 0,
		max_backtracks,
	};
	molecule->atoms.reserve(atom_count);
	molecule->bonds.reserve(bond_count);
	const uint8_t *cursor = request + kRequestHeaderBytes;
	for (uint32_t index = 0; index < atom_count; ++index, cursor += kAtomBytes) {
		const uint16_t presence_flags = read_u16(cursor + 2);
		const int32_t formal_charge = read_i32(cursor + 4);
		const uint16_t isotope = read_u16(cursor + 8);
		const uint16_t explicit_hydrogens = read_u16(cursor + 10);
		if (cursor[0] == 0 || cursor[0] > 118 || cursor[1] > 1 ||
			(presence_flags & ~kKnownAtomPresenceBits) != 0 ||
			((presence_flags & kFormalChargePresent) == 0 && formal_charge != 0) ||
			((presence_flags & kIsotopePresent) == 0 && isotope != 0) ||
			((presence_flags & kIsotopePresent) != 0 && isotope == 0) ||
			((presence_flags & kExplicitHydrogensPresent) == 0 && explicit_hydrogens != 0)) {
			*error = "request contains an invalid atom record";
			return false;
		}
		molecule->atoms.push_back({
			cursor[0], cursor[1] == 1, presence_flags, formal_charge, isotope, explicit_hydrogens,
		});
	}
	std::unordered_set<uint64_t> undirected_bonds;
	undirected_bonds.reserve(bond_count);
	for (uint32_t index = 0; index < bond_count; ++index, cursor += kBondBytes) {
		RDKit::Bond::BondType type;
		const uint32_t begin_atom = read_u32(cursor);
		const uint32_t end_atom = read_u32(cursor + 4);
		const bool aromatic = cursor[9] == 1;
		const uint32_t lower_atom = std::min(begin_atom, end_atom);
		const uint32_t higher_atom = std::max(begin_atom, end_atom);
		const uint64_t bond_key = (static_cast<uint64_t>(lower_atom) << 32U) | higher_atom;
		if (begin_atom >= atom_count || end_atom >= atom_count || begin_atom == end_atom ||
			!decode_bond_type(cursor[8], &type) || cursor[9] > 1 ||
			cursor[10] != 0 || cursor[11] != 0 ||
			(type == RDKit::Bond::AROMATIC && !aromatic) ||
			(aromatic && type != RDKit::Bond::AROMATIC) ||
			(aromatic && (!molecule->atoms[begin_atom].aromatic ||
				!molecule->atoms[end_atom].aromatic)) ||
			!undirected_bonds.insert(bond_key).second) {
			*error = "request contains an invalid bond record";
			return false;
		}
		molecule->bonds.push_back({begin_atom, end_atom, type, aromatic});
	}
	return true;
}

WireMolecule kekulize(const WireMolecule &input) {
	RDKit::RWMol molecule;
	for (const WireAtom &atom : input.atoms) {
		auto *rdkit_atom = new RDKit::Atom(atom.atomic_number);
		rdkit_atom->setIsAromatic(atom.aromatic);
		if ((atom.presence_flags & kFormalChargePresent) != 0) {
			rdkit_atom->setFormalCharge(atom.formal_charge);
		}
		if ((atom.presence_flags & kIsotopePresent) != 0) {
			rdkit_atom->setIsotope(atom.isotope);
		}
		if ((atom.presence_flags & kExplicitHydrogensPresent) != 0) {
			rdkit_atom->setNumExplicitHs(atom.explicit_hydrogens);
		}
		molecule.addAtom(rdkit_atom, true, true);
	}
	for (const WireBond &bond : input.bonds) {
		molecule.addBond(bond.begin_atom, bond.end_atom, bond.type);
		RDKit::Bond *rdkit_bond = molecule.getBondBetweenAtoms(bond.begin_atom, bond.end_atom);
		rdkit_bond->setIsAromatic(bond.aromatic);
	}
	RDKit::MolOps::Kekulize(
		molecule, input.options.clear_aromatic_flags, input.options.canonical,
		input.options.max_backtracks);

	WireMolecule output;
	output.options = input.options;
	output.atoms.reserve(molecule.getNumAtoms());
	output.bonds.reserve(molecule.getNumBonds());
	if (molecule.getNumBonds() != input.bonds.size()) {
		throw std::runtime_error("RDKit changed topology outside the Ferrum wire contract");
	}
	for (uint32_t index = 0; index < molecule.getNumAtoms(); ++index) {
		const RDKit::Atom *atom = molecule.getAtomWithIdx(index);
		const WireAtom &input_atom = input.atoms[index];
		output.atoms.push_back({
			input_atom.atomic_number, atom->getIsAromatic(), input_atom.presence_flags,
			input_atom.formal_charge, input_atom.isotope, input_atom.explicit_hydrogens,
		});
	}
	for (uint32_t index = 0; index < molecule.getNumBonds(); ++index) {
		const RDKit::Bond *bond = molecule.getBondWithIdx(index);
		const WireBond &input_bond = input.bonds[index];
		if (bond->getBeginAtomIdx() != input_bond.begin_atom ||
			bond->getEndAtomIdx() != input_bond.end_atom) {
			throw std::runtime_error("RDKit reordered topology outside the Ferrum wire contract");
		}
		if (encode_bond_type(bond->getBondType()) ==
			FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED) {
			throw std::runtime_error("RDKit produced a bond type outside the Ferrum wire contract");
		}
		output.bonds.push_back({
			bond->getBeginAtomIdx(), bond->getEndAtomIdx(), bond->getBondType(), bond->getIsAromatic(),
		});
	}
	return output;
}

bool encode_response(uint32_t status, std::string_view detail, const WireMolecule *molecule,
		ferrum_chem_owned_buffer *response) {
	try {
		const uint64_t detail_len = std::min(detail.size(), static_cast<size_t>(kMaximumDetailBytes));
		const uint64_t atom_count = molecule == nullptr ? 0 : molecule->atoms.size();
		const uint64_t bond_count = molecule == nullptr ? 0 : molecule->bonds.size();
		const uint32_t option_bits = molecule == nullptr ? 0 :
			((static_cast<uint32_t>(molecule->options.clear_aromatic_flags) *
				FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS) |
				(static_cast<uint32_t>(molecule->options.canonical) *
					FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL));
		const uint32_t max_backtracks = molecule == nullptr ? 0 : molecule->options.max_backtracks;
		if (atom_count > kMaximumAtoms || bond_count > kMaximumBonds ||
			detail_len > std::numeric_limits<uint64_t>::max() - kResponseHeaderBytes) {
			return false;
		}
		const uint64_t record_len = atom_count * kAtomBytes + bond_count * kBondBytes;
		if (record_len > std::numeric_limits<uint64_t>::max() - kResponseHeaderBytes - detail_len ||
			kResponseHeaderBytes + detail_len + record_len > std::numeric_limits<size_t>::max()) {
			return false;
		}
		const size_t response_len = static_cast<size_t>(kResponseHeaderBytes + detail_len + record_len);
		std::vector<uint8_t> bytes;
		bytes.reserve(response_len);
		bytes.insert(bytes.end(), std::begin(kResponseMagic), std::end(kResponseMagic));
		append_u32(bytes, kWireVersion);
		append_u32(bytes, status);
		append_u32(bytes, static_cast<uint32_t>(detail_len));
		append_u32(bytes, option_bits);
		append_u32(bytes, max_backtracks);
		append_u32(bytes, static_cast<uint32_t>(atom_count));
		append_u32(bytes, static_cast<uint32_t>(bond_count));
		bytes.insert(bytes.end(), detail.begin(), detail.begin() + detail_len);
		if (molecule != nullptr) {
			for (const WireAtom &atom : molecule->atoms) {
				bytes.push_back(atom.atomic_number);
				bytes.push_back(atom.aromatic ? 1 : 0);
				append_u16(bytes, atom.presence_flags);
				append_i32(bytes, atom.formal_charge);
				append_u16(bytes, atom.isotope);
				append_u16(bytes, atom.explicit_hydrogens);
			}
			for (const WireBond &bond : molecule->bonds) {
				append_u32(bytes, bond.begin_atom);
				append_u32(bytes, bond.end_atom);
				bytes.push_back(encode_bond_type(bond.type));
				bytes.push_back(bond.aromatic ? 1 : 0);
				append_u16(bytes, 0);
			}
		}
		if (bytes.size() != response_len || response_len > kMaximumResponseBytes) {
			return false;
		}
		response->data = new uint8_t[response_len];
		std::memcpy(response->data, bytes.data(), response_len);
		response->len = response_len;
		return true;
	} catch (...) {
		return false;
	}
}

#ifdef FERRUM_CHEM_ENABLE_DEPICTOR
bool encode_coordinate_response(uint32_t status, std::string_view detail,
		const WireMolecule *molecule, const RDKit::Conformer *conformer,
		ferrum_chem_owned_buffer *response) {
	try {
		const uint64_t detail_len = std::min(detail.size(), static_cast<size_t>(kMaximumDetailBytes));
		const uint64_t atom_count = conformer == nullptr ? 0 : molecule->atoms.size();
		if (atom_count > kMaximumAtoms ||
			detail_len > std::numeric_limits<uint64_t>::max() - 20U ||
			atom_count > (std::numeric_limits<uint64_t>::max() - 20U - detail_len) / 16U) {
			return false;
		}
		const uint64_t response_len = 20U + detail_len + atom_count * 16U;
		if (response_len > std::numeric_limits<size_t>::max()) {
			return false;
		}
		std::vector<uint8_t> bytes;
		bytes.reserve(static_cast<size_t>(response_len));
		bytes.insert(bytes.end(), {'F', 'C', 'L', '1'});
		append_u32(bytes, 1U);
		append_u32(bytes, status);
		append_u32(bytes, static_cast<uint32_t>(detail_len));
		append_u32(bytes, static_cast<uint32_t>(atom_count));
		bytes.insert(bytes.end(), detail.begin(), detail.begin() + detail_len);
		if (conformer != nullptr) {
			for (uint32_t index = 0; index < atom_count; ++index) {
				const RDGeom::Point3D &point = conformer->getAtomPos(index);
				const uint64_t x = std::bit_cast<uint64_t>(point.x);
				const uint64_t y = std::bit_cast<uint64_t>(point.y);
				for (unsigned int byte = 0; byte < 8; ++byte) {
					bytes.push_back(static_cast<uint8_t>(x >> (byte * 8U)));
				}
				for (unsigned int byte = 0; byte < 8; ++byte) {
					bytes.push_back(static_cast<uint8_t>(y >> (byte * 8U)));
				}
			}
		}
		if (bytes.size() != response_len || response_len > kMaximumResponseBytes) {
			return false;
		}
		response->data = new uint8_t[bytes.size()];
		std::memcpy(response->data, bytes.data(), bytes.size());
		response->len = bytes.size();
		return true;
	} catch (...) {
		return false;
	}
}

bool generate_2d(const WireMolecule &input, ferrum_chem_owned_buffer *response) {
	RDKit::RWMol molecule;
	for (const WireAtom &atom : input.atoms) {
		auto *rdkit_atom = new RDKit::Atom(atom.atomic_number);
		rdkit_atom->setIsAromatic(atom.aromatic);
		if ((atom.presence_flags & kFormalChargePresent) != 0) {
			rdkit_atom->setFormalCharge(atom.formal_charge);
		}
		if ((atom.presence_flags & kIsotopePresent) != 0) {
			rdkit_atom->setIsotope(atom.isotope);
		}
		if ((atom.presence_flags & kExplicitHydrogensPresent) != 0) {
			rdkit_atom->setNumExplicitHs(atom.explicit_hydrogens);
		}
		molecule.addAtom(rdkit_atom, true, true);
	}
	for (const WireBond &bond : input.bonds) {
		molecule.addBond(bond.begin_atom, bond.end_atom, bond.type);
		molecule.getBondBetweenAtoms(bond.begin_atom, bond.end_atom)->setIsAromatic(bond.aromatic);
	}
	RDDepict::Compute2DCoordParameters parameters;
	parameters.canonOrient = true;
	parameters.clearConfs = true;
	parameters.forceRDKit = true;
	parameters.nFlipsPerSample = 0;
	parameters.nSamples = 0;
	parameters.useRingTemplates = false;
	const unsigned int conformer_id = RDDepict::compute2DCoords(molecule, parameters);
	const RDKit::Conformer &conformer = molecule.getConformer(conformer_id);
	for (uint32_t index = 0; index < molecule.getNumAtoms(); ++index) {
		const RDGeom::Point3D &point = conformer.getAtomPos(index);
		if (!std::isfinite(point.x) || !std::isfinite(point.y) || !std::isfinite(point.z)) {
			throw std::runtime_error("RDKit generated a non-finite coordinate");
		}
	}
	return encode_coordinate_response(FERRUM_CHEM_RESULT_OK, "", &input, &conformer, response);
}
#endif

bool smiles_to_molecule(const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) {
	if (request == nullptr || request_len == 0U || request_len > kMaximumSmilesBytes)
		return ferrum_chem::emit_molecule_response(FERRUM_CHEM_RESULT_MALFORMED_REQUEST, "SMILES request must be non-empty UTF-8 text within the limit", nullptr, nullptr, response);
	const std::string smiles(reinterpret_cast<const char *>(request), static_cast<size_t>(request_len));
	if (smiles.find('\0') != std::string::npos || !ferrum_chem::is_valid_utf8(smiles))
		return ferrum_chem::emit_molecule_response(FERRUM_CHEM_RESULT_MALFORMED_REQUEST, "SMILES request must be UTF-8 without NUL bytes", nullptr, nullptr, response);
	std::unique_ptr<RDKit::ROMol> molecule(RDKit::SmilesToMol(smiles));
	if (!molecule) return ferrum_chem::emit_molecule_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE, "RDKit could not parse SMILES", nullptr, nullptr, response);
	RDDepict::Compute2DCoordParameters parameters; parameters.canonOrient = true; parameters.clearConfs = true; parameters.forceRDKit = true; parameters.nFlipsPerSample = 0; parameters.nSamples = 0; parameters.useRingTemplates = false;
	const unsigned int id = RDDepict::compute2DCoords(*molecule, parameters);
	RDKit::Chirality::wedgeMolBonds(*molecule, &molecule->getConformer(id));
	return ferrum_chem::emit_molecule_response(FERRUM_CHEM_RESULT_OK, "", molecule.get(), &molecule->getConformer(id), response);
}

uint32_t emit_error(uint32_t result_status, std::string_view detail,
		ferrum_chem_owned_buffer *response) {
	return encode_response(result_status, detail, nullptr, response) ?
		FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
}

}  // namespace

extern "C" uint32_t ferrum_chem_abi_version(void) noexcept {
	return FERRUM_CHEM_ADAPTER_ABI_VERSION;
}

extern "C" uint64_t ferrum_chem_capabilities_v1(void) noexcept {
	return FERRUM_CHEM_CAPABILITY_KEKULIZE | FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE |
		FERRUM_CHEM_CAPABILITY_GENERATE_2D | FERRUM_CHEM_CAPABILITY_SMARTS |
		FERRUM_CHEM_CAPABILITY_MOLFILE | FERRUM_CHEM_CAPABILITY_SDF_WRITE |
		FERRUM_CHEM_CAPABILITY_SDF_READ | FERRUM_CHEM_CAPABILITY_MOLFILE_READ |
		FERRUM_CHEM_CAPABILITY_INCHI | FERRUM_CHEM_CAPABILITY_COMPOSITION |
		FERRUM_CHEM_CAPABILITY_SMILES_WRITE | FERRUM_CHEM_CAPABILITY_MOLFILE_TITLE |
		FERRUM_CHEM_CAPABILITY_SMARTS_MATCH;
}

extern "C" uint32_t ferrum_chem_smarts_match_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) noexcept {
	return ferrum_chem::smarts_match_v1(request, request_len, response);
}

extern "C" uint32_t ferrum_chem_kekulize_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) {
		return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	}
	response->data = nullptr;
	response->len = 0;
	try {
		WireMolecule input;
		std::string error;
		if (!parse_request(request, request_len, true, &input, &error)) {
			return emit_error(FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, response);
		}
		try {
			WireMolecule output = kekulize(input);
			return encode_response(FERRUM_CHEM_RESULT_OK, "", &output, response) ?
				FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		} catch (const RDKit::KekulizeException &error) {
			return emit_error(FERRUM_CHEM_RESULT_DEPICTION_FAILURE, error.what(), response);
		} catch (const RDKit::MolSanitizeException &error) {
			return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), response);
		}
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return emit_error(FERRUM_CHEM_RESULT_INTERNAL_FAILURE, error.what(), response);
	} catch (...) {
		return emit_error(FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", response);
	}
}

#ifdef FERRUM_CHEM_ENABLE_DEPICTOR
extern "C" uint32_t ferrum_chem_generate_2d_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) {
		return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	}
	response->data = nullptr;
	response->len = 0;
	try {
		WireMolecule input;
		std::string error;
		if (!parse_request(request, request_len, false, &input, &error)) {
			return encode_coordinate_response(
				FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, nullptr, nullptr, response) ?
				FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		}
		return generate_2d(input, response) ? FERRUM_CHEM_CALL_OK :
			FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const RDDepict::DepictException &error) {
		return encode_coordinate_response(
			FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), nullptr, nullptr, response) ?
			FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return encode_coordinate_response(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, error.what(), nullptr, nullptr, response) ?
			FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (...) {
		return encode_coordinate_response(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", nullptr, nullptr, response) ?
			FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	}
}
#endif

extern "C" uint32_t ferrum_chem_smiles_to_molecule_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) {
		return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	}
	response->data = nullptr;
	response->len = 0;
	try {
		return smiles_to_molecule(request, request_len, response) ? FERRUM_CHEM_CALL_OK :
			FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return ferrum_chem::emit_molecule_response(FERRUM_CHEM_RESULT_INTERNAL_FAILURE, error.what(), nullptr, nullptr,
			response) ? FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (...) {
		return ferrum_chem::emit_molecule_response(FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", nullptr,
			nullptr, response) ? FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	}
}

extern "C" void ferrum_chem_owned_buffer_free_v1(ferrum_chem_owned_buffer *owner) noexcept {
	if (owner == nullptr) {
		return;
	}
	delete[] owner->data;
	owner->data = nullptr;
	owner->len = 0;
}
