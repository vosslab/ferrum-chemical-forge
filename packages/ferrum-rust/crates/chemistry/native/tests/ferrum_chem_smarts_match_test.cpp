#include "ferrum_chem_adapter.h"

#include <algorithm>
#include <cassert>
#include <cstdint>
#include <cstring>
#include <string>
#include <string_view>
#include <vector>

namespace {

void append_u16(std::vector<uint8_t> *bytes, uint16_t value) {
	bytes->push_back(static_cast<uint8_t>(value));
	bytes->push_back(static_cast<uint8_t>(value >> 8U));
}

void append_u32(std::vector<uint8_t> *bytes, uint32_t value) {
	bytes->push_back(static_cast<uint8_t>(value));
	bytes->push_back(static_cast<uint8_t>(value >> 8U));
	bytes->push_back(static_cast<uint8_t>(value >> 16U));
	bytes->push_back(static_cast<uint8_t>(value >> 24U));
}

void put_u32(std::vector<uint8_t> *bytes, size_t offset, uint32_t value) {
	assert(offset + 4 <= bytes->size());
	(*bytes)[offset] = static_cast<uint8_t>(value);
	(*bytes)[offset + 1] = static_cast<uint8_t>(value >> 8U);
	(*bytes)[offset + 2] = static_cast<uint8_t>(value >> 16U);
	(*bytes)[offset + 3] = static_cast<uint8_t>(value >> 24U);
}

struct Atom {
	uint8_t atomic_number;
	bool aromatic = false;
	uint8_t chirality = FERRUM_CHEM_CHIRAL_UNSPECIFIED;
	uint32_t presence = 0;
	int32_t charge = 0;
	uint16_t isotope = 0;
	uint16_t hydrogens = 0;
	bool no_implicit = false;
};

struct Bond {
	uint32_t start;
	uint32_t end;
	uint8_t order = FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE;
	bool aromatic = false;
};

void append_atom(std::vector<uint8_t> *graph, const Atom &atom) {
	graph->push_back(atom.atomic_number);
	graph->push_back(atom.aromatic ? 1 : 0);
	graph->push_back(atom.chirality);
	graph->push_back(0);
	append_u32(graph, atom.presence);
	append_u32(graph, static_cast<uint32_t>(atom.charge));
	append_u16(graph, atom.isotope);
	append_u16(graph, atom.hydrogens);
	graph->push_back(0);
	graph->push_back(atom.no_implicit ? 1 : 0);
	append_u16(graph, 0);
	append_u32(graph, 0);
}

void append_bond(std::vector<uint8_t> *graph, const Bond &bond) {
	append_u32(graph, bond.start);
	append_u32(graph, bond.end);
	graph->push_back(bond.order);
	graph->push_back(bond.aromatic ? 1 : 0);
	graph->push_back(FERRUM_CHEM_BOND_STEREO_NONE);
	graph->push_back(FERRUM_CHEM_BOND_DIRECTION_NONE);
	append_u32(graph, FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE);
	append_u32(graph, FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE);
	append_u32(graph, 0);
}

std::vector<uint8_t> fcg1(const std::vector<Atom> &atoms, const std::vector<Bond> &bonds = {}) {
	std::vector<uint8_t> graph{'F', 'C', 'G', '1'};
	append_u32(&graph, 1);
	append_u32(&graph, static_cast<uint32_t>(atoms.size()));
	append_u32(&graph, static_cast<uint32_t>(bonds.size()));
	append_u32(&graph, 0);
	for (const Atom &atom : atoms) append_atom(&graph, atom);
	for (const Bond &bond : bonds) append_bond(&graph, bond);
	return graph;
}

std::vector<uint8_t> carbon_fcg1(uint32_t atom_count) {
	return fcg1(std::vector<Atom>(atom_count, Atom{6}));
}

std::vector<uint8_t> fcq1_bytes(std::string_view smarts, const std::vector<uint8_t> &graph,
		uint32_t cap = 10, uint32_t flags = 0) {
	std::vector<uint8_t> request{'F', 'C', 'Q', '1'};
	append_u32(&request, 1); append_u32(&request, static_cast<uint32_t>(smarts.size()));
	append_u32(&request, static_cast<uint32_t>(graph.size())); append_u32(&request, cap); append_u32(&request, flags);
	request.insert(request.end(), smarts.begin(), smarts.end()); request.insert(request.end(), graph.begin(), graph.end());
	return request;
}

std::vector<uint8_t> fcq1(const char *smarts, uint32_t cap = 10, uint32_t target_atoms = 1) {
	return fcq1_bytes(smarts, carbon_fcg1(target_atoms), cap);
}

uint32_t u32_at(const ferrum_chem_owned_buffer &buffer, size_t offset) {
	return static_cast<uint32_t>(buffer.data[offset]) |
		(static_cast<uint32_t>(buffer.data[offset + 1]) << 8U) |
		(static_cast<uint32_t>(buffer.data[offset + 2]) << 16U) |
		(static_cast<uint32_t>(buffer.data[offset + 3]) << 24U);
}

void expect_status(const std::vector<uint8_t> &request, uint32_t status, const char *detail) {
	ferrum_chem_owned_buffer response{nullptr, 0};
	assert(ferrum_chem_smarts_match_v1(request.data(), request.size(), &response) == FERRUM_CHEM_CALL_OK);
	const uint32_t detail_len = static_cast<uint32_t>(std::strlen(detail));
	assert(response.data != nullptr && response.len ==
		FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES + detail_len &&
		std::memcmp(response.data, "FQM1", 4) == 0);
	assert(u32_at(response, 4) == FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION);
	assert(u32_at(response, 8) == status);
	assert(u32_at(response, 12) == detail_len);
	assert(status != FERRUM_CHEM_SMARTS_MATCH_STATUS_OK);
	assert(u32_at(response, 16) == 0 && u32_at(response, 20) == 0 && u32_at(response, 24) == 0);
	assert(std::memcmp(response.data + FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES, detail, detail_len) == 0);
	ferrum_chem_owned_buffer_free_v1(&response);
}

void expect_match(const std::vector<uint8_t> &request, uint32_t query_atoms, uint32_t match_count,
		const std::vector<uint32_t> &expected_target_indexes = {}) {
	ferrum_chem_owned_buffer response{nullptr, 0};
	assert(ferrum_chem_smarts_match_v1(request.data(), request.size(), &response) == FERRUM_CHEM_CALL_OK);
	assert(response.data != nullptr && response.len ==
		FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES + query_atoms * match_count * sizeof(uint32_t));
	assert(std::memcmp(response.data, "FQM1", 4) == 0);
	assert(u32_at(response, 4) == FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION);
	assert(u32_at(response, 8) == FERRUM_CHEM_SMARTS_MATCH_STATUS_OK);
	assert(u32_at(response, 12) == 0 && u32_at(response, 16) == query_atoms);
	assert(u32_at(response, 20) == match_count);
	assert(u32_at(response, 24) == 0);
	assert(expected_target_indexes.empty() || expected_target_indexes.size() == query_atoms * match_count);
	for (size_t index = 0; index < expected_target_indexes.size(); ++index) {
		assert(u32_at(response, FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES + index * sizeof(uint32_t)) ==
			expected_target_indexes[index]);
	}
	ferrum_chem_owned_buffer_free_v1(&response);
}

void expect_hostile_detail_is_redacted() {
	const char *hostile = "/private/ferrum/FCQ1/FQM1 native parser diagnostic [";
	ferrum_chem_owned_buffer response{nullptr, 0};
	const std::vector<uint8_t> request = fcq1(hostile);
	assert(ferrum_chem_smarts_match_v1(request.data(), request.size(), &response) == FERRUM_CHEM_CALL_OK);
	assert(u32_at(response, 8) == FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY);
	const char path_token[] = "/private/ferrum";
	const char diagnostic_token[] = "native parser diagnostic";
	assert(std::search(response.data, response.data + response.len, path_token, path_token + sizeof(path_token) - 1) ==
		response.data + response.len);
	assert(std::search(response.data, response.data + response.len, diagnostic_token,
		diagnostic_token + sizeof(diagnostic_token) - 1) == response.data + response.len);
	ferrum_chem_owned_buffer_free_v1(&response);
}

void expect_truncated_deterministic() {
	const std::vector<uint8_t> request = fcq1("C", 1, 2);
	ferrum_chem_owned_buffer first{nullptr, 0};
	ferrum_chem_owned_buffer second{nullptr, 0};
	assert(ferrum_chem_smarts_match_v1(request.data(), request.size(), &first) == FERRUM_CHEM_CALL_OK);
	assert(ferrum_chem_smarts_match_v1(request.data(), request.size(), &second) == FERRUM_CHEM_CALL_OK);
	assert(first.len == second.len && std::memcmp(first.data, second.data, first.len) == 0);
	assert(first.len == FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES + sizeof(uint32_t));
	assert(std::memcmp(first.data, "FQM1", 4) == 0);
	assert(u32_at(first, 4) == FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION);
	assert(u32_at(first, 8) == FERRUM_CHEM_SMARTS_MATCH_STATUS_OK);
	assert(u32_at(first, 12) == 0 && u32_at(first, 16) == 1 && u32_at(first, 20) == 1);
	assert(u32_at(first, 24) == FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED);
	assert(u32_at(first, FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES) == 0);
	ferrum_chem_owned_buffer_free_v1(&first);
	ferrum_chem_owned_buffer_free_v1(&second);
}

}  // namespace

