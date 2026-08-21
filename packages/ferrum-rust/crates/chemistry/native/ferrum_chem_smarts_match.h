#ifndef FERRUM_CHEM_SMARTS_MATCH_H
#define FERRUM_CHEM_SMARTS_MATCH_H

#include "ferrum_chem_adapter.h"

namespace ferrum_chem {

uint32_t smarts_match_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) noexcept;

}  // namespace ferrum_chem

#endif
