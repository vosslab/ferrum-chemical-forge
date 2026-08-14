#include "ferrum_chem_adapter.h"
#include "ferrum_chem_complete_graph.h"

#include <GraphMol/Atom.h>
#include <GraphMol/MolOps.h>
#include <GraphMol/PeriodicTable.h>
#include <GraphMol/RWMol.h>
#include <GraphMol/SanitException.h>

#include <algorithm>
#include <bit>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iterator>
#include <limits>
#include <map>
#include <new>
#include <stdexcept>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {

constexpr uint8_t kCompositionMagic[] = {'F', 'C', 'S', '1'};

struct CompositionEntry {
	uint8_t atomic_number;
	uint16_t isotope;
	uint64_t count;
	double average_mass_contribution;
};

enum class EntryAddResult {
	Ok,
	InvalidMass,
	ResourceLimit,
};

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

void append_u64(std::vector<uint8_t> &bytes, uint64_t value) {
	for (unsigned int shift = 0; shift < 64U; shift += 8U) {
		bytes.push_back(static_cast<uint8_t>(value >> shift));
	}
}

void append_i64(std::vector<uint8_t> &bytes, int64_t value) {
	append_u64(bytes, std::bit_cast<uint64_t>(value));
}

void append_f64(std::vector<uint8_t> &bytes, double value) {
	append_u64(bytes, std::bit_cast<uint64_t>(value));
}

bool hill_less(const CompositionEntry &first, const CompositionEntry &second) {
	const RDKit::PeriodicTable *table = RDKit::PeriodicTable::getTable();
	const std::string first_symbol = table->getElementSymbol(first.atomic_number);
	const std::string second_symbol = table->getElementSymbol(second.atomic_number);
	if (first_symbol == "C") {
		return second_symbol != "C" || first.isotope < second.isotope;
	}
	if (second_symbol == "C") return false;
	if (first_symbol == "H") {
		return second_symbol != "H" || first.isotope < second.isotope;
	}
	if (second_symbol == "H") return false;
	return std::make_pair(first.isotope, first_symbol) <
		std::make_pair(second.isotope, second_symbol);
}

EntryAddResult add_entry(std::map<std::pair<uint8_t, uint16_t>, CompositionEntry> *entries,
		uint8_t atomic_number, uint16_t isotope, uint64_t count, double contribution) {
	if (count == 0 || !std::isfinite(contribution) || contribution <= 0.0) {
		return EntryAddResult::InvalidMass;
	}
	auto [position, inserted] = entries->try_emplace(
		std::make_pair(atomic_number, isotope),
		CompositionEntry{atomic_number, isotope, 0, 0.0});
	CompositionEntry &entry = position->second;
	if (entry.count > std::numeric_limits<uint64_t>::max() - count) {
		return EntryAddResult::ResourceLimit;
	}
	entry.count += count;
	entry.average_mass_contribution += contribution;
	return std::isfinite(entry.average_mass_contribution) &&
		entry.average_mass_contribution > 0.0 ? EntryAddResult::Ok :
		EntryAddResult::ResourceLimit;
}

