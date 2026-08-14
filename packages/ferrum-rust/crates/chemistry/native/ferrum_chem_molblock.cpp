#include "ferrum_chem_adapter.h"
#include "ferrum_chem_complete_graph.h"
#include "ferrum_chem_molblock_request.h"
#include "ferrum_chem_text_response.h"
#include "ferrum_chem_utf8.h"

#include <Geometry/point.h>
#include <GraphMol/Conformer.h>
#include <GraphMol/FileParsers/FileWriters.h>
#include <GraphMol/RWMol.h>

#include <bit>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <iterator>
#include <limits>
#include <memory>
#include <new>
#include <string>
#include <vector>

namespace {

constexpr uint8_t kMolblockMagic[] = {'F', 'C', 'B', '1'};
constexpr uint8_t kTitledMolblockMagic[] = {'F', 'B', 'T', '1'};
constexpr uint8_t kGraphMagic[] = {'F', 'C', 'G', '1'};

uint32_t read_u32(const uint8_t *bytes) {
	return static_cast<uint32_t>(bytes[0]) |
		(static_cast<uint32_t>(bytes[1]) << 8U) |
		(static_cast<uint32_t>(bytes[2]) << 16U) |
		(static_cast<uint32_t>(bytes[3]) << 24U);
}

uint64_t read_u64(const uint8_t *bytes) {
	return static_cast<uint64_t>(read_u32(bytes)) |
		(static_cast<uint64_t>(read_u32(bytes + 4)) << 32U);
}

void append_u32(std::vector<uint8_t> &bytes, uint32_t value) {
	bytes.push_back(static_cast<uint8_t>(value));
	bytes.push_back(static_cast<uint8_t>(value >> 8U));
	bytes.push_back(static_cast<uint8_t>(value >> 16U));
	bytes.push_back(static_cast<uint8_t>(value >> 24U));
}

bool checked_length(uint64_t atom_count, uint64_t bond_count, uint64_t *records,
		uint64_t *coordinates) {
	if (atom_count > FERRUM_CHEM_KEKULIZE_MAX_ATOMS ||
		bond_count > FERRUM_CHEM_KEKULIZE_MAX_BONDS) {
		return false;
	}
	*records = atom_count * FERRUM_CHEM_GRAPH_ATOM_BYTES +
		bond_count * FERRUM_CHEM_GRAPH_BOND_BYTES;
	*coordinates = atom_count * FERRUM_CHEM_COORDINATE_BYTES;
	return *records <= std::numeric_limits<uint64_t>::max() - *coordinates -
		FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES;
}

bool valid_title(const std::string &title) {
	return title.find('\0') == std::string::npos &&
		title.find('\r') == std::string::npos && title.find('\n') == std::string::npos &&
		ferrum_chem::is_valid_utf8(title);
}

uint32_t write_molblock(RDKit::RWMol &molecule, uint32_t format,
		ferrum_chem_owned_buffer *response) {
	RDKit::MolWriterParams parameters;
	parameters.includeStereo = true;
	parameters.kekulize = true;
	const std::string output = format == FERRUM_CHEM_MOLBLOCK_FORMAT_V2000 ?
		RDKit::MolToV2KMolBlock(molecule, parameters) :
		RDKit::MolToV3KMolBlock(molecule, parameters);
	if (output.empty()) {
		return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
			"RDKit could not generate the requested molblock", "", response);
	}
	return ferrum_chem::emit_text_response(FERRUM_CHEM_RESULT_OK, "", output, response);
}

bool parse_titled_molblock_request(const uint8_t *request, uint64_t request_len,
		RDKit::RWMol *molecule, uint32_t *format, std::string *error) {
	if (request == nullptr ||
		request_len < FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES) {
		*error = "titled molblock request is missing or truncated";
		return false;
	}
	if (std::memcmp(request, kTitledMolblockMagic, sizeof(kTitledMolblockMagic)) != 0 ||
		read_u32(request + 4) != FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION) {
		*error = "titled molblock request has invalid magic or version";
		return false;
	}
	const uint32_t molecule_length = read_u32(request + 8);
	const uint32_t title_length = read_u32(request + 12);
	const uint64_t payload_length = static_cast<uint64_t>(molecule_length) + title_length;
	if (request_len != FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES + payload_length) {
		*error = "titled molblock request is truncated or has trailing bytes";
		return false;
	}
	const uint8_t *molecule_bytes =
		request + FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES;
	if (!ferrum_chem::parse_molblock_request(
			molecule_bytes, molecule_length, molecule, format, error)) {
		return false;
	}
	const uint8_t *title_bytes = molecule_bytes + molecule_length;
	const std::string title(reinterpret_cast<const char *>(title_bytes), title_length);
	if (!valid_title(title)) {
		*error = "molblock title is invalid UTF-8 or contains NUL or a line break";
		return false;
	}
	molecule->setProp(RDKit::common_properties::_Name, title);
	return true;
}

}  // namespace

