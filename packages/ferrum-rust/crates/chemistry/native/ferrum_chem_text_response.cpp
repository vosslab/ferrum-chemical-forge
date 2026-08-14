#include "ferrum_chem_text_response.h"

#include <cstddef>
#include <cstdint>
#include <cstring>
#include <iterator>
#include <limits>
#include <new>
#include <vector>

namespace {

constexpr uint8_t kTextMagic[] = {'F', 'C', 'T', '1'};

void append_u32(std::vector<uint8_t> &bytes, uint32_t value) {
	bytes.push_back(static_cast<uint8_t>(value));
	bytes.push_back(static_cast<uint8_t>(value >> 8U));
	bytes.push_back(static_cast<uint8_t>(value >> 16U));
	bytes.push_back(static_cast<uint8_t>(value >> 24U));
}

bool encode_text_response(uint32_t status, std::string_view detail, std::string_view text,
		ferrum_chem_owned_buffer *response) {
	try {
		if (response == nullptr || detail.size() > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES ||
			detail.size() > std::numeric_limits<uint32_t>::max() ||
			text.size() > std::numeric_limits<uint32_t>::max()) {
			return false;
		}
		if ((status == FERRUM_CHEM_RESULT_OK && (!detail.empty() || text.empty())) ||
			(status != FERRUM_CHEM_RESULT_OK && (detail.empty() || !text.empty()))) {
			return false;
		}
		const uint64_t total = FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES + detail.size() +
			text.size();
		if (total > FERRUM_CHEM_MAX_RESPONSE_BYTES ||
			total > std::numeric_limits<size_t>::max()) {
			return false;
		}
		std::vector<uint8_t> bytes;
		bytes.reserve(static_cast<size_t>(total));
		bytes.insert(bytes.end(), std::begin(kTextMagic), std::end(kTextMagic));
		append_u32(bytes, FERRUM_CHEM_TEXT_WIRE_VERSION);
		append_u32(bytes, status);
		append_u32(bytes, static_cast<uint32_t>(detail.size()));
		append_u32(bytes, static_cast<uint32_t>(text.size()));
		append_u32(bytes, FERRUM_CHEM_TEXT_FLAGS_NONE);
		bytes.insert(bytes.end(), detail.begin(), detail.end());
		bytes.insert(bytes.end(), text.begin(), text.end());
		if (bytes.size() != total) {
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

}  // namespace

uint32_t ferrum_chem::emit_text_response(
		uint32_t status, std::string_view detail, std::string_view text,
		ferrum_chem_owned_buffer *response) {
	return encode_text_response(status, detail, text, response) ? FERRUM_CHEM_CALL_OK :
		FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
}
