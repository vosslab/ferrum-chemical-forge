#ifndef FERRUM_CHEM_MOLBLOCK_REQUEST_H
#define FERRUM_CHEM_MOLBLOCK_REQUEST_H

#include <cstdint>
#include <string>

namespace RDKit {
class RWMol;
}

namespace ferrum_chem {

bool parse_molblock_request(const uint8_t *request, uint64_t request_len,
	RDKit::RWMol *molecule, uint32_t *format, std::string *error);

}

#endif
