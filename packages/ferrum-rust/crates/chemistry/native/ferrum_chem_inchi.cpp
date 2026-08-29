#include "ferrum_chem_adapter.h"
#include "ferrum_chem_complete_graph.h"
#include "ferrum_chem_molecule_response.h"
#include "ferrum_chem_text_response.h"
#include "ferrum_chem_text_output_limit.h"
#include "ferrum_chem_writer_probe.h"

#include <GraphMol/Depictor/RDDepictor.h>
#include <GraphMol/RWMol.h>
#include <GraphMol/inchi.h>

#include <algorithm>
#include <cstdint>
#include <cstring>
#include <exception>
#include <memory>
#include <new>
#include <string>
#include <string_view>

namespace {

constexpr uint8_t kInchiRequestMagic[] = {'F', 'C', 'I', '1'};

uint32_t read_u32(const uint8_t *bytes) {
	return static_cast<uint32_t>(bytes[0]) |
		(static_cast<uint32_t>(bytes[1]) << 8U) |
		(static_cast<uint32_t>(bytes[2]) << 16U) |
		(static_cast<uint32_t>(bytes[3]) << 24U);
}

std::string bounded_detail(std::string_view source, std::string_view fallback) {
	if (source.empty()) source = fallback;
	std::string detail;
	detail.reserve(std::min(source.size(), static_cast<size_t>(
		FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES)));
	for (const unsigned char byte : source) {
		if (detail.size() == FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES) break;
		detail.push_back(byte >= 0x20U && byte <= 0x7eU ? static_cast<char>(byte) : '?');
	}
	return detail;
}

bool valid_inchi_line(const std::string &input) {
	const bool valid_prefix = input.starts_with("InChI=1S/") || input.starts_with("InChI=1/");
	return valid_prefix && std::all_of(input.begin(), input.end(), [](unsigned char byte) {
		return byte >= 0x21U && byte <= 0x7eU;
	});
}

bool valid_inchi_key(const std::string &inchi, const std::string &key) {
	if (key.size() != FERRUM_CHEM_INCHI_KEY_BYTES || key[14] != '-' || key[25] != '-') {
		return false;
	}
	for (size_t index = 0; index < key.size(); ++index) {
		if (index == 14 || index == 25) continue;
		if (key[index] < 'A' || key[index] > 'Z') return false;
	}
	const char expected_kind = inchi.starts_with("InChI=1S/") ? 'S' : 'N';
	return key[23] == expected_kind && key[24] == 'A';
}

uint32_t emit_import_failure(uint32_t status, std::string_view detail,
		ferrum_chem_owned_buffer *response) {
	return ferrum_chem::emit_molecule_response(
		status, detail, nullptr, nullptr, response) ? FERRUM_CHEM_CALL_OK :
		FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
}

bool parse_export_request(const uint8_t *request, uint64_t request_len,
		RDKit::RWMol *molecule, uint32_t *mode, std::string *error) {
	if (request == nullptr || request_len < FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES) {
		*error = "InChI export request is missing or truncated";
		return false;
	}
	if (std::memcmp(request, kInchiRequestMagic, sizeof(kInchiRequestMagic)) != 0 ||
			read_u32(request + 4) != FERRUM_CHEM_INCHI_WIRE_VERSION) {
		*error = "InChI export request has invalid magic or version";
		return false;
	}
	*mode = read_u32(request + 8);
	const uint32_t graph_length = read_u32(request + 12);
	if ((*mode != FERRUM_CHEM_INCHI_MODE_STANDARD &&
			*mode != FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN) ||
			read_u32(request + 16) != FERRUM_CHEM_INCHI_FLAGS_NONE) {
		*error = "InChI export request has an unsupported mode or reserved flags";
		return false;
	}
	if (graph_length > FERRUM_CHEM_MAX_RESPONSE_BYTES ||
			request_len != FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES +
				static_cast<uint64_t>(graph_length)) {
		*error = "InChI export graph is oversized, truncated, or trailing";
		return false;
	}
	return ferrum_chem::parse_complete_graph(
		request + FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES,
		graph_length, molecule, error);
}

}  // namespace

