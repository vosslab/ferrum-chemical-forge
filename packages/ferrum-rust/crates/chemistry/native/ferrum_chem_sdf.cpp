#include "ferrum_chem_adapter.h"
#include "ferrum_chem_molblock_request.h"
#include "ferrum_chem_text_response.h"
#include "ferrum_chem_text_output_limit.h"
#include "ferrum_chem_utf8.h"
#include "ferrum_chem_writer_probe.h"

#include <GraphMol/FileParsers/MolWriters.h>
#include <GraphMol/RWMol.h>

#include <cstdint>
#include <cstring>
#include <limits>
#include <new>
#include <string>
#include <string_view>
#include <unordered_set>
#include <utility>
#include <vector>

namespace {

constexpr uint8_t kSdfMagic[] = {'F', 'S', 'D', '1'};

uint32_t read_u32(const uint8_t *bytes) {
	return static_cast<uint32_t>(bytes[0]) |
		(static_cast<uint32_t>(bytes[1]) << 8U) |
		(static_cast<uint32_t>(bytes[2]) << 16U) |
		(static_cast<uint32_t>(bytes[3]) << 24U);
}

class Reader {
public:
	Reader(const uint8_t *bytes, uint64_t length) : cursor_(bytes), remaining_(length) {}

	bool take(uint64_t length, const uint8_t **value) {
		if (length > remaining_) return false;
		*value = cursor_;
		cursor_ += length;
		remaining_ -= length;
		return true;
	}

	bool u32(uint32_t *value) {
		const uint8_t *bytes = nullptr;
		if (!take(4U, &bytes)) return false;
		*value = read_u32(bytes);
		return true;
	}

