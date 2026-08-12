cmake_minimum_required(VERSION 3.24)
project(ferrum_chem_adapter LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 20)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

set(FERRUM_CHEM_RDKIT_INCLUDE_DIR "" CACHE PATH
    "Explicit RDKit include/rdkit root containing GraphMol/ and RDGeneral/")
set(FERRUM_CHEM_BOOST_INCLUDE_DIR "" CACHE PATH
    "Explicit pinned Boost header root containing boost/config.hpp")
set(FERRUM_CHEM_RDKIT_GRAPHMOL "" CACHE FILEPATH
    "Explicit libRDKitGraphMol dylib for Ferrum's private adapter")
set(FERRUM_CHEM_RDKIT_RDGENERAL "" CACHE FILEPATH
    "Explicit libRDKitRDGeneral dylib for Ferrum's private adapter")
set(FERRUM_CHEM_RDKIT_DEPICTOR "" CACHE FILEPATH
    "Optional libRDKitDepictor dylib enabling M4c coordinate generation")

foreach(required_input IN ITEMS FERRUM_CHEM_RDKIT_INCLUDE_DIR FERRUM_CHEM_BOOST_INCLUDE_DIR
		FERRUM_CHEM_RDKIT_GRAPHMOL FERRUM_CHEM_RDKIT_RDGENERAL)
	if(NOT ${required_input})
		message(FATAL_ERROR "${required_input} is required; Ferrum never discovers host RDKit")
	endif()
	if(NOT EXISTS "${${required_input}}")
		message(FATAL_ERROR "${required_input} does not exist: ${${required_input}}")
	endif()
endforeach()
if(FERRUM_CHEM_RDKIT_DEPICTOR AND NOT EXISTS "${FERRUM_CHEM_RDKIT_DEPICTOR}")
	message(FATAL_ERROR "FERRUM_CHEM_RDKIT_DEPICTOR does not exist: ${FERRUM_CHEM_RDKIT_DEPICTOR}")
endif()
if(NOT EXISTS "${FERRUM_CHEM_RDKIT_INCLUDE_DIR}/GraphMol/MolOps.h")
	message(FATAL_ERROR "FERRUM_CHEM_RDKIT_INCLUDE_DIR lacks GraphMol/MolOps.h")
endif()
if(NOT EXISTS "${FERRUM_CHEM_RDKIT_INCLUDE_DIR}/RDGeneral/types.h")
	message(FATAL_ERROR "FERRUM_CHEM_RDKIT_INCLUDE_DIR lacks RDGeneral/types.h")
endif()
if(NOT EXISTS "${FERRUM_CHEM_BOOST_INCLUDE_DIR}/boost/config.hpp")
	message(FATAL_ERROR "FERRUM_CHEM_BOOST_INCLUDE_DIR lacks boost/config.hpp")
endif()

add_library(ferrum_chem SHARED ferrum_chem_adapter.cpp)
target_include_directories(ferrum_chem
	PUBLIC "$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>"
	PRIVATE "${FERRUM_CHEM_RDKIT_INCLUDE_DIR}" "${FERRUM_CHEM_BOOST_INCLUDE_DIR}")
target_link_libraries(ferrum_chem PRIVATE
	"${FERRUM_CHEM_RDKIT_GRAPHMOL}"
	"${FERRUM_CHEM_RDKIT_RDGENERAL}")
if(FERRUM_CHEM_RDKIT_DEPICTOR)
	target_compile_definitions(ferrum_chem PRIVATE FERRUM_CHEM_ENABLE_DEPICTOR=1)
	target_link_libraries(ferrum_chem PRIVATE "${FERRUM_CHEM_RDKIT_DEPICTOR}")
endif()
target_link_options(ferrum_chem PRIVATE "-Wl,-rpath,@loader_path")
set_target_properties(ferrum_chem PROPERTIES
	OUTPUT_NAME ferrum_chem
	MACOSX_RPATH ON
	INSTALL_NAME_DIR "@rpath")

install(TARGETS ferrum_chem LIBRARY DESTINATION lib)
install(FILES include/ferrum_chem_adapter.h DESTINATION include)
