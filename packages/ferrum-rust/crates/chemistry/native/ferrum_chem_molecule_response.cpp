#include "ferrum_chem_molecule_response.h"

#include <GraphMol/Atom.h>
#include <GraphMol/Bond.h>
#include <GraphMol/Conformer.h>
#include <GraphMol/ROMol.h>
#include <GraphMol/SmilesParse/SmilesWrite.h>

#include <algorithm>
#include <bit>
#include <cmath>
#include <cstdint>
#include <iterator>
#include <limits>
#include <new>
#include <stdexcept>
#include <string>
#include <vector>

namespace {

constexpr uint8_t kMoleculeResponseMagic[] = {'F', 'C', 'M', '1'};

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

void append_f64(std::vector<uint8_t> &bytes, double value) {
	const uint64_t bits = std::bit_cast<uint64_t>(value);
	for (unsigned int shift = 0; shift < 64U; shift += 8U) {
		bytes.push_back(static_cast<uint8_t>(bits >> shift));
	}
}

uint8_t bond_type(RDKit::Bond::BondType type) {
	switch (type) {
	case RDKit::Bond::SINGLE: return FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE;
	case RDKit::Bond::DOUBLE: return FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE;
	case RDKit::Bond::TRIPLE: return FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE;
	case RDKit::Bond::AROMATIC: return FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC;
	case RDKit::Bond::QUADRUPLE: return FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE;
	default: throw std::runtime_error("RDKit bond type is outside FCM1 vocabulary");
	}
}

uint8_t chirality(RDKit::Atom::ChiralType tag) {
	switch (tag) {
	case RDKit::Atom::CHI_UNSPECIFIED: return FERRUM_CHEM_CHIRAL_UNSPECIFIED;
	case RDKit::Atom::CHI_TETRAHEDRAL_CW: return FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW;
	case RDKit::Atom::CHI_TETRAHEDRAL_CCW: return FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW;
	default: return FERRUM_CHEM_CHIRAL_OTHER;
	}
}

uint8_t stereo(RDKit::Bond::BondStereo value) {
	switch (value) {
	case RDKit::Bond::STEREONONE: return FERRUM_CHEM_BOND_STEREO_NONE;
	case RDKit::Bond::STEREOANY: return FERRUM_CHEM_BOND_STEREO_ANY;
	case RDKit::Bond::STEREOZ: return FERRUM_CHEM_BOND_STEREO_Z;
	case RDKit::Bond::STEREOE: return FERRUM_CHEM_BOND_STEREO_E;
	case RDKit::Bond::STEREOCIS: return FERRUM_CHEM_BOND_STEREO_CIS;
	case RDKit::Bond::STEREOTRANS: return FERRUM_CHEM_BOND_STEREO_TRANS;
	default: return FERRUM_CHEM_BOND_STEREO_OTHER;
	}
}

uint8_t direction(RDKit::Bond::BondDir value) {
	switch (value) {
	case RDKit::Bond::NONE: return FERRUM_CHEM_BOND_DIRECTION_NONE;
	case RDKit::Bond::BEGINWEDGE: return FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE;
	case RDKit::Bond::BEGINDASH: return FERRUM_CHEM_BOND_DIRECTION_BEGINDASH;
	case RDKit::Bond::ENDUPRIGHT: return FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT;
	case RDKit::Bond::ENDDOWNRIGHT: return FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT;
	default: return FERRUM_CHEM_BOND_DIRECTION_OTHER;
	}
}

}  // namespace