	uint64_t remaining() const { return remaining_; }

private:
	const uint8_t *cursor_;
	uint64_t remaining_;
};

bool text(Reader *reader, uint32_t length, std::string *value) {
	const uint8_t *bytes = nullptr;
	if (!reader->take(length, &bytes)) return false;
	value->assign(reinterpret_cast<const char *>(bytes), length);
	return value->find('\0') == std::string::npos && ferrum_chem::is_valid_utf8(*value);
}

bool valid_title(const std::string &title) {
	return title.find('\r') == std::string::npos && title.find('\n') == std::string::npos;
}

bool valid_property_name(const std::string &name) {
	return !name.empty() && name.find('\r') == std::string::npos &&
		name.find('\n') == std::string::npos;
}

bool valid_property_value(const std::string &value) {
	return value.find("\n\n") == std::string::npos &&
		value.find("\r\n\r\n") == std::string::npos;
}

bool preflight_sdf_text_upper_bound(const uint8_t *request, uint64_t request_len,
		uint64_t *upper_bound, std::string *error, uint32_t *failure_status) {
	if (request == nullptr || request_len < FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES ||
		std::memcmp(request, kSdfMagic, sizeof(kSdfMagic)) != 0 ||
		read_u32(request + 4) != FERRUM_CHEM_SDF_WIRE_VERSION) {
		*error = "SDF request is missing or has invalid magic or version";
		return false;
	}
	const uint32_t record_count = read_u32(request + 8);
	if (record_count > FERRUM_CHEM_SDF_MAX_RECORDS) {
		*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
		*error = "SDF record count exceeds the ABI limit";
		return false;
	}
	if (record_count == 0 || read_u32(request + 12) != FERRUM_CHEM_SDF_FLAGS_NONE) {
		*error = "SDF request has no records, too many records, or nonzero reserved flags";
		return false;
	}
	Reader reader(request + FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES,
		request_len - FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES);
	uint64_t total = 0;
	for (uint32_t index = 0; index < record_count; ++index) {
		uint32_t molecule_length = 0;
		uint32_t title_length = 0;
		uint32_t property_count = 0;
		uint32_t flags = 0;
		const uint8_t *molecule = nullptr;
		if (!reader.u32(&molecule_length) || !reader.u32(&title_length) ||
			!reader.u32(&property_count) || !reader.u32(&flags) ||
			flags != FERRUM_CHEM_SDF_FLAGS_NONE ||
			!reader.take(molecule_length, &molecule) ||
			molecule_length < FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES ||
			std::memcmp(molecule, "FCB1", 4) != 0 ||
			read_u32(molecule + 4) != FERRUM_CHEM_MOLBLOCK_WIRE_VERSION) {
			*error = "SDF record is truncated or has an invalid molblock request";
			return false;
		}
		if (property_count > FERRUM_CHEM_SDF_MAX_PROPERTIES) {
			*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			*error = "SDF property count exceeds the ABI limit";
			return false;
		}
		const uint32_t format = read_u32(molecule + 8);
		const uint32_t atom_count = read_u32(molecule + 12);
		const uint32_t bond_count = read_u32(molecule + 16);
		const uint8_t *title = nullptr;
		if ((format != FERRUM_CHEM_MOLBLOCK_FORMAT_V2000 &&
				format != FERRUM_CHEM_MOLBLOCK_FORMAT_V3000) ||
			!reader.take(title_length, &title)) {
			*error = "SDF record has an unsupported format or truncated title";
			return false;
		}
		uint64_t property_name_bytes = 0;
		uint64_t property_value_bytes = 0;
		for (uint32_t property_index = 0; property_index < property_count; ++property_index) {
			uint32_t name_length = 0;
			uint32_t value_length = 0;
			const uint8_t *ignored = nullptr;
			if (!reader.u32(&name_length) || !reader.u32(&value_length) ||
				!reader.take(name_length, &ignored) || !reader.take(value_length, &ignored)) {
				*error = "SDF property is truncated";
				return false;
			}
			property_name_bytes = ferrum_chem::saturating_add(property_name_bytes, name_length);
			property_value_bytes = ferrum_chem::saturating_add(property_value_bytes, value_length);
		}
		total = ferrum_chem::saturating_add(total,
			ferrum_chem::sdf_record_text_upper_bound(format, atom_count, bond_count,
				title_length, property_name_bytes, property_value_bytes, property_count));
	}
	if (reader.remaining() != 0) {
		*error = "SDF request has trailing bytes";
		return false;
	}
	*upper_bound = total;
	return true;
}

bool parse_record(Reader *reader, uint32_t index, std::string *output,
		std::string *error, uint32_t *failure_status) {
	uint32_t molecule_length = 0;
	uint32_t title_length = 0;
	uint32_t property_count = 0;
	uint32_t flags = 0;
	if (!reader->u32(&molecule_length) || !reader->u32(&title_length) ||
		!reader->u32(&property_count) || !reader->u32(&flags) ||
		flags != FERRUM_CHEM_SDF_FLAGS_NONE ||
		static_cast<uint64_t>(property_count) > reader->remaining() /
			FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES) {
		*error = "SDF record header is truncated or invalid";
		return false;
	}
	if (property_count > FERRUM_CHEM_SDF_MAX_PROPERTIES) {
		*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
		*error = "SDF property count exceeds the ABI limit";
		return false;
	}
	const uint8_t *molecule_bytes = nullptr;
	if (!reader->take(molecule_length, &molecule_bytes)) {
		*error = "SDF record molecule is truncated";
		return false;
	}
	RDKit::RWMol molecule;
	uint32_t format = 0;
	if (!ferrum_chem::parse_molblock_request(
			molecule_bytes, molecule_length, &molecule, &format, error)) {
		return false;
	}
	std::string title;
	if (!text(reader, title_length, &title) || !valid_title(title)) {
		*error = "SDF record title is invalid UTF-8 or contains a line break";
		return false;
	}
	molecule.setProp(RDKit::common_properties::_Name, title);
	std::vector<std::string> property_names;
	property_names.reserve(property_count);
	std::unordered_set<std::string> unique_names;
	unique_names.reserve(property_count);
	for (uint32_t property_index = 0; property_index < property_count; ++property_index) {
		uint32_t name_length = 0;
		uint32_t value_length = 0;
		std::string name;
		std::string value;
		if (!reader->u32(&name_length) || !reader->u32(&value_length) ||
			!text(reader, name_length, &name) || !text(reader, value_length, &value) ||
			!valid_property_name(name) || !valid_property_value(value) ||
			!unique_names.insert(name).second) {
			*error = "SDF property is malformed, duplicated, or not representable";
			return false;
		}
		molecule.setProp(name, value);
		property_names.push_back(std::move(name));
	}
	const bool force_v3000 = format == FERRUM_CHEM_MOLBLOCK_FORMAT_V3000;
	ferrum_chem::record_native_text_writer_invocation(
		ferrum_chem::NativeTextWriter::Sdf);
	const std::string record = RDKit::SDWriter::getText(
		molecule, -1, true, force_v3000, static_cast<int>(index), &property_names);
	if (record.empty() || record.size() > FERRUM_CHEM_MAX_RESPONSE_BYTES - output->size()) {
		*error = "SDF output exceeds the ABI response bound";
		return false;
	}
	output->append(record);
	return true;
}

bool records_to_sdf(const uint8_t *request, uint64_t request_len,
		uint64_t maximum_text_bytes, std::string *output, std::string *error,
		uint32_t *failure_status) {
	*failure_status = FERRUM_CHEM_RESULT_MALFORMED_REQUEST;
	if (request == nullptr || request_len < FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES ||
		request_len > FERRUM_CHEM_MAX_RESPONSE_BYTES ||
		std::memcmp(request, kSdfMagic, sizeof(kSdfMagic)) != 0 ||
		read_u32(request + 4) != FERRUM_CHEM_SDF_WIRE_VERSION) {
		*error = "SDF request is missing, oversized, or has invalid magic or version";
		return false;
	}
	uint64_t upper_bound = 0;
	if (!preflight_sdf_text_upper_bound(
			request, request_len, &upper_bound, error, failure_status)) {
		return false;
	}
	if (!ferrum_chem::text_output_is_admitted(upper_bound, maximum_text_bytes)) {
		*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
		*error = "SDF output upper bound exceeds the requested text limit";
		return false;
	}
	const uint32_t record_count = read_u32(request + 8);
	if (record_count == 0 || record_count > FERRUM_CHEM_SDF_MAX_RECORDS ||
		read_u32(request + 12) != FERRUM_CHEM_SDF_FLAGS_NONE) {
		if (record_count > FERRUM_CHEM_SDF_MAX_RECORDS) {
			*failure_status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			*error = "SDF record count exceeds the ABI limit";
		} else {
			*error = "SDF request has no records or nonzero reserved flags";
		}
		return false;
	}
	Reader reader(request + FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES,
		request_len - FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES);
	if (static_cast<uint64_t>(record_count) > reader.remaining() /
		FERRUM_CHEM_SDF_RECORD_HEADER_BYTES) {
		*error = "SDF record headers are truncated";
		return false;
	}
	for (uint32_t index = 0; index < record_count; ++index) {
		if (!parse_record(&reader, index, output, error, failure_status)) return false;
	}
	if (reader.remaining() != 0) {
		*error = "SDF request has trailing bytes";
		return false;
	}
	return true;
}

}  // namespace

extern "C" uint32_t ferrum_chem_records_to_sdf_v1(
		const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		std::string output;
		std::string error;
		uint32_t failure_status = FERRUM_CHEM_RESULT_MALFORMED_REQUEST;
		if (!records_to_sdf(request, request_len, maximum_text_bytes, &output, &error,
				&failure_status)) {
			return ferrum_chem::emit_text_response(
				failure_status, error, "", response);
		}
		return ferrum_chem::emit_text_response(
			FERRUM_CHEM_RESULT_OK, "", output, response);
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
