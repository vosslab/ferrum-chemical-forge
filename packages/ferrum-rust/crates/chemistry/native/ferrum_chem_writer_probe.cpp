#include "ferrum_chem_writer_probe.h"

#ifdef FERRUM_CHEM_NATIVE_TESTING

#include <array>
#include <atomic>
#include <cstddef>

namespace ferrum_chem {
namespace {

constexpr size_t kWriterCount = 5;
std::array<std::atomic<uint64_t>, kWriterCount> writer_invocations{};

size_t writer_index(NativeTextWriter writer) {
	return static_cast<size_t>(writer);
}

}  // namespace

void record_native_text_writer_invocation(NativeTextWriter writer) {
	writer_invocations[writer_index(writer)].fetch_add(1, std::memory_order_relaxed);
}

void reset_native_text_writer_invocations() {
	for (auto &count : writer_invocations) count.store(0, std::memory_order_relaxed);
}

uint64_t native_text_writer_invocations(NativeTextWriter writer) {
	return writer_invocations[writer_index(writer)].load(std::memory_order_relaxed);
}

}  // namespace ferrum_chem

#endif
