/*
 * Minimal Ferrum-Chem dynamic-adapter ABI used by the M4a wheel proof.
 *
 * Ownership: both functions return scalar values or pointers to static storage.
 * Callers must never free the returned marker.  A future fallible ABI must return
 * an explicit status and document which side releases any allocated result.
 */
#ifndef FERRUM_CHEM_ADAPTER_H
#define FERRUM_CHEM_ADAPTER_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* This ABI is compatible only when this value remains 1. */
uint32_t ferrum_chem_abi_version(void);

/* A non-null, NUL-terminated marker in static storage. */
const char *ferrum_chem_build_marker(void);

#ifdef __cplusplus
}
#endif

#endif
