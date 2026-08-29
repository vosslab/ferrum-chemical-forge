#include "ferrum_chem_adapter.h"
#include "ferrum_chem_text_output_limit.h"
#include "ferrum_chem_writer_probe.h"

#include <cassert>
#include <cstdint>
#include <cstring>
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

uint32_t u32_at(const ferrum_chem_owned_buffer &buffer, size_t offset) {
	assert(offset + sizeof(uint32_t) <= buffer.len);
	return static_cast<uint32_t>(buffer.data[offset]) |
		(static_cast<uint32_t>(buffer.data[offset + 1]) << 8U) |
		(static_cast<uint32_t>(buffer.data[offset + 2]) << 16U) |
		(static_cast<uint32_t>(buffer.data[offset + 3]) << 24U);
}

void expect_text_status(uint32_t call_status, ferrum_chem_owned_buffer *response,
		uint32_t result_status) {
	assert(call_status == FERRUM_CHEM_CALL_OK);
	assert(response->data != nullptr && response->len >= FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES);
	assert(std::memcmp(response->data, "FCT1", 4) == 0);
	assert(u32_at(*response, 8) == result_status);
	ferrum_chem_owned_buffer_free_v1(response);
}

std::vector<uint8_t> carbon_graph() {
	std::vector<uint8_t> graph{'F', 'C', 'G', '1'};
	append_u32(&graph, FERRUM_CHEM_GRAPH_WIRE_VERSION);
	append_u32(&graph, 1);
	append_u32(&graph, 0);
	append_u32(&graph, FERRUM_CHEM_GRAPH_FLAGS_NONE);
	graph.push_back(6);  // carbon
	graph.push_back(0);  // non-aromatic
	graph.push_back(FERRUM_CHEM_CHIRAL_UNSPECIFIED);
	graph.push_back(0);
	append_u32(&graph, 0);  // no optional atom facts
	append_u32(&graph, 0);  // neutral charge
	append_u16(&graph, 0);
	append_u16(&graph, 0);
	graph.push_back(0);
	graph.push_back(0);  // implicit hydrogens allowed
	append_u16(&graph, 0);
	append_u32(&graph, 0);
	return graph;
}

std::vector<uint8_t> carbon_molblock(uint32_t format) {
	std::vector<uint8_t> request{'F', 'C', 'B', '1'};
	append_u32(&request, FERRUM_CHEM_MOLBLOCK_WIRE_VERSION);
	append_u32(&request, format);
	append_u32(&request, 1);
	append_u32(&request, 0);
	append_u32(&request, FERRUM_CHEM_MOLBLOCK_FLAGS_NONE);
	const std::vector<uint8_t> graph = carbon_graph();
	request.insert(request.end(), graph.begin() + FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES,
		graph.end());
	for (uint32_t index = 0; index < FERRUM_CHEM_COORDINATE_BYTES; ++index) {
		request.push_back(0);
	}
	return request;
}

std::vector<uint8_t> titled_molblock(uint32_t format, std::string_view title) {
	const std::vector<uint8_t> molblock = carbon_molblock(format);
	std::vector<uint8_t> request{'F', 'B', 'T', '1'};
	append_u32(&request, FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION);
	append_u32(&request, static_cast<uint32_t>(molblock.size()));
	append_u32(&request, static_cast<uint32_t>(title.size()));
	request.insert(request.end(), molblock.begin(), molblock.end());
	request.insert(request.end(), title.begin(), title.end());
	return request;
}

std::vector<uint8_t> inchi_request(uint32_t mode) {
	const std::vector<uint8_t> graph = carbon_graph();
	std::vector<uint8_t> request{'F', 'C', 'I', '1'};
	append_u32(&request, FERRUM_CHEM_INCHI_WIRE_VERSION);
	append_u32(&request, mode);
	append_u32(&request, static_cast<uint32_t>(graph.size()));
	append_u32(&request, FERRUM_CHEM_INCHI_FLAGS_NONE);
	request.insert(request.end(), graph.begin(), graph.end());
	return request;
}

void append_sdf_record(std::vector<uint8_t> *request, uint32_t format,
		std::string_view title, std::string_view property_name, std::string_view property_value) {
	const std::vector<uint8_t> molecule = carbon_molblock(format);
	append_u32(request, static_cast<uint32_t>(molecule.size()));
	append_u32(request, static_cast<uint32_t>(title.size()));
	append_u32(request, 1);
	append_u32(request, FERRUM_CHEM_SDF_FLAGS_NONE);
	request->insert(request->end(), molecule.begin(), molecule.end());
	request->insert(request->end(), title.begin(), title.end());
	append_u32(request, static_cast<uint32_t>(property_name.size()));
	append_u32(request, static_cast<uint32_t>(property_value.size()));
	request->insert(request->end(), property_name.begin(), property_name.end());
	request->insert(request->end(), property_value.begin(), property_value.end());
}

