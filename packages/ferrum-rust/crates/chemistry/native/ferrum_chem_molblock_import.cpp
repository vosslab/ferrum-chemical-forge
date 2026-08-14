#include "ferrum_chem_adapter.h"
#include "ferrum_chem_molecule_response.h"
#include "ferrum_chem_utf8.h"

#include <RDGeneral/RDConfig.h>
#include <GraphMol/Conformer.h>
#include <GraphMol/FileParsers/FileParsers.h>
#include <GraphMol/RWMol.h>

#include <cstdint>
#include <exception>
#include <memory>
#include <new>
#include <string>
#include <string_view>
#include <vector>

namespace {

struct ImportFailure {
	uint32_t status = FERRUM_CHEM_RESULT_MALFORMED_REQUEST;
	std::string detail;
};

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

bool is_ascii_space(std::string_view text) {
	for (const char byte : text) {
		if (byte != ' ' && byte != '\t' && byte != '\r') return false;
	}
	return true;
}

bool validate_envelope(const std::string &input, ImportFailure *failure) {
	if (input.find("$$$$") != std::string::npos) {
		failure->detail = "molblock input must not contain an SDF record delimiter";
		return false;
	}
	const std::vector<std::string_view> input_lines = lines(input);
	if (input_lines.size() < 5) {
		failure->detail = "molblock input is missing its counts line or terminator";
		return false;
	}
	const std::string_view counts = input_lines[3];
	if (!counts.ends_with("V2000") && !counts.ends_with("V3000")) {
		failure->detail = "molblock counts line must explicitly select V2000 or V3000";
		return false;
	}
	size_t terminator = input_lines.size();
	for (size_t index = 0; index < input_lines.size(); ++index) {
		if (input_lines[index] != "M  END") continue;
		if (terminator != input_lines.size()) {
			failure->detail = "molblock input has more than one M  END terminator";
			return false;
		}
		terminator = index;
	}
	if (terminator == input_lines.size()) {
		failure->detail = "molblock input has no M  END terminator";
		return false;
	}
	for (size_t index = terminator + 1; index < input_lines.size(); ++index) {
		if (!is_ascii_space(input_lines[index])) {
			failure->detail = "molblock input has data after M  END";
			return false;
		}
	}
	return true;
}

std::string exception_detail(const std::exception &error) {
	constexpr std::string_view prefix = "native molblock parser raised an exception: ";
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

bool emit_failure(const ImportFailure &failure, ferrum_chem_owned_buffer *response) {
	return ferrum_chem::emit_molecule_response(
		failure.status, failure.detail, nullptr, nullptr, response);
}

}  // namespace

extern "C" uint32_t ferrum_chem_molblock_to_molecule_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		ImportFailure failure;
		if (request == nullptr || request_len == 0 ||
				request_len > FERRUM_CHEM_MAX_RESPONSE_BYTES) {
			failure.detail = "molblock input is empty or exceeds the ABI input bound";
			return emit_failure(failure, response) ? FERRUM_CHEM_CALL_OK :
				FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		}
		const std::string input(
			reinterpret_cast<const char *>(request), static_cast<size_t>(request_len));
		if (input.find('\0') != std::string::npos || !ferrum_chem::is_valid_utf8(input)) {
			failure.detail = "molblock input must be UTF-8 without NUL bytes";
			return emit_failure(failure, response) ? FERRUM_CHEM_CALL_OK :
				FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		}
		if (!validate_envelope(input, &failure)) {
			return emit_failure(failure, response) ? FERRUM_CHEM_CALL_OK :
				FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		}

		RDKit::v2::FileParsers::MolFileParserParams parameters;
		parameters.sanitize = true;
		parameters.removeHs = false;
		parameters.strictParsing = true;
		std::unique_ptr<RDKit::RWMol> molecule =
			RDKit::v2::FileParsers::MolFromMolBlock(input, parameters);
		if (!molecule || molecule->getNumAtoms() == 0 || molecule->getNumConformers() != 1) {
			failure.status = FERRUM_CHEM_RESULT_INVALID_MOLECULE;
			failure.detail = "molblock is invalid or lacks exactly one nonempty conformer";
			return emit_failure(failure, response) ? FERRUM_CHEM_CALL_OK :
				FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		}
		const RDKit::Conformer &conformer = molecule->getConformer();
		if (conformer.is3D()) {
			failure.status = FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE;
			failure.detail = "three-dimensional molblock coordinates are not representable by Point2";
			return emit_failure(failure, response) ? FERRUM_CHEM_CALL_OK :
				FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
		}
		return ferrum_chem::emit_molecule_response(
			FERRUM_CHEM_RESULT_OK, "", molecule.get(), &conformer, response) ?
			FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return ferrum_chem::emit_molecule_response(
			FERRUM_CHEM_RESULT_INVALID_MOLECULE, exception_detail(error),
			nullptr, nullptr, response) ? FERRUM_CHEM_CALL_OK :
			FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (...) {
		return ferrum_chem::emit_molecule_response(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure",
			nullptr, nullptr, response) ? FERRUM_CHEM_CALL_OK :
			FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	}
}
