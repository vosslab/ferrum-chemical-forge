#include "ferrum_chem_adapter.h"

#ifndef FERRUM_CHEM_BUILD_MARKER
#define FERRUM_CHEM_BUILD_MARKER "unmarked"
#endif

extern "C" uint32_t ferrum_chem_abi_version(void) {
	return 1;
}

extern "C" const char *ferrum_chem_build_marker(void) {
	return FERRUM_CHEM_BUILD_MARKER;
}