std::vector<uint8_t> multi_record_sdf() {
	std::vector<uint8_t> request{'F', 'S', 'D', '1'};
	append_u32(&request, FERRUM_CHEM_SDF_WIRE_VERSION);
	append_u32(&request, 2);
	append_u32(&request, FERRUM_CHEM_SDF_FLAGS_NONE);
	append_sdf_record(&request, FERRUM_CHEM_MOLBLOCK_FORMAT_V2000, "first", "kind", "alkane");
	append_sdf_record(&request, FERRUM_CHEM_MOLBLOCK_FORMAT_V3000, "second", "kind", "alkane");
	return request;
}

uint64_t maximum_text_output_bytes() {
	return static_cast<uint64_t>(FERRUM_CHEM_MAX_RESPONSE_BYTES) -
		FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES;
}

template <typename Call>
void expect_preallocation_refusal_then_admission(ferrum_chem::NativeTextWriter writer,
		uint64_t upper_bound, uint64_t expected_admitted_calls, Call call) {
	assert(upper_bound > 0);
	assert(upper_bound <= maximum_text_output_bytes());
	assert(FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES + upper_bound <=
		FERRUM_CHEM_MAX_RESPONSE_BYTES);
	ferrum_chem::reset_native_text_writer_invocations();
	ferrum_chem_owned_buffer refused{nullptr, 0};
	expect_text_status(call(upper_bound - 1, &refused), &refused,
		FERRUM_CHEM_RESULT_RESOURCE_LIMIT);
	assert(ferrum_chem::native_text_writer_invocations(writer) == 0);

	ferrum_chem_owned_buffer admitted{nullptr, 0};
	expect_text_status(call(upper_bound, &admitted), &admitted, FERRUM_CHEM_RESULT_OK);
	assert(ferrum_chem::native_text_writer_invocations(writer) == expected_admitted_calls);
}

void check_molblock_rows() {
	for (const uint32_t format : {FERRUM_CHEM_MOLBLOCK_FORMAT_V2000,
			FERRUM_CHEM_MOLBLOCK_FORMAT_V3000}) {
		const std::vector<uint8_t> request = carbon_molblock(format);
		const uint64_t upper_bound = ferrum_chem::molblock_text_upper_bound(format, 1, 0, 0);
		expect_preallocation_refusal_then_admission(ferrum_chem::NativeTextWriter::Molblock,
			upper_bound, 1, [&request](uint64_t limit, ferrum_chem_owned_buffer *response) {
				return ferrum_chem_molecule_to_molblock_v1(
					request.data(), request.size(), limit, response);
			});
	}
}

void check_titled_molblock_rows() {
	constexpr std::string_view kTitle = "carbon";
	for (const uint32_t format : {FERRUM_CHEM_MOLBLOCK_FORMAT_V2000,
			FERRUM_CHEM_MOLBLOCK_FORMAT_V3000}) {
		const std::vector<uint8_t> request = titled_molblock(format, kTitle);
		const uint64_t upper_bound = ferrum_chem::molblock_text_upper_bound(
			format, 1, 0, kTitle.size());
		expect_preallocation_refusal_then_admission(
			ferrum_chem::NativeTextWriter::TitledMolblock, upper_bound, 1,
			[&request](uint64_t limit, ferrum_chem_owned_buffer *response) {
				return ferrum_chem_molecule_to_molblock_with_title_v1(
					request.data(), request.size(), limit, response);
			});
	}
}

void check_smiles_row() {
	const std::vector<uint8_t> request = carbon_graph();
	expect_preallocation_refusal_then_admission(ferrum_chem::NativeTextWriter::Smiles,
		ferrum_chem::smiles_text_upper_bound(1, 0), 1,
		[&request](uint64_t limit, ferrum_chem_owned_buffer *response) {
			return ferrum_chem_molecule_to_smiles_v1(request.data(), request.size(), limit, response);
		});
}

