#ifndef FERRUM_CHEM_WRITER_PROBE_H
#define FERRUM_CHEM_WRITER_PROBE_H

#include <cstdint>

namespace ferrum_chem {

enum class NativeTextWriter : uint8_t {
	Molblock,
	TitledMolblock,
	Smiles,
	Sdf,
	Inchi,
};

#ifdef FERRUM_CHEM_NATIVE_TESTING
void record_native_text_writer_invocation(NativeTextWriter writer);
void reset_native_text_writer_invocations();
uint64_t native_text_writer_invocations(NativeTextWriter writer);
#else
inline void record_native_text_writer_invocation(NativeTextWriter) {}
#endif

}  // namespace ferrum_chem

#endif