bool ferrum_chem::parse_molblock_request(const uint8_t *request, uint64_t request_len,
		RDKit::RWMol *molecule, uint32_t *format, std::string *error) {
	if (request == nullptr || request_len < FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES) {
		*error = "molblock request is missing or truncated";
		return false;
	}
	if (std::memcmp(request, kMolblockMagic, sizeof(kMolblockMagic)) != 0 ||
		read_u32(request + 4) != FERRUM_CHEM_MOLBLOCK_WIRE_VERSION) {
		*error = "molblock request has invalid magic or version";
		return false;
	}
	*format = read_u32(request + 8);
	const uint32_t atom_count = read_u32(request + 12);
	const uint32_t bond_count = read_u32(request + 16);
	if ((*format != FERRUM_CHEM_MOLBLOCK_FORMAT_V2000 &&
			*format != FERRUM_CHEM_MOLBLOCK_FORMAT_V3000) ||
		read_u32(request + 20) != FERRUM_CHEM_MOLBLOCK_FLAGS_NONE) {
		*error = "molblock request has an unsupported format or reserved flags";
		return false;
	}
	uint64_t record_bytes = 0;
	uint64_t coordinate_bytes = 0;
	if (!checked_length(atom_count, bond_count, &record_bytes, &coordinate_bytes) ||
		request_len != FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES + record_bytes +
			coordinate_bytes) {
		*error = "molblock request records are oversized, truncated, or trailing";
		return false;
	}

	std::vector<uint8_t> graph;
	graph.reserve(FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES + record_bytes);
	graph.insert(graph.end(), std::begin(kGraphMagic), std::end(kGraphMagic));
	append_u32(graph, FERRUM_CHEM_GRAPH_WIRE_VERSION);
	append_u32(graph, atom_count);
	append_u32(graph, bond_count);
	append_u32(graph, FERRUM_CHEM_GRAPH_FLAGS_NONE);
	const uint8_t *records = request + FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES;
	graph.insert(graph.end(), records, records + record_bytes);
	if (!parse_complete_graph(graph.data(), graph.size(), molecule, error)) {
		return false;
	}

	auto conformer = std::make_unique<RDKit::Conformer>(atom_count);
	conformer->set3D(false);
	const uint8_t *coordinate = records + record_bytes;
	for (uint32_t index = 0; index < atom_count;
			++index, coordinate += FERRUM_CHEM_COORDINATE_BYTES) {
		const double x = std::bit_cast<double>(read_u64(coordinate));
		const double y = std::bit_cast<double>(read_u64(coordinate + 8));
		if (!std::isfinite(x) || !std::isfinite(y)) {
			*error = "molblock request contains a non-finite coordinate";
			return false;
		}
		conformer->setAtomPos(index, RDGeom::Point3D(x, y, 0.0));
	}
	molecule->addConformer(conformer.release(), true);
	return true;
}

extern "C" uint32_t ferrum_chem_molecule_to_molblock_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) {
		return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	}
	response->data = nullptr;
	response->len = 0;
	try {
		RDKit::RWMol molecule;
		uint32_t format = 0;
		std::string error;
		if (!ferrum_chem::parse_molblock_request(
				request, request_len, &molecule, &format, &error)) {
			return ferrum_chem::emit_text_response(
				FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, "", response);
		}
		return write_molblock(molecule, format, response);
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

extern "C" uint32_t ferrum_chem_molecule_to_molblock_with_title_v1(
		const uint8_t *request, uint64_t request_len,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) {
		return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	}
	response->data = nullptr;
	response->len = 0;
	try {
		RDKit::RWMol molecule;
		uint32_t format = 0;
		std::string error;
		if (!parse_titled_molblock_request(
				request, request_len, &molecule, &format, &error)) {
			return ferrum_chem::emit_text_response(
				FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, "", response);
		}
		return write_molblock(molecule, format, response);
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