void check_inchi_rows() {
	for (const uint32_t mode : {FERRUM_CHEM_INCHI_MODE_STANDARD,
			FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN}) {
		const std::vector<uint8_t> request = inchi_request(mode);
		expect_preallocation_refusal_then_admission(ferrum_chem::NativeTextWriter::Inchi,
			ferrum_chem::inchi_text_upper_bound(1, 0), 1,
			[&request](uint64_t limit, ferrum_chem_owned_buffer *response) {
				return ferrum_chem_molecule_to_inchi_v1(request.data(), request.size(), limit, response);
			});
	}
}

void check_sdf_multi_record_row() {
	const std::vector<uint8_t> request = multi_record_sdf();
	const uint64_t upper_bound = ferrum_chem::saturating_add(
		ferrum_chem::sdf_record_text_upper_bound(
			FERRUM_CHEM_MOLBLOCK_FORMAT_V2000, 1, 0, 5, 4, 6, 1),
		ferrum_chem::sdf_record_text_upper_bound(
			FERRUM_CHEM_MOLBLOCK_FORMAT_V3000, 1, 0, 6, 4, 6, 1));
	expect_preallocation_refusal_then_admission(ferrum_chem::NativeTextWriter::Sdf,
		upper_bound, 2, [&request](uint64_t limit, ferrum_chem_owned_buffer *response) {
			return ferrum_chem_records_to_sdf_v1(request.data(), request.size(), limit, response);
		});
}

void check_sdf_property_limit_refuses_before_parse_allocation() {
	std::vector<uint8_t> request = multi_record_sdf();
	// First record's property count lives after the FSD1 header and its four record-header words.
	request[FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES + 8] =
		static_cast<uint8_t>(FERRUM_CHEM_SDF_MAX_PROPERTIES + 1U);
	request[FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES + 9] =
		static_cast<uint8_t>((FERRUM_CHEM_SDF_MAX_PROPERTIES + 1U) >> 8U);
	request[FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES + 10] =
		static_cast<uint8_t>((FERRUM_CHEM_SDF_MAX_PROPERTIES + 1U) >> 16U);
	request[FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES + 11] =
		static_cast<uint8_t>((FERRUM_CHEM_SDF_MAX_PROPERTIES + 1U) >> 24U);
	ferrum_chem::reset_native_text_writer_invocations();
	ferrum_chem_owned_buffer response{nullptr, 0};
	expect_text_status(ferrum_chem_records_to_sdf_v1(
		request.data(), request.size(), maximum_text_output_bytes(), &response), &response,
		FERRUM_CHEM_RESULT_RESOURCE_LIMIT);
	assert(ferrum_chem::native_text_writer_invocations(
		ferrum_chem::NativeTextWriter::Sdf) == 0);
}

void check_oversized_caller_budget_refuses_before_writer_invocation() {
	const std::vector<uint8_t> request = carbon_graph();
	const uint64_t maximum_text_bytes = maximum_text_output_bytes();
	assert(FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES + maximum_text_bytes ==
		FERRUM_CHEM_MAX_RESPONSE_BYTES);
	for (const uint64_t rejected_limit : {uint64_t{0},
			static_cast<uint64_t>(FERRUM_CHEM_MAX_RESPONSE_BYTES), maximum_text_bytes + 1U}) {
		ferrum_chem::reset_native_text_writer_invocations();
		ferrum_chem_owned_buffer response{nullptr, 0};
		expect_text_status(ferrum_chem_molecule_to_smiles_v1(
			request.data(), request.size(), rejected_limit, &response),
			&response, FERRUM_CHEM_RESULT_RESOURCE_LIMIT);
		assert(ferrum_chem::native_text_writer_invocations(
			ferrum_chem::NativeTextWriter::Smiles) == 0);
	}

	assert(!ferrum_chem::text_output_is_admitted(maximum_text_bytes + 1U,
		maximum_text_bytes));
	ferrum_chem::reset_native_text_writer_invocations();
	ferrum_chem_owned_buffer response{nullptr, 0};
	expect_text_status(ferrum_chem_molecule_to_smiles_v1(
		request.data(), request.size(), maximum_text_bytes, &response),
		&response, FERRUM_CHEM_RESULT_OK);
	assert(ferrum_chem::native_text_writer_invocations(
		ferrum_chem::NativeTextWriter::Smiles) == 1);
}

}  // namespace

int main() {
	check_molblock_rows();
	check_titled_molblock_rows();
	check_smiles_row();
	check_inchi_rows();
	check_sdf_multi_record_row();
	check_sdf_property_limit_refuses_before_parse_allocation();
	check_oversized_caller_budget_refuses_before_writer_invocation();
}
