#include "ferrum_chem_adapter.h"
#include "ferrum_chem_molecule_response.h"
#include "ferrum_chem_utf8.h"

#include <RDGeneral/RDConfig.h>
#include <GraphMol/Conformer.h>
#include <GraphMol/FileParsers/MolSupplier.h>
#include <GraphMol/ROMol.h>

#include <algorithm>
#include <cstdint>
#include <cstring>
#include <exception>
#include <limits>
#include <memory>
#include <new>
#include <string>
#include <string_view>
#include <utility>
#include <vector>

namespace {

constexpr uint8_t kResponseMagic[] = {'F', 'S', 'I', '1'};

struct ImportedRecord {
	std::vector<uint8_t> molecule;
	std::string title;
	std::vector<std::pair<std::string, std::string>> properties;
};

struct ImportFailure {
	uint32_t status = FERRUM_CHEM_RESULT_INVALID_MOLECULE;
	std::string detail;
};

std::string exception_detail(const std::exception &error) {
	constexpr std::string_view prefix = "native SDF supplier raised an exception: ";
	std::string detail(prefix);
	const char *source = error.what();
	while (source != nullptr && *source != '\0' &&
			detail.size() < FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES) {
		const auto byte = static_cast<unsigned char>(*source);
		detail.push_back(byte >= 0x20U && byte <= 0x7eU ? static_cast<char>(byte) : '?');
		++source;
	}
	return detail;
}

void append_u32(std::vector<uint8_t> &bytes, uint32_t value) {
	bytes.push_back(static_cast<uint8_t>(value));
	bytes.push_back(static_cast<uint8_t>(value >> 8U));
	bytes.push_back(static_cast<uint8_t>(value >> 16U));
	bytes.push_back(static_cast<uint8_t>(value >> 24U));
}

std::vector<std::string_view> lines(const std::string &text) {
	std::vector<std::string_view> result;
	size_t start = 0;
	while (start < text.size()) {
		const size_t end = text.find('\n', start);
		const size_t length = end == std::string::npos ? text.size() - start : end - start;
		std::string_view line(text.data() + start, length);
		if (!line.empty() && line.back() == '\r') line.remove_suffix(1);
		result.push_back(line);
		if (end == std::string::npos) break;
		start = end + 1;
	}
	return result;
}

bool property_name(std::string_view header, std::string *name) {
	if (header.empty() || header.front() != '>') return false;
	const size_t opening = header.find('<');
	const size_t closing = opening == std::string_view::npos ?
		std::string_view::npos : header.find('>', opening + 1);
	if (opening == std::string_view::npos || closing == std::string_view::npos ||
		closing == opening + 1) return false;
	name->assign(header.substr(opening + 1, closing - opening - 1));
	return name->find('\0') == std::string::npos &&
		name->find('\r') == std::string::npos && name->find('\n') == std::string::npos;
}

bool extract_record_text(
		const std::string &raw, std::string *title,
		std::vector<std::pair<std::string, std::string>> *properties,
		ImportFailure *failure) {
	if (!ferrum_chem::is_valid_utf8(raw) || raw.find('\0') != std::string::npos) {
		failure->detail = "SDF record text is not UTF-8 without NUL bytes";
		return false;
	}
	const std::vector<std::string_view> record_lines = lines(raw);
	if (record_lines.empty()) {
		failure->detail = "SDF record is empty";
		return false;
	}
	title->assign(record_lines.front());
	size_t index = 1;
	while (index < record_lines.size() && record_lines[index] != "M  END") ++index;
	if (index == record_lines.size()) {
		failure->detail = "SDF record has no molblock terminator";
		return false;
	}
	++index;
	while (index < record_lines.size()) {
		while (index < record_lines.size() && record_lines[index].empty()) ++index;
		if (index == record_lines.size() || record_lines[index] == "$$$$") break;
		std::string name;
		if (!property_name(record_lines[index], &name)) {
			failure->detail = "SDF property header is malformed";
			return false;
		}
		++index;
		std::string value;
		while (index < record_lines.size() && !record_lines[index].empty() &&
				record_lines[index] != "$$$$") {
			if (!value.empty()) value.push_back('\n');
			value.append(record_lines[index]);
			++index;
		}
		properties->emplace_back(std::move(name), std::move(value));
		if (properties->size() > FERRUM_CHEM_SDF_MAX_PROPERTIES) {
			failure->status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			failure->detail = "SDF property count exceeds the ABI limit";
			return false;
		}
	}
	return true;
}

bool import_records(
		const uint8_t *request, uint64_t request_len,
		std::vector<ImportedRecord> *records, ImportFailure *failure) {
	if (request == nullptr || request_len == 0 ||
		request_len > FERRUM_CHEM_MAX_RESPONSE_BYTES) {
		failure->status = FERRUM_CHEM_RESULT_MALFORMED_REQUEST;
		failure->detail = "SDF input is empty or exceeds the ABI input bound";
		return false;
	}
	const std::string input(reinterpret_cast<const char *>(request),
		static_cast<size_t>(request_len));
	if (input.find('\0') != std::string::npos || !ferrum_chem::is_valid_utf8(input)) {
		failure->status = FERRUM_CHEM_RESULT_MALFORMED_REQUEST;
		failure->detail = "SDF input must be UTF-8 without NUL bytes";
		return false;
	}
	RDKit::SDMolSupplier supplier;
	supplier.setData(input, true, false, true);
	const unsigned int record_count = supplier.length();
	if (record_count == 0) {
		failure->detail = "SDF input contains no molecule records";
		return false;
	}
	if (record_count > FERRUM_CHEM_SDF_MAX_RECORDS) {
		failure->status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
		failure->detail = "SDF record count exceeds the ABI limit";
		return false;
	}
	records->reserve(record_count);
	uint64_t total_properties = 0;
	for (unsigned int index = 0; index < record_count; ++index) {
		std::unique_ptr<RDKit::ROMol> molecule(supplier[index]);
		if (!molecule || molecule->getNumConformers() != 1) {
			failure->detail = "SDF record is invalid or lacks exactly one conformer";
			return false;
		}
		const RDKit::Conformer &conformer = molecule->getConformer();
		if (conformer.is3D()) {
			failure->status = FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE;
			failure->detail = "three-dimensional SDF coordinates are not representable by Point2";
			return false;
		}
		ImportedRecord record;
		if (!extract_record_text(
				supplier.getItemText(index), &record.title, &record.properties,
				failure)) return false;
		total_properties += record.properties.size();
		if (total_properties > FERRUM_CHEM_SDF_MAX_PROPERTIES) {
			failure->status = FERRUM_CHEM_RESULT_RESOURCE_LIMIT;
			failure->detail = "total SDF property count exceeds the ABI limit";
			return false;
		}
		if (!ferrum_chem::encode_molecule_response_bytes(
				FERRUM_CHEM_RESULT_OK, "", molecule.get(), &conformer,
				&record.molecule)) {
			failure->status = FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE;
			failure->detail = "SDF molecule cannot be represented by FCM1";
			return false;
		}
		records->push_back(std::move(record));
	}
	return true;
}

bool response_size(
		const std::vector<ImportedRecord> &records, size_t detail_length,
		uint64_t *total) {
	*total = FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES + detail_length;
	for (const ImportedRecord &record : records) {
		uint64_t record_size = FERRUM_CHEM_SDF_RECORD_HEADER_BYTES +
			record.molecule.size() + record.title.size();
		if (record.molecule.size() > UINT32_MAX || record.title.size() > UINT32_MAX ||
			record.properties.size() > UINT32_MAX) return false;
		for (const auto &[name, value] : record.properties) {
			if (name.size() > UINT32_MAX || value.size() > UINT32_MAX) return false;
			const uint64_t property_size = FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES +
				name.size() + value.size();
			if (property_size > FERRUM_CHEM_MAX_RESPONSE_BYTES - record_size) return false;
			record_size += property_size;
		}
		if (record_size > FERRUM_CHEM_MAX_RESPONSE_BYTES - *total) return false;
		*total += record_size;
	}
	return *total <= FERRUM_CHEM_MAX_RESPONSE_BYTES &&
		*total <= std::numeric_limits<size_t>::max();
}

uint32_t emit_response(
		uint32_t status, std::string_view detail,
		const std::vector<ImportedRecord> &records,
		ferrum_chem_owned_buffer *response) {
	if (response == nullptr || detail.size() > FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES ||
		(status == FERRUM_CHEM_RESULT_OK && (!detail.empty() || records.empty())) ||
		(status != FERRUM_CHEM_RESULT_OK && (detail.empty() || !records.empty()))) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	}
	uint64_t total = 0;
	if (!response_size(records, detail.size(), &total)) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	}
	try {
		std::vector<uint8_t> bytes;
		bytes.reserve(static_cast<size_t>(total));
		bytes.insert(bytes.end(), std::begin(kResponseMagic), std::end(kResponseMagic));
		append_u32(bytes, FERRUM_CHEM_SDF_WIRE_VERSION);
		append_u32(bytes, status);
		append_u32(bytes, static_cast<uint32_t>(detail.size()));
		append_u32(bytes, static_cast<uint32_t>(records.size()));
		append_u32(bytes, FERRUM_CHEM_SDF_FLAGS_NONE);
		bytes.insert(bytes.end(), detail.begin(), detail.end());
		for (const ImportedRecord &record : records) {
			append_u32(bytes, static_cast<uint32_t>(record.molecule.size()));
			append_u32(bytes, static_cast<uint32_t>(record.title.size()));
			append_u32(bytes, static_cast<uint32_t>(record.properties.size()));
			append_u32(bytes, FERRUM_CHEM_SDF_FLAGS_NONE);
			bytes.insert(bytes.end(), record.molecule.begin(), record.molecule.end());
			bytes.insert(bytes.end(), record.title.begin(), record.title.end());
			for (const auto &[name, value] : record.properties) {
				append_u32(bytes, static_cast<uint32_t>(name.size()));
				append_u32(bytes, static_cast<uint32_t>(value.size()));
				bytes.insert(bytes.end(), name.begin(), name.end());
				bytes.insert(bytes.end(), value.begin(), value.end());
			}
		}
		if (bytes.size() != total) return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		response->data = new uint8_t[bytes.size()];
		std::copy(bytes.begin(), bytes.end(), response->data);
		response->len = bytes.size();
		return FERRUM_CHEM_CALL_OK;
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (...) {
		return FERRUM_CHEM_CALL_INTERNAL_FAILURE;
	}
}

}  // namespace

extern "C" uint32_t ferrum_chem_sdf_to_records_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		std::vector<ImportedRecord> records;
		ImportFailure failure;
		if (!import_records(request, request_len, &records, &failure)) {
			return emit_response(failure.status, failure.detail, {}, response);
		}
		return emit_response(FERRUM_CHEM_RESULT_OK, "", records, response);
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return emit_response(
			FERRUM_CHEM_RESULT_INVALID_MOLECULE,
			exception_detail(error), {}, response);
	} catch (...) {
		return emit_response(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", {}, response);
	}
}
