#ifndef FERRUM_CHEM_COMPLETE_GRAPH_H
#define FERRUM_CHEM_COMPLETE_GRAPH_H

#include <cstdint>
#include <string>

namespace RDKit {
class RWMol;
}

namespace ferrum_chem {

bool parse_complete_graph(
	const uint8_t *request,
	uint64_t request_len,
	RDKit::RWMol *molecule,
	std::string *error);

}  // namespace ferrum_chem

#endif