extern "C" uint32_t ferrum_chem_inchi_to_molecule_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		if (request == nullptr || request_len == 0 ||
				request_len > FERRUM_CHEM_INCHI_MAX_BYTES) {
			return emit_import_failure(FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
				"InChI input is empty or exceeds the ABI input bound", response);
		}
		const std::string input(
			reinterpret_cast<const char *>(request), static_cast<size_t>(request_len));
		if (!valid_inchi_line(input)) {
			return emit_import_failure(FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
				"InChI input must be one ASCII InChI=1S/ or InChI=1/ line", response);
		}
		RDKit::ExtraInchiReturnValues inchi_result;
		std::unique_ptr<RDKit::RWMol> molecule(
			RDKit::InchiToMol(input, inchi_result, true, true));
		if (!molecule || molecule->getNumAtoms() == 0) {
			return emit_import_failure(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
				bounded_detail(inchi_result.messagePtr, "RDKit could not parse InChI"), response);
		}
		RDDepict::Compute2DCoordParameters parameters;
		parameters.canonOrient = true;
		parameters.clearConfs = true;
		parameters.forceRDKit = true;
		parameters.nFlipsPerSample = 0;
		parameters.nSamples = 0;
		parameters.useRingTemplates = false;
		const unsigned int conformer_id = RDDepict::compute2DCoords(*molecule, parameters);
		return ferrum_chem::emit_molecule_response(
			FERRUM_CHEM_RESULT_OK, "", molecule.get(),
			&molecule->getConformer(conformer_id), response) ? FERRUM_CHEM_CALL_OK :
			FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return emit_import_failure(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
			bounded_detail(error.what(), "native InChI import failed"), response);
	} catch (...) {
		return emit_import_failure(FERRUM_CHEM_RESULT_INTERNAL_FAILURE,
			"unknown native InChI import failure", response);
	}
}

extern "C" uint32_t ferrum_chem_molecule_to_inchi_v1(
		const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		RDKit::RWMol molecule;
		uint32_t mode = 0;
		std::string error;
		if (!parse_export_request(request, request_len, &molecule, &mode, &error)) {
			return ferrum_chem::emit_text_response(
				FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, "", response);
		}
		if (!ferrum_chem::text_output_is_admitted(
				ferrum_chem::inchi_text_upper_bound(
					molecule.getNumAtoms(), molecule.getNumBonds()), maximum_text_bytes)) {
			return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_RESOURCE_LIMIT,
				"InChI upper bound exceeds the requested text limit", "", response);
		}
		RDKit::ExtraInchiReturnValues inchi_result;
		const char *options = mode == FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN ?
			"-FixedH" : nullptr;
		ferrum_chem::record_native_text_writer_invocation(
			ferrum_chem::NativeTextWriter::Inchi);
		const std::string output = RDKit::MolToInchi(molecule, inchi_result, options);
		const bool prefix_matches = mode == FERRUM_CHEM_INCHI_MODE_STANDARD ?
			output.starts_with("InChI=1S/") :
			output.starts_with("InChI=1/") && !output.starts_with("InChI=1S/");
		if (output.empty() || !prefix_matches) {
			return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
				bounded_detail(inchi_result.messagePtr,
					"RDKit could not generate the requested InChI mode"), "", response);
		}
		return ferrum_chem::emit_text_response(
			FERRUM_CHEM_RESULT_OK, "", output, response);
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
			bounded_detail(error.what(), "native InChI export failed"), "", response);
	} catch (...) {
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INTERNAL_FAILURE,
			"unknown native InChI export failure", "", response);
	}
}

extern "C" uint32_t ferrum_chem_inchi_to_inchi_key_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		if (request == nullptr || request_len == 0 ||
				request_len > FERRUM_CHEM_INCHI_MAX_BYTES) {
			return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
				"InChI input is empty or exceeds the ABI input bound", "", response);
		}
		const std::string input(
			reinterpret_cast<const char *>(request), static_cast<size_t>(request_len));
		if (!valid_inchi_line(input)) {
			return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_MALFORMED_REQUEST,
				"InChI input must be one ASCII InChI=1S/ or InChI=1/ line", "", response);
		}
		const std::string key = RDKit::InchiToInchiKey(input);
		if (!valid_inchi_key(input, key)) {
			return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
				"RDKit could not generate a valid InChIKey", "", response);
		}
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_OK, "", key, response);
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::exception &error) {
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
			bounded_detail(error.what(), "native InChIKey generation failed"), "", response);
	} catch (...) {
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INTERNAL_FAILURE,
			"unknown native InChIKey generation failure", "", response);
	}
}
