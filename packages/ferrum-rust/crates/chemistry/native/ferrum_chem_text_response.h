#ifndef FERRUM_CHEM_TEXT_RESPONSE_H
#define FERRUM_CHEM_TEXT_RESPONSE_H

#include "ferrum_chem_adapter.h"

#include <cstdint>
#include <string_view>

namespace ferrum_chem {

uint32_t emit_text_response(
	uint32_t status,
	std::string_view detail,
	std::string_view text,
	ferrum_chem_owned_buffer *response);

}  // namespace ferrum_chem

#endif