namespace ferrum_chem {

bool encode_molecule_response_bytes(
		uint32_t status, std::string_view detail, const RDKit::ROMol *molecule,
		const RDKit::Conformer *conformer, std::vector<uint8_t> *bytes) {
	if (bytes == nullptr || detail.size() > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES) return false;
	const uint32_t atom_count = molecule == nullptr ? 0U : molecule->getNumAtoms();
	const uint32_t bond_count = molecule == nullptr ? 0U : molecule->getNumBonds();
	const std::string smiles = molecule == nullptr ? "" : RDKit::MolToSmiles(*molecule);
	if (status == FERRUM_CHEM_RESULT_OK && (conformer == nullptr || smiles.empty())) return false;
	if (status != FERRUM_CHEM_RESULT_OK &&
		(detail.empty() || molecule != nullptr || conformer != nullptr)) return false;
	const uint64_t records = static_cast<uint64_t>(atom_count) *
			FERRUM_CHEM_MOLECULE_ATOM_BYTES +
		static_cast<uint64_t>(bond_count) * FERRUM_CHEM_MOLECULE_BOND_BYTES +
		static_cast<uint64_t>(atom_count) * FERRUM_CHEM_COORDINATE_BYTES;
	const uint64_t total = FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES +
		detail.size() + smiles.size() + records;
	if (atom_count > FERRUM_CHEM_KEKULIZE_MAX_ATOMS ||
		bond_count > FERRUM_CHEM_KEKULIZE_MAX_BONDS ||
		smiles.size() > FERRUM_CHEM_SMILES_MAX_BYTES ||
		total > FERRUM_CHEM_MAX_RESPONSE_BYTES ||
		total > std::numeric_limits<size_t>::max()) return false;
	bytes->clear();
	bytes->reserve(static_cast<size_t>(total));
	bytes->insert(bytes->end(), std::begin(kMoleculeResponseMagic),
		std::end(kMoleculeResponseMagic));
	append_u32(*bytes, FERRUM_CHEM_MOLECULE_WIRE_VERSION);
	append_u32(*bytes, status);
	append_u32(*bytes, static_cast<uint32_t>(detail.size()));
	append_u32(*bytes, static_cast<uint32_t>(smiles.size()));
	append_u32(*bytes, atom_count);
	append_u32(*bytes, bond_count);
	append_u32(*bytes, FERRUM_CHEM_MOLECULE_FLAGS_NONE);
	bytes->insert(bytes->end(), detail.begin(), detail.end());
	bytes->insert(bytes->end(), smiles.begin(), smiles.end());
	if (molecule != nullptr) {
		for (uint32_t index = 0; index < atom_count; ++index) {
			const RDKit::Atom *atom = molecule->getAtomWithIdx(index);
			const unsigned int isotope = atom->getIsotope();
			const unsigned int hydrogens = atom->getNumExplicitHs();
			if (isotope > UINT16_MAX || hydrogens > UINT16_MAX ||
				atom->getAtomicNum() == 0 || atom->getAtomicNum() > 118 ||
				atom->getNumRadicalElectrons() > UINT8_MAX) return false;
			bytes->push_back(static_cast<uint8_t>(atom->getAtomicNum()));
			bytes->push_back(atom->getIsAromatic() ? 1U : 0U);
			bytes->push_back(chirality(atom->getChiralTag()));
			bytes->push_back(FERRUM_CHEM_MOLECULE_RESERVED);
			append_i32(*bytes, atom->getFormalCharge());
			append_u16(*bytes, static_cast<uint16_t>(isotope));
			append_u16(*bytes, static_cast<uint16_t>(hydrogens));
			bytes->push_back(static_cast<uint8_t>(atom->getNumRadicalElectrons()));
			bytes->push_back(atom->getNoImplicit() ? 1U : 0U);
			append_u16(*bytes, FERRUM_CHEM_MOLECULE_RESERVED);
			append_u32(*bytes, atom->getAtomMapNum());
		}
		for (uint32_t index = 0; index < bond_count; ++index) {
			const RDKit::Bond *bond = molecule->getBondWithIdx(index);
			const std::vector<int> &references = bond->getStereoAtoms();
			if (references.size() != 0 && references.size() != 2) return false;
			append_u32(*bytes, bond->getBeginAtomIdx());
			append_u32(*bytes, bond->getEndAtomIdx());
			bytes->push_back(bond_type(bond->getBondType()));
			bytes->push_back(bond->getIsAromatic() ? 1U : 0U);
			bytes->push_back(stereo(bond->getStereo()));
			bytes->push_back(direction(bond->getBondDir()));
			append_u32(*bytes, references.empty() ?
				FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE :
				static_cast<uint32_t>(references[0]));
			append_u32(*bytes, references.empty() ?
				FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE :
				static_cast<uint32_t>(references[1]));
			append_u32(*bytes, FERRUM_CHEM_MOLECULE_RESERVED);
		}
		for (uint32_t index = 0; index < atom_count; ++index) {
			const RDGeom::Point3D &point = conformer->getAtomPos(index);
			if (!std::isfinite(point.x) || !std::isfinite(point.y)) return false;
			append_f64(*bytes, point.x);
			append_f64(*bytes, point.y);
		}
	}
	return bytes->size() == total;
}

bool emit_molecule_response(
		uint32_t status, std::string_view detail, const RDKit::ROMol *molecule,
		const RDKit::Conformer *conformer, ferrum_chem_owned_buffer *response) {
	try {
		if (response == nullptr) return false;
		std::vector<uint8_t> bytes;
		if (!encode_molecule_response_bytes(
				status, detail, molecule, conformer, &bytes)) return false;
		response->data = new uint8_t[bytes.size()];
		std::copy(bytes.begin(), bytes.end(), response->data);
		response->len = bytes.size();
		return true;
	} catch (...) {
		return false;
	}
}

}  // namespace ferrum_chem
