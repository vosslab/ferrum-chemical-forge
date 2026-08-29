#ifndef FERRUM_CHEM_TEXT_OUTPUT_LIMIT_H
#define FERRUM_CHEM_TEXT_OUTPUT_LIMIT_H

#include "ferrum_chem_adapter.h"

#include <cstdint>
#include <limits>

namespace ferrum_chem {

// These are deliberately generous, format-specific maxima for the exact
// bounded wire facts. They are admission bounds, not estimates of typical
// RDKit output: a writer is called only after its entire possible text fits.
inline uint64_t saturating_add(uint64_t left, uint64_t right) {
	return left > std::numeric_limits<uint64_t>::max() - right ?
		std::numeric_limits<uint64_t>::max() : left + right;
}

inline uint64_t saturating_product(uint64_t left, uint64_t right) {
	return left != 0 && right > std::numeric_limits<uint64_t>::max() / left ?
		std::numeric_limits<uint64_t>::max() : left * right;
}

inline uint64_t molblock_text_upper_bound(uint32_t format, uint32_t atom_count,
		uint32_t bond_count, uint64_t title_bytes) {
	const uint64_t atom_bytes = format == FERRUM_CHEM_MOLBLOCK_FORMAT_V2000 ? 256U : 384U;
	const uint64_t bond_bytes = format == FERRUM_CHEM_MOLBLOCK_FORMAT_V2000 ? 128U : 192U;
	return saturating_add(
		saturating_add(saturating_add(4096U, title_bytes),
			saturating_product(atom_count, atom_bytes)),
		saturating_add(saturating_product(bond_count, bond_bytes), 4096U));
}

inline uint64_t smiles_text_upper_bound(uint32_t atom_count, uint32_t bond_count) {
	return saturating_add(saturating_add(1024U, saturating_product(atom_count, 256U)),
		saturating_product(bond_count, 128U));
}

inline uint64_t inchi_text_upper_bound(uint32_t atom_count, uint32_t bond_count) {
	return saturating_add(saturating_add(2048U, saturating_product(atom_count, 512U)),
		saturating_product(bond_count, 256U));
}

inline uint64_t sdf_record_text_upper_bound(uint32_t format, uint32_t atom_count,
		uint32_t bond_count, uint64_t title_bytes, uint64_t property_name_bytes,
		uint64_t property_value_bytes, uint32_t property_count) {
	const uint64_t molblock = molblock_text_upper_bound(
		format, atom_count, bond_count, title_bytes);
	const uint64_t property_markup = saturating_product(property_count, 16U);
	return saturating_add(saturating_add(saturating_add(molblock, property_name_bytes),
		property_value_bytes), saturating_add(property_markup, 16U));
}

inline uint64_t maximum_text_output_bytes() {
	return static_cast<uint64_t>(FERRUM_CHEM_MAX_RESPONSE_BYTES) -
		FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES;
}

inline bool text_output_is_admitted(uint64_t upper_bound, uint64_t maximum_text_bytes) {
	return maximum_text_bytes != 0 &&
		maximum_text_bytes <= maximum_text_output_bytes() &&
		upper_bound <= maximum_text_bytes;
}

}  // namespace ferrum_chem

#endif
