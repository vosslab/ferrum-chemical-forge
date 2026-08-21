#include "ferrum_chem_smarts_match.h"

#include "ferrum_chem_complete_graph.h"
#include "ferrum_chem_utf8.h"

#include <GraphMol/RDKitBase.h>
#include <GraphMol/RWMol.h>
#include <GraphMol/SmilesParse/SmilesParse.h>
#include <GraphMol/Substruct/SubstructMatch.h>

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <cstring>
#include <limits>
#include <memory>
#include <new>
#include <string>
#include <string_view>
#include <unordered_set>
#include <utility>
#include <vector>

namespace {

constexpr uint8_t kRequestMagic[] = {'F', 'C', 'Q', '1'};
constexpr uint8_t kResponseMagic[] = {'F', 'Q', 'M', '1'};
constexpr uint32_t kWireVersion = FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION;
constexpr uint32_t kRequestHeaderBytes = FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES;
constexpr uint32_t kResponseHeaderBytes = FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES;
constexpr uint32_t kMaximumQueryBytes = FERRUM_CHEM_SMARTS_MATCH_MAX_QUERY_BYTES;
constexpr uint32_t kMaximumParsedQueryAtoms = FERRUM_CHEM_SMARTS_MATCH_MAX_PARSED_QUERY_ATOMS;
constexpr uint32_t kProfileMaximumQueryAtoms = FERRUM_CHEM_SMARTS_MATCH_PROFILE_MAX_QUERY_ATOMS;
constexpr uint32_t kMaximumRows = FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS;
constexpr uint32_t kMaximumMatrixCells = FERRUM_CHEM_SMARTS_MATCH_MAX_MATRIX_CELLS;
constexpr uint32_t kTruncatedFlag = FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED;

constexpr std::string_view kInvalidRequest = "invalid_request";
constexpr std::string_view kQueryAtomLimitExceeded = "query_atom_limit_exceeded";
constexpr std::string_view kInvalidQuery = "invalid_query";
constexpr std::string_view kUnsupportedTarget = "unsupported_target";
constexpr std::string_view kResourceLimited = "resource_limited";
constexpr std::string_view kNativeFailure = "native_failure";

struct Request {
	std::string smarts;
	const uint8_t *graph_bytes;
	uint32_t graph_len;
	uint32_t max_matches;
};

uint32_t read_u32(const uint8_t *bytes) {
	return static_cast<uint32_t>(bytes[0]) |
		(static_cast<uint32_t>(bytes[1]) << 8U) |
		(static_cast<uint32_t>(bytes[2]) << 16U) |
		(static_cast<uint32_t>(bytes[3]) << 24U);
}

void append_u32(std::vector<uint8_t> *bytes, uint32_t value) {
	bytes->push_back(static_cast<uint8_t>(value));
	bytes->push_back(static_cast<uint8_t>(value >> 8U));
	bytes->push_back(static_cast<uint8_t>(value >> 16U));
	bytes->push_back(static_cast<uint8_t>(value >> 24U));
}

bool response_size_is_valid(uint32_t query_atom_count, uint32_t match_count) {
	if (query_atom_count > 0 && match_count > kMaximumMatrixCells / query_atom_count) {
		return false;
	}
	const uint64_t cells = static_cast<uint64_t>(query_atom_count) * match_count;
	return cells <= (std::numeric_limits<size_t>::max() - kResponseHeaderBytes) / sizeof(uint32_t) &&
		kResponseHeaderBytes + cells * sizeof(uint32_t) <= FERRUM_CHEM_MAX_RESPONSE_BYTES;
}

bool emit_response(uint32_t status, std::string_view detail, uint32_t query_atom_count,
		const std::vector<RDKit::MatchVectType> *matches, bool truncated,
		ferrum_chem_owned_buffer *response) {
	const uint32_t match_count = matches == nullptr ? 0U : static_cast<uint32_t>(matches->size());
	if (detail.size() > std::numeric_limits<uint32_t>::max() ||
		!response_size_is_valid(query_atom_count, match_count)) {
		return false;
	}
	const uint64_t matrix_bytes = static_cast<uint64_t>(query_atom_count) * match_count * sizeof(uint32_t);
	const uint64_t response_len = kResponseHeaderBytes + detail.size() + matrix_bytes;
	if (response_len > FERRUM_CHEM_MAX_RESPONSE_BYTES || response_len > std::numeric_limits<size_t>::max()) {
		return false;
	}
	try {
		std::vector<uint8_t> bytes;
		bytes.reserve(static_cast<size_t>(response_len));
		bytes.insert(bytes.end(), std::begin(kResponseMagic), std::end(kResponseMagic));
		append_u32(&bytes, kWireVersion);
		append_u32(&bytes, status);
		append_u32(&bytes, static_cast<uint32_t>(detail.size()));
		append_u32(&bytes, query_atom_count);
		append_u32(&bytes, match_count);
		append_u32(&bytes, truncated ? kTruncatedFlag : 0U);
		bytes.insert(bytes.end(), detail.begin(), detail.end());
		if (matches != nullptr) {
			for (const RDKit::MatchVectType &match : *matches) {
				if (match.size() != query_atom_count) return false;
				for (const auto &[query_index, target_index] : match) {
					(void)query_index;
					append_u32(&bytes, target_index);
				}
			}
		}
		uint8_t *owned = new uint8_t[bytes.size()];
		std::memcpy(owned, bytes.data(), bytes.size());
		response->data = owned;
		response->len = bytes.size();
		return true;
	} catch (const std::bad_alloc &) {
		return false;
	}
}

uint32_t emit_error(uint32_t status, std::string_view detail, ferrum_chem_owned_buffer *response) {
	return emit_response(status, detail, 0U, nullptr, false, response) ?
		FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
}

bool parse_request(const uint8_t *request, uint64_t request_len, Request *parsed) {
	if (request == nullptr || request_len < kRequestHeaderBytes ||
		std::memcmp(request, kRequestMagic, sizeof(kRequestMagic)) != 0 ||
		read_u32(request + 4) != kWireVersion) {
		return false;
	}
	const uint32_t smarts_len = read_u32(request + 8);
	const uint32_t graph_len = read_u32(request + 12);
	const uint32_t max_matches = read_u32(request + 16);
	const uint32_t flags = read_u32(request + 20);
	if (smarts_len == 0 || smarts_len > kMaximumQueryBytes || graph_len == 0 ||
		max_matches == 0 || max_matches > kMaximumRows || flags != 0 ||
		static_cast<uint64_t>(smarts_len) + graph_len != request_len - kRequestHeaderBytes) {
		return false;
	}
	const char *smarts_start = reinterpret_cast<const char *>(request + kRequestHeaderBytes);
	parsed->smarts.assign(smarts_start, smarts_len);
	if (parsed->smarts.find('\0') != std::string::npos || !ferrum_chem::is_valid_utf8(parsed->smarts)) {
		return false;
	}
	parsed->graph_bytes = request + kRequestHeaderBytes + smarts_len;
	parsed->graph_len = graph_len;
	parsed->max_matches = max_matches;
	return true;
}

bool match_is_lexicographically_less(const RDKit::MatchVectType &left,
		const RDKit::MatchVectType &right) {
	for (size_t index = 0; index < left.size(); ++index) {
		if (left[index].second != right[index].second) return left[index].second < right[index].second;
	}
	return false;
}

bool normalize_matches(std::vector<RDKit::MatchVectType> *matches,
		uint32_t query_atom_count, uint32_t target_atom_count) {
	for (RDKit::MatchVectType &match : *matches) {
		if (match.size() != query_atom_count) return false;
		std::sort(match.begin(), match.end(), [](const auto &left, const auto &right) {
			return left.first < right.first;
		});
		std::unordered_set<int> target_indexes;
		for (uint32_t query_index = 0; query_index < query_atom_count; ++query_index) {
			const auto &[native_query_index, native_target_index] = match[query_index];
			if (native_query_index != static_cast<int>(query_index) || native_target_index < 0 ||
				static_cast<uint32_t>(native_target_index) >= target_atom_count ||
				!target_indexes.insert(native_target_index).second) {
				return false;
			}
		}
	}
	return true;
}

}  // namespace