bool encode_response(uint32_t status, std::string_view detail, std::string_view formula,
		const std::vector<CompositionEntry> &entries, int64_t net_charge,
		double average_mass, double exact_mass, ferrum_chem_owned_buffer *response) {
	if (response == nullptr || detail.size() > FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES ||
		formula.size() > FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES ||
		detail.size() > std::numeric_limits<uint32_t>::max() ||
		formula.size() > std::numeric_limits<uint32_t>::max() ||
		entries.size() > std::numeric_limits<uint32_t>::max()) return false;
	if (status == FERRUM_CHEM_RESULT_OK) {
		if (!detail.empty() || formula.empty() || entries.empty() ||
			!std::isfinite(average_mass) || average_mass <= 0.0 ||
			!std::isfinite(exact_mass) || exact_mass <= 0.0) return false;
	} else if (detail.empty() || !formula.empty() || !entries.empty() ||
			net_charge != 0 || average_mass != 0.0 || exact_mass != 0.0) {
		return false;
	}
	const uint64_t entry_bytes = static_cast<uint64_t>(entries.size()) *
		FERRUM_CHEM_COMPOSITION_ENTRY_BYTES;
	const uint64_t total = FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES +
		detail.size() + formula.size() + entry_bytes;
	if (total > FERRUM_CHEM_MAX_RESPONSE_BYTES ||
		total > std::numeric_limits<size_t>::max()) return false;
	std::vector<uint8_t> bytes;
	bytes.reserve(static_cast<size_t>(total));
	bytes.insert(bytes.end(), std::begin(kCompositionMagic), std::end(kCompositionMagic));
	append_u32(bytes, FERRUM_CHEM_COMPOSITION_WIRE_VERSION);
	append_u32(bytes, status);
	append_u32(bytes, static_cast<uint32_t>(detail.size()));
	append_u32(bytes, static_cast<uint32_t>(formula.size()));
	append_u32(bytes, static_cast<uint32_t>(entries.size()));
	append_u32(bytes, FERRUM_CHEM_COMPOSITION_FLAGS_NONE);
	append_u32(bytes, 0U);
	append_i64(bytes, net_charge);
	append_f64(bytes, average_mass);
	append_f64(bytes, exact_mass);
	bytes.insert(bytes.end(), detail.begin(), detail.end());
	bytes.insert(bytes.end(), formula.begin(), formula.end());
	for (const CompositionEntry &entry : entries) {
		bytes.push_back(entry.atomic_number);
		bytes.push_back(entry.isotope == 0 ? 0U : 1U);
		append_u16(bytes, 0U);
		append_u16(bytes, entry.isotope);
		append_u16(bytes, 0U);
		append_u64(bytes, entry.count);
		append_f64(bytes, entry.average_mass_contribution);
	}
	if (bytes.size() != total) return false;
	response->data = new uint8_t[bytes.size()];
	std::memcpy(response->data, bytes.data(), bytes.size());
	response->len = bytes.size();
	return true;
}

uint32_t emit_error(uint32_t status, std::string_view detail,
		ferrum_chem_owned_buffer *response) {
	return encode_response(status, detail, "", {}, 0, 0.0, 0.0, response) ?
		FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
}