int main() {
	const std::vector<uint8_t> match = fcq1("C");
	expect_match(match, 1, 1, {0});
	expect_match(fcq1("O"), 1, 0);

	// Profile-defining valid SMARTS forms are accepted and reach matching.
	expect_match(fcq1_bytes("c", fcg1({Atom{6, true}})), 1, 1);
	expect_match(fcq1_bytes("[13C+]", fcg1({Atom{6, false, FERRUM_CHEM_CHIRAL_UNSPECIFIED,
		FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE | FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE,
		1, 13}})), 1, 1);
	const std::vector<uint8_t> chiral_target = fcg1(
		{{6, false, FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW,
			FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS, 0, 0, 1}, {9}, {17}, {35}},
		{{0, 1}, {0, 2}, {0, 3}});
	const std::vector<uint8_t> clockwise = fcq1_bytes("[C@H](F)(Cl)Br", chiral_target);
	const std::vector<uint8_t> counterclockwise = fcq1_bytes("[C@@H](F)(Cl)Br", chiral_target);
	expect_match(clockwise, 4, 1);
	expect_match(counterclockwise, 4, 0);
	expect_match(fcq1("[$(C)]"), 1, 1);
	expect_match(fcq1_bytes("C.C", carbon_fcg1(2)), 2, 1);

	expect_status(fcq1("["), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY, "invalid_query");
	// RDKit accepts whitespace as an atomless SMARTS; FCQ1 rejects its zero-atom query exactly.
	expect_status(fcq1(" "), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY, "invalid_query");
	std::string oversized_query;
	for (uint32_t index = 0; index < 65; ++index) oversized_query += "C";
	std::vector<uint8_t> malformed_target = carbon_fcg1(1);
	malformed_target[0] = 'X';
	expect_status(fcq1_bytes(oversized_query, malformed_target), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
		"query_atom_limit_exceeded");
	std::string nul_smarts{"C\0C", 3};
	expect_status(fcq1_bytes(nul_smarts, carbon_fcg1(1)), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
		"invalid_request");
	const std::string invalid_utf8(1, static_cast<char>(0xff));
	expect_status(fcq1_bytes(invalid_utf8, carbon_fcg1(1)), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
		"invalid_request");
	expect_status(fcq1("C", 0), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, "invalid_request");
	expect_status(fcq1("C", FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS + 1),
		FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, "invalid_request");
	expect_status(fcq1_bytes("C", carbon_fcg1(1), 1, 1),
		FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, "invalid_request");
	expect_status(fcq1_bytes("C", malformed_target), FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
		"invalid_request");
	expect_status(fcq1_bytes("C", fcg1({})), FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET,
		"unsupported_target");
	expect_truncated_deterministic();
	expect_hostile_detail_is_redacted();
	std::vector<uint8_t> malformed = match; malformed.pop_back();
	expect_status(malformed, FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, "invalid_request");
	std::vector<uint8_t> trailing = match; trailing.push_back(0);
	expect_status(trailing, FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST, "invalid_request");
	return 0;
}