uint32_t ferrum_chem::smarts_match_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		Request parsed;
		if (!parse_request(request, request_len, &parsed)) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, kInvalidRequest, response);
		}

		std::unique_ptr<RDKit::ROMol> query(RDKit::SmartsToMol(parsed.smarts));
		if (!query || query->getNumAtoms() == 0) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY, kInvalidQuery, response);
		}
		const uint32_t query_atoms = query->getNumAtoms();
		if (query_atoms > kMaximumParsedQueryAtoms || query_atoms > kProfileMaximumQueryAtoms) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
				kQueryAtomLimitExceeded, response);
		}

		RDKit::RWMol target;
		std::string graph_error;
		if (!parse_complete_graph(parsed.graph_bytes, parsed.graph_len, &target, &graph_error)) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, kInvalidRequest, response);
		}
		if (target.getNumAtoms() == 0) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET,
				kUnsupportedTarget, response);
		}

		const uint64_t requested_rows = static_cast<uint64_t>(parsed.max_matches) + 1U;
		if (!response_size_is_valid(query_atoms, parsed.max_matches) ||
			requested_rows > kMaximumRows + 1ULL ||
			query_atoms > 0 && requested_rows > kMaximumMatrixCells / query_atoms) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED,
				kResourceLimited, response);
		}
		RDKit::SubstructMatchParameters parameters;
		parameters.useChirality = true;
		parameters.uniquify = true;
		parameters.maxMatches = static_cast<unsigned int>(requested_rows);
		std::vector<RDKit::MatchVectType> matches = RDKit::SubstructMatch(target, *query, parameters);
		if (!normalize_matches(&matches, query_atoms, target.getNumAtoms())) {
			return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE, kNativeFailure, response);
		}
		const bool truncated = matches.size() > parsed.max_matches;
		if (truncated) matches.resize(parsed.max_matches);
		std::sort(matches.begin(), matches.end(), match_is_lexicographically_less);
		return emit_response(FERRUM_CHEM_SMARTS_MATCH_STATUS_OK, "", query_atoms, &matches, truncated, response) ?
			FERRUM_CHEM_CALL_OK : FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const std::bad_alloc &) {
		return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED, kResourceLimited, response);
	} catch (...) {
		return emit_error(FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE, kNativeFailure, response);
	}
}
