#ifndef FERRUM_CHEM_MOLECULE_RESPONSE_H
#define FERRUM_CHEM_MOLECULE_RESPONSE_H

#include "ferrum_chem_adapter.h"

#include <cstdint>
#include <string_view>
#include <vector>

namespace RDKit {
class Conformer;
class ROMol;
}

namespace ferrum_chem {

bool encode_molecule_response_bytes(
	uint32_t status, std::string_view detail, const RDKit::ROMol *molecule,
	const RDKit::Conformer *conformer, std::vector<uint8_t> *bytes);

bool emit_molecule_response(
	uint32_t status, std::string_view detail, const RDKit::ROMol *molecule,
	const RDKit::Conformer *conformer, ferrum_chem_owned_buffer *response);

}  // namespace ferrum_chem

#endif
