#ifndef FERRUM_CHEM_UTF8_H
#define FERRUM_CHEM_UTF8_H

#include <cstddef>
#include <cstdint>
#include <string_view>

namespace ferrum_chem {

inline bool is_valid_utf8(std::string_view text) {
	for (size_t index = 0; index < text.size();) {
		const uint8_t byte = static_cast<uint8_t>(text[index]);
		if (byte <= 0x7fU) {
			++index;
			continue;
		}
		uint32_t code_point = 0;
		size_t continuation_count = 0;
		if (byte >= 0xc2U && byte <= 0xdfU) {
			code_point = byte & 0x1fU;
			continuation_count = 1;
		} else if (byte >= 0xe0U && byte <= 0xefU) {
			code_point = byte & 0x0fU;
			continuation_count = 2;
		} else if (byte >= 0xf0U && byte <= 0xf4U) {
			code_point = byte & 0x07U;
			continuation_count = 3;
		} else {
			return false;
		}
		if (continuation_count >= text.size() - index) {
			return false;
		}
		for (size_t offset = 1; offset <= continuation_count; ++offset) {
			const uint8_t continuation = static_cast<uint8_t>(text[index + offset]);
			if ((continuation & 0xc0U) != 0x80U) {
				return false;
			}
			code_point = (code_point << 6U) | (continuation & 0x3fU);
		}
		if ((continuation_count == 2 && code_point < 0x800U) ||
			(continuation_count == 3 && code_point < 0x10000U) ||
			(code_point >= 0xd800U && code_point <= 0xdfffU) || code_point > 0x10ffffU) {
			return false;
		}
		index += continuation_count + 1;
	}
	return true;
}

}

#endif