bool calculate_composition(RDKit::RWMol *molecule,
		std::vector<CompositionEntry> *ordered_entries, std::string *formula,
		int64_t *net_charge, double *average_mass, double *exact_mass,
		uint32_t *failure_status, std::string *error) {
	if (molecule->getNumAtoms() == 0) {
		*failure_status = FERRUM_CHEM_RESULT_INVALID_MOLECULE;
		*error = "molecule composition requires at least one atom";
		return false;
	}
	RDKit::MolOps::sanitizeMol(*molecule);
	const RDKit::PeriodicTable *table = RDKit::PeriodicTable::getTable();
	const double hydrogen_mass = table->getAtomicWeight(1);
	if (!std::isfinite(hydrogen_mass) || hydrogen_mass <= 0.0) {
		throw std::runtime_error("RDKit returned an invalid ordinary-hydrogen mass");
	}
	std::map<std::pair<uint8_t, uint16_t>, CompositionEntry> entries;
	int64_t charge = 0;
	uint64_t attached_hydrogens = 0;
	for (const RDKit::Atom *atom : molecule->atoms()) {
		const unsigned int atomic_number = atom->getAtomicNum();
		const unsigned int isotope = atom->getIsotope();
		if (atomic_number == 0 || atomic_number > 118 || isotope > UINT16_MAX) {
			*failure_status = FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE;
			*error = "RDKit molecule contains an unsupported element or isotope";
			return false;
		}
		const double mass = atom->getMass();
		const EntryAddResult atom_result = add_entry(&entries,
			static_cast<uint8_t>(atomic_number), static_cast<uint16_t>(isotope), 1, mass);
		if (atom_result != EntryAddResult::Ok) {
			*failure_status = atom_result == EntryAddResult::ResourceLimit ?
				FERRUM_CHEM_RESULT_RESOURCE_LIMIT : FERRUM_CHEM_RESULT_INVALID_MOLECULE;
			*error = "molecule composition atom count or mass is invalid";
			return false;
		}
		const int atom_charge = atom->getFormalCharge();
		if ((atom_charge > 0 && charge > std::numeric_limits<int64_t>::max() - atom_charge) ||
			(atom_charge < 0 && charge < std::numeric_limits<int64_t>::min() - atom_charge)) {
			*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			*error = "molecule composition charge overflows i64";
			return false;
		}
		charge += atom_charge;
		if (atomic_number != 1) {
			const uint64_t hydrogens = atom->getTotalNumHs(false);
			if (attached_hydrogens > std::numeric_limits<uint64_t>::max() - hydrogens) {
				*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
				*error = "molecule composition hydrogen count overflows u64";
				return false;
			}
			attached_hydrogens += hydrogens;
		}
	}
	if (attached_hydrogens > static_cast<uint64_t>(std::numeric_limits<int>::max()) ||
		charge < std::numeric_limits<int>::min() ||
		charge > std::numeric_limits<int>::max()) {
		*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
		*error = "molecule composition exceeds RDKit formula or exact-mass integer limits";
		return false;
	}
	if (attached_hydrogens != 0) {
		const double contribution = static_cast<double>(attached_hydrogens) * hydrogen_mass;
		const EntryAddResult hydrogen_result =
			add_entry(&entries, 1U, 0U, attached_hydrogens, contribution);
		if (hydrogen_result != EntryAddResult::Ok) {
			*failure_status = hydrogen_result == EntryAddResult::ResourceLimit ?
				FERRUM_CHEM_RESULT_RESOURCE_LIMIT : FERRUM_CHEM_RESULT_INVALID_MOLECULE;
			*error = "molecule composition hydrogen mass is invalid";
			return false;
		}
	}
	ordered_entries->clear();
	ordered_entries->reserve(entries.size());
	for (const auto &item : entries) ordered_entries->push_back(item.second);
	std::sort(ordered_entries->begin(), ordered_entries->end(), hill_less);
	double total = 0.0;
	for (const CompositionEntry &entry : *ordered_entries) {
		if (entry.count > std::numeric_limits<unsigned int>::max()) {
			*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			*error = "molecule composition entry exceeds RDKit formula count limits";
			return false;
		}
		total += entry.average_mass_contribution;
		if (!std::isfinite(total)) {
			*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			*error = "molecule composition average mass is not finite";
			return false;
		}
	}
	const double exact = RDKit::MolOps::getExactMolWt(*molecule, false);
	const std::string rdkit_formula = RDKit::MolOps::getMolFormula(*molecule, true, false);
	if (rdkit_formula.size() > FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES) {
		*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
		*error = "RDKit molecule composition formula exceeds the ABI limit";
		return false;
	}
	if (rdkit_formula.empty() || !std::isfinite(total) || total <= 0.0 ||
		!std::isfinite(exact) || exact <= 0.0) {
		*failure_status = FERRUM_CHEM_RESULT_INVALID_MOLECULE;
		*error = "RDKit returned invalid molecule composition values";
		return false;
	}
	*formula = rdkit_formula;
	*net_charge = charge;
	*average_mass = total;
	*exact_mass = exact;
	return true;
}

}  // namespace

extern "C" uint32_t ferrum_chem_molecule_composition_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		RDKit::RWMol molecule;
		std::string error;
		if (!ferrum_chem::parse_complete_graph(request, request_len, &molecule, &error)) {
			return emit_error(FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, response);
		}
		std::vector<CompositionEntry> entries;
		std::string formula;
		int64_t net_charge = 0;
		double average_mass = 0.0;
		double exact_mass = 0.0;
		uint32_t failure_status = FERRUM_CHEM_RESULT_INVALID_MOLECULE;
		if (!calculate_composition(&molecule, &entries, &formula, &net_charge,
				&average_mass, &exact_mass, &failure_status, &error)) {
			return emit_error(failure_status, error, response);
		}
		return encode_response(FERRUM_CHEM_RESULT_OK, "", formula, entries,
			net_charge, average_mass, exact_mass, response) ?
			FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const RDKit::MolSanitizeException &error) {
		return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), response);
	} catch (const std::exception &error) {
		return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), response);
	} catch (...) {
		return emit_error(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", response);
	}
}
