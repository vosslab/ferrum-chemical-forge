#include "ferrum_chem_adapter.h"
#include "ferrum_chem_complete_graph.h"
#include "ferrum_chem_text_response.h"
#include "ferrum_chem_text_output_limit.h"
#include "ferrum_chem_writer_probe.h"

#include <GraphMol/MolOps.h>
#include <GraphMol/RWMol.h>
#include <GraphMol/SanitException.h>
#include <GraphMol/SmilesParse/SmilesWrite.h>

#include <algorithm>
#include <cstddef>
#include <cstdint>
#include <exception>
#include <new>
#include <string>
#include <string_view>

namespace {

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

bool valid_smiles_line(const std::string &smiles) {
	return !smiles.empty() && std::all_of(smiles.begin(), smiles.end(), [](unsigned char byte) {
		return byte >= 0x21U && byte <= 0x7eU;
	});
}

uint32_t emit_error(uint32_t status, std::string_view detail,
		ferrum_chem_owned_buffer *response) {
	return ferrum_chem::emit_text_response(
		status, bounded_detail(detail, "native SMILES export failed"), "", response);
}

}  // namespace

extern "C" uint32_t ferrum_chem_molecule_to_smiles_v1(
		const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
		ferrum_chem_owned_buffer *response) noexcept {
	if (response == nullptr) return FERRUM_CHEM_CALL_INVALID_ARGUMENT;
	response->data = nullptr;
	response->len = 0;
	try {
		RDKit::RWMol molecule;
		std::string error;
		if (!ferrum_chem::parse_complete_graph(request, request_len, &molecule, &error)) {
			return emit_error(FERRUM_CHEM_RESULT_MALFORMED_REQUEST, error, response);
		}
		if (molecule.getNumAtoms() == 0) {
			return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
				"SMILES export requires at least one atom", response);
		}
		if (!ferrum_chem::text_output_is_admitted(
				ferrum_chem::smiles_text_upper_bound(
					molecule.getNumAtoms(), molecule.getNumBonds()), maximum_text_bytes)) {
			return emit_error(FERRUM_CHEM_RESULT_RESOURCE_LIMIT,
				"canonical SMILES upper bound exceeds the requested text limit", response);
		}
		RDKit::MolOps::sanitizeMol(molecule);
		RDKit::SmilesWriteParams parameters;
		parameters.doIsomericSmiles = true;
		parameters.doKekule = false;
		parameters.canonical = true;
		parameters.cleanStereo = true;
		parameters.allBondsExplicit = false;
		parameters.allHsExplicit = false;
		parameters.doRandom = false;
		parameters.rootedAtAtom = -1;
		parameters.includeDativeBonds = true;
		parameters.ignoreAtomMapNumbers = false;
		ferrum_chem::record_native_text_writer_invocation(
			ferrum_chem::NativeTextWriter::Smiles);
		const std::string smiles = RDKit::MolToSmiles(molecule, parameters);
		if (smiles.size() > FERRUM_CHEM_SMILES_WRITE_MAX_BYTES) {
			return emit_error(FERRUM_CHEM_RESULT_RESOURCE_LIMIT,
				"canonical SMILES exceeds the ABI output limit", response);
		}
		if (!valid_smiles_line(smiles)) {
			return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE,
				"RDKit returned an invalid canonical SMILES line", response);
		}
		return ferrum_chem::emit_text_response(
			FERRUM_CHEM_RESULT_OK, "", smiles, response);
	} catch (const std::bad_alloc &) {
		return FERRUM_CHEM_CALL_ALLOCATION_FAILURE;
	} catch (const RDKit::MolSanitizeException &error) {
		return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), response);
	} catch (const std::exception &error) {
		return emit_error(FERRUM_CHEM_RESULT_INVALID_MOLECULE, error.what(), response);
	} catch (...) {
		return emit_error(
			FERRUM_CHEM_RESULT_INTERNAL_FAILURE, "unknown native failure", response);
	}
}
