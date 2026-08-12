cmake_minimum_required(VERSION 3.24)
project(ferrum_chem_adapter LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(FERRUM_CHEM_BUILD_MARKER "wheel" CACHE STRING
    "Static build marker exposed by the M4a probe")
set(FERRUM_CHEM_RDKIT_RDGENERAL "" CACHE FILEPATH
    "Optional libRDKitRDGeneral dylib; only the packaging proof supplies it")

if(NOT FERRUM_CHEM_BUILD_MARKER MATCHES "^[A-Za-z0-9._-]+$")
	message(FATAL_ERROR "FERRUM_CHEM_BUILD_MARKER must contain only letters, digits, ., _, or -")
endif()

add_library(ferrum_chem SHARED ferrum_chem_adapter.cpp)
target_include_directories(ferrum_chem
	PUBLIC "$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>")
target_compile_definitions(ferrum_chem
	PRIVATE FERRUM_CHEM_BUILD_MARKER="${FERRUM_CHEM_BUILD_MARKER}")
set_target_properties(ferrum_chem PROPERTIES
	OUTPUT_NAME ferrum_chem
	MACOSX_RPATH ON
	INSTALL_NAME_DIR "@rpath")

# M4a deliberately does not call RDKit.  Linking this one declared library is
# sufficient to prove the package closure while keeping chemistry semantics in M4b.
if(FERRUM_CHEM_RDKIT_RDGENERAL)
	if(NOT EXISTS "${FERRUM_CHEM_RDKIT_RDGENERAL}")
		message(FATAL_ERROR "FERRUM_CHEM_RDKIT_RDGENERAL does not exist: ${FERRUM_CHEM_RDKIT_RDGENERAL}")
	endif()
	target_link_libraries(ferrum_chem PRIVATE "${FERRUM_CHEM_RDKIT_RDGENERAL}")
	target_link_options(ferrum_chem PRIVATE "-Wl,-rpath,@loader_path")
endif()

install(TARGETS ferrum_chem LIBRARY DESTINATION lib)
install(FILES include/ferrum_chem_adapter.h DESTINATION include)
