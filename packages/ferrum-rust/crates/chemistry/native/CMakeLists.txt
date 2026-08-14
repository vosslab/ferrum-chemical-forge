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
set(FERRUM_CHEM_RDKIT_DEPICTOR "" CACHE FILEPATH "Explicit libRDKitDepictor dylib")
set(FERRUM_CHEM_RDKIT_SMILESPARSE "" CACHE FILEPATH "Explicit libRDKitSmilesParse dylib")
set(FERRUM_CHEM_RDKIT_FILEPARSERS "" CACHE FILEPATH "Explicit libRDKitFileParsers dylib")
set(FERRUM_CHEM_RDKIT_RDINCHI "" CACHE FILEPATH "Explicit libRDKitRDInchiLib dylib")

foreach(required_input IN ITEMS FERRUM_CHEM_RDKIT_INCLUDE_DIR FERRUM_CHEM_BOOST_INCLUDE_DIR
		FERRUM_CHEM_RDKIT_GRAPHMOL FERRUM_CHEM_RDKIT_RDGENERAL
		FERRUM_CHEM_RDKIT_DEPICTOR FERRUM_CHEM_RDKIT_SMILESPARSE
		FERRUM_CHEM_RDKIT_FILEPARSERS
		FERRUM_CHEM_RDKIT_RDINCHI
	)
	if(NOT ${required_input})
		message(FATAL_ERROR "${required_input} is required; Ferrum never discovers host RDKit")
	endif()
	if(NOT EXISTS "${${required_input}}")
		message(FATAL_ERROR "${required_input} does not exist: ${${required_input}}")
	endif()
endforeach()
if(NOT EXISTS "${FERRUM_CHEM_RDKIT_INCLUDE_DIR}/GraphMol/MolOps.h")
	message(FATAL_ERROR "FERRUM_CHEM_RDKIT_INCLUDE_DIR lacks GraphMol/MolOps.h")
endif()
if(NOT EXISTS "${FERRUM_CHEM_RDKIT_INCLUDE_DIR}/RDGeneral/types.h")
	message(FATAL_ERROR "FERRUM_CHEM_RDKIT_INCLUDE_DIR lacks RDGeneral/types.h")
endif()
if(NOT EXISTS "${FERRUM_CHEM_BOOST_INCLUDE_DIR}/boost/config.hpp")
	message(FATAL_ERROR "FERRUM_CHEM_BOOST_INCLUDE_DIR lacks boost/config.hpp")
endif()

add_library(ferrum_chem SHARED
	ferrum_chem_adapter.cpp ferrum_chem_smarts.cpp ferrum_chem_smiles.cpp
	ferrum_chem_molblock.cpp
	ferrum_chem_molblock_import.cpp ferrum_chem_sdf.cpp ferrum_chem_sdf_import.cpp
	ferrum_chem_molecule_response.cpp
	ferrum_chem_text_response.cpp ferrum_chem_inchi.cpp ferrum_chem_composition.cpp)
target_include_directories(ferrum_chem
	PUBLIC "$<BUILD_INTERFACE:${CMAKE_CURRENT_SOURCE_DIR}/include>"
	PRIVATE "${FERRUM_CHEM_RDKIT_INCLUDE_DIR}" "${FERRUM_CHEM_BOOST_INCLUDE_DIR}")
target_link_libraries(ferrum_chem PRIVATE
	"${FERRUM_CHEM_RDKIT_GRAPHMOL}"
	"${FERRUM_CHEM_RDKIT_RDGENERAL}")
target_compile_definitions(ferrum_chem PRIVATE FERRUM_CHEM_ENABLE_DEPICTOR=1 FERRUM_CHEM_ENABLE_SMILES=1)
target_link_libraries(ferrum_chem PRIVATE
	"${FERRUM_CHEM_RDKIT_DEPICTOR}" "${FERRUM_CHEM_RDKIT_SMILESPARSE}"
	"${FERRUM_CHEM_RDKIT_FILEPARSERS}")
target_link_libraries(ferrum_chem PRIVATE "${FERRUM_CHEM_RDKIT_RDINCHI}")
target_link_options(ferrum_chem PRIVATE "-Wl,-rpath,@loader_path")
set_target_properties(ferrum_chem PROPERTIES
	OUTPUT_NAME ferrum_chem
	MACOSX_RPATH ON
	INSTALL_NAME_DIR "@rpath")

install(TARGETS ferrum_chem LIBRARY DESTINATION lib)
install(FILES include/ferrum_chem_adapter.h DESTINATION include)
