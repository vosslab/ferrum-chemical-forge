#include "ferrum_chem_adapter.h"
#include "ferrum_chem_complete_graph.h"
#include "ferrum_chem_text_response.h"

#include <GraphMol/Atom.h>
#include <GraphMol/Bond.h>
#include <GraphMol/RWMol.h>
#include <GraphMol/SmilesParse/SmartsWrite.h>

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iterator>
#include <limits>
#include <new>
#include <stdexcept>
#include <string>
#include <string_view>
#include <unordered_set>
#include <vector>

namespace {

constexpr uint8_t kGraphMagic[] = {'F', 'C', 'G', '1'};
constexpr uint32_t kKnownPresence = FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE |
	FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE |
	FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS;

struct BondRecord {
	uint32_t start;
	uint32_t end;
	RDKit::Bond::BondType order;
	bool aromatic;
	RDKit::Bond::BondStereo stereo;
	RDKit::Bond::BondDir direction;
	uint32_t first_stereo_atom;
	uint32_t second_stereo_atom;
};

uint16_t read_u16(const uint8_t *bytes) {
	return static_cast<uint16_t>(bytes[0]) |
		(static_cast<uint16_t>(bytes[1]) << 8U);
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

bool valid_atom_chirality(uint8_t value, RDKit::Atom::ChiralType *result) {
	switch (value) {
	case FERRUM_CHEM_CHIRAL_UNSPECIFIED:
		*result = RDKit::Atom::CHI_UNSPECIFIED;
		return true;
	case FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW:
		*result = RDKit::Atom::CHI_TETRAHEDRAL_CW;
		return true;
	case FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW:
		*result = RDKit::Atom::CHI_TETRAHEDRAL_CCW;
		return true;
	default:
		return false;
	}
}

bool valid_bond_order(uint8_t value, RDKit::Bond::BondType *result) {
	switch (value) {
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE:
		*result = RDKit::Bond::SINGLE;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE:
		*result = RDKit::Bond::DOUBLE;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE:
		*result = RDKit::Bond::TRIPLE;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC:
		*result = RDKit::Bond::AROMATIC;
		return true;
	case FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE:
		*result = RDKit::Bond::QUADRUPLE;
		return true;
	default:
		return false;
	}
}

bool valid_bond_stereo(uint8_t value, RDKit::Bond::BondStereo *result) {
	switch (value) {
	case FERRUM_CHEM_BOND_STEREO_NONE:
		*result = RDKit::Bond::STEREONONE;
		return true;
	case FERRUM_CHEM_BOND_STEREO_ANY:
		*result = RDKit::Bond::STEREOANY;
		return true;
	case FERRUM_CHEM_BOND_STEREO_Z:
		*result = RDKit::Bond::STEREOZ;
		return true;
	case FERRUM_CHEM_BOND_STEREO_E:
		*result = RDKit::Bond::STEREOE;
		return true;
	case FERRUM_CHEM_BOND_STEREO_CIS:
		*result = RDKit::Bond::STEREOCIS;
		return true;
	case FERRUM_CHEM_BOND_STEREO_TRANS:
		*result = RDKit::Bond::STEREOTRANS;
		return true;
	default:
		return false;
	}
}

bool valid_bond_direction(uint8_t value, RDKit::Bond::BondDir *result) {
	switch (value) {
	case FERRUM_CHEM_BOND_DIRECTION_NONE:
		*result = RDKit::Bond::NONE;
		return true;
	case FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE:
		*result = RDKit::Bond::BEGINWEDGE;
		return true;
	case FERRUM_CHEM_BOND_DIRECTION_BEGINDASH:
		*result = RDKit::Bond::BEGINDASH;
		return true;
	case FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT:
		*result = RDKit::Bond::ENDUPRIGHT;
		return true;
	case FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT:
		*result = RDKit::Bond::ENDDOWNRIGHT;
		return true;
	default:
		return false;
	}
}

bool parse_graph(const uint8_t *request, uint64_t request_len, RDKit::RWMol *molecule,
		std::string *error) {
	if (request == nullptr || request_len < FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES) {
		*error = "complete graph request is missing or truncated";
		return false;
	}
	if (std::memcmp(request, kGraphMagic, sizeof(kGraphMagic)) != 0 ||
		read_u32(request + 4) != FERRUM_CHEM_GRAPH_WIRE_VERSION) {
		*error = "complete graph request has invalid magic or version";
		return false;
	}
	const uint32_t atom_count = read_u32(request + 8);
	const uint32_t bond_count = read_u32(request + 12);
	if (read_u32(request + 16) != FERRUM_CHEM_GRAPH_FLAGS_NONE ||
		atom_count > FERRUM_CHEM_KEKULIZE_MAX_ATOMS ||
		bond_count > FERRUM_CHEM_KEKULIZE_MAX_BONDS) {
		*error = "complete graph request has reserved flags or oversized counts";
		return false;
	}
	const uint64_t expected = FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES +
		static_cast<uint64_t>(atom_count) * FERRUM_CHEM_GRAPH_ATOM_BYTES +
		static_cast<uint64_t>(bond_count) * FERRUM_CHEM_GRAPH_BOND_BYTES;
	if (request_len != expected) {
		*error = "complete graph records are truncated or trailing";
		return false;
	}

	const uint8_t *cursor = request + FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES;
	std::vector<bool> aromatic_atoms;
	aromatic_atoms.reserve(atom_count);
	for (uint32_t index = 0; index < atom_count;
			++index, cursor += FERRUM_CHEM_GRAPH_ATOM_BYTES) {
		RDKit::Atom::ChiralType chirality;
		const uint32_t presence = read_u32(cursor + 4);
		const int32_t charge = read_i32(cursor + 8);
		const uint16_t isotope = read_u16(cursor + 12);
		const uint16_t hydrogens = read_u16(cursor + 14);
		const uint32_t atom_map = read_u32(cursor + 20);
		if (cursor[0] == 0 || cursor[0] > 118 || cursor[1] > 1 || cursor[3] != 0 ||
			!valid_atom_chirality(cursor[2], &chirality) ||
			(presence & ~kKnownPresence) != 0 ||
			((presence & FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE) == 0 && charge != 0) ||
			((presence & FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE) == 0 && isotope != 0) ||
			((presence & FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE) != 0 && isotope == 0) ||
			((presence & FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS) == 0 && hydrogens != 0) ||
			cursor[17] > 1 || read_u16(cursor + 18) != 0 ||
			atom_map > static_cast<uint32_t>(std::numeric_limits<int>::max())) {
			*error = "complete graph request contains an invalid atom";
			return false;
		}
		auto *atom = new RDKit::Atom(cursor[0]);
		atom->setIsAromatic(cursor[1] == 1);
		atom->setChiralTag(chirality);
		if ((presence & FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE) != 0) {
			atom->setFormalCharge(charge);
		}
		if ((presence & FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE) != 0) {
			atom->setIsotope(isotope);
		}
		if ((presence & FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS) != 0) {
			atom->setNumExplicitHs(hydrogens);
		}
		atom->setNumRadicalElectrons(cursor[16]);
		atom->setNoImplicit(cursor[17] == 1);
		if (atom_map != 0) {
			atom->setAtomMapNum(static_cast<int>(atom_map));
		}
		molecule->addAtom(atom, true, true);
		aromatic_atoms.push_back(cursor[1] == 1);
	}

	std::vector<BondRecord> bonds;
	bonds.reserve(bond_count);
	std::unordered_set<uint64_t> edges;
	edges.reserve(bond_count);
	for (uint32_t index = 0; index < bond_count;
			++index, cursor += FERRUM_CHEM_GRAPH_BOND_BYTES) {
		BondRecord bond;
		bond.start = read_u32(cursor);
		bond.end = read_u32(cursor + 4);
		bond.aromatic = cursor[9] == 1;
		bond.first_stereo_atom = read_u32(cursor + 12);
		bond.second_stereo_atom = read_u32(cursor + 16);
		const uint32_t lower = std::min(bond.start, bond.end);
		const uint32_t upper = std::max(bond.start, bond.end);
		const uint64_t edge = (static_cast<uint64_t>(lower) << 32U) | upper;
		const bool no_stereo_atoms =
			bond.first_stereo_atom == FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE &&
			bond.second_stereo_atom == FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE;
		const bool two_stereo_atoms =
			bond.first_stereo_atom != FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE &&
			bond.second_stereo_atom != FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE;
		if (bond.start >= atom_count || bond.end >= atom_count || bond.start == bond.end ||
			!valid_bond_order(cursor[8], &bond.order) || cursor[9] > 1 ||
			!valid_bond_stereo(cursor[10], &bond.stereo) ||
			!valid_bond_direction(cursor[11], &bond.direction) || read_u32(cursor + 20) != 0 ||
			!edges.insert(edge).second ||
			(bond.order == RDKit::Bond::AROMATIC && !bond.aromatic) ||
			(bond.aromatic && (!aromatic_atoms[bond.start] || !aromatic_atoms[bond.end])) ||
			(!no_stereo_atoms && !two_stereo_atoms) ||
			(bond.stereo != RDKit::Bond::STEREONONE && !two_stereo_atoms) ||
			(two_stereo_atoms && (bond.first_stereo_atom >= atom_count ||
				bond.second_stereo_atom >= atom_count ||
				bond.first_stereo_atom == bond.second_stereo_atom ||
				bond.first_stereo_atom == bond.start || bond.first_stereo_atom == bond.end ||
				bond.second_stereo_atom == bond.start || bond.second_stereo_atom == bond.end))) {
			*error = "complete graph request contains an invalid bond";
			return false;
		}
		bonds.push_back(bond);
	}
	for (const BondRecord &bond : bonds) {
		molecule->addBond(bond.start, bond.end, bond.order);
	}
	for (const BondRecord &bond : bonds) {
		RDKit::Bond *output = molecule->getBondBetweenAtoms(bond.start, bond.end);
		output->setIsAromatic(bond.aromatic);
		output->setStereo(bond.stereo);
		output->setBondDir(bond.direction);
		if (bond.first_stereo_atom != FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE) {
			output->setStereoAtoms(bond.first_stereo_atom, bond.second_stereo_atom);
		}
	}
	return true;
}

}  // namespace

bool ferrum_chem::parse_complete_graph(
		const uint8_t *request, uint64_t request_len, RDKit::RWMol *molecule,
		std::string *error) {
	return parse_graph(request, request_len, molecule, error);
}

extern "C" uint32_t ferrum_chem_molecule_to_smarts_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) {
		return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	}
	response->data = nullptr;
	response->len = 0;
	try {
		RDKit::RWMol molecule;
		std::string error;
		if (!ferrum_chem::parse_complete_graph(request, request_len, &molecule, &error)) {
			return ferrum_chem::emit_text_response(
				FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, "", response);
		}
		const std::string smarts = RDKit::MolToSmarts(molecule, true, -1);
		if (smarts.empty()) {
			return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
				"RDKit could not generate SMARTS for this molecule", "", response);
		}
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_OK, "", smarts, response);
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return ferrum_chem::emit_text_response(
			FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), "", response);
	} catch (...) {
		return ferrum_chem::emit_text_response(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", "", response);
	}
}
