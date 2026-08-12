/* Ferrum-Chem C adapter ABI. RDKit types never cross this boundary. */
#ifndef FERRUM_CHEM_ADAPTER_H
#define FERRUM_CHEM_ADAPTER_H

#include <stdint.h>

/* The public header is the sole source of truth for the adapter ABI version. */
#define FERRUM_CHEM_ADAPTER_ABI_VERSION 3U

#ifdef __cplusplus
#define FERRUM_CHEM_NOEXCEPT noexcept
extern "C" {
#else
#define FERRUM_CHEM_NOEXCEPT
#endif

#define FERRUM_CHEM_CALL_OK 0U
#define FERRUM_CHEM_CALL_INVALID_ARGUMENT 1U
#define FERRUM_CHEM_CALL_ALLOCATION_FAILURE 2U
#define FERRUM_CHEM_CALL_INTERNAL_FAILURE 3U

#define FERRUM_CHEM_RESULT_OK 0U
#define FERRUM_CHEM_RESULT_MALFORMED_REQUEST 1U
#define FERRUM_CHEM_RESULT_INVALID_MOLECULE 2U
#define FERRUM_CHEM_RESULT_KEKULIZE_FAILURE 3U
#define FERRUM_CHEM_RESULT_INTERNAL_FAILURE 4U

/* ABI-3 capability bits. Each adapter exports its supported subset. */
#define FERRUM_CHEM_CAPABILITY_KEKULIZE 0x0000000000000001ULL
#define FERRUM_CHEM_CAPABILITY_SMILES 0x0000000000000002ULL
#define FERRUM_CHEM_CAPABILITY_GENERATE_2D 0x0000000000000004ULL
#define FERRUM_CHEM_CAPABILITY_SMARTS 0x0000000000000008ULL
#define FERRUM_CHEM_CAPABILITY_MOLFILE 0x0000000000000010ULL
#define FERRUM_CHEM_CAPABILITY_SDF 0x0000000000000020ULL
#define FERRUM_CHEM_CAPABILITY_INCHI 0x0000000000000040ULL

/* Every adapter-owned result must fit this bound before a consumer reads it. */
#define FERRUM_CHEM_MAX_RESPONSE_BYTES 40000000U

/* Canonical Kekulize wire constants shared by C++ and generated Rust code. */
#define FERRUM_CHEM_KEKULIZE_WIRE_VERSION 1U
#define FERRUM_CHEM_KEKULIZE_OPTION_CLEAR_AROMATIC_FLAGS 0x00000001U
#define FERRUM_CHEM_KEKULIZE_OPTION_CANONICAL 0x00000002U
#define FERRUM_CHEM_KEKULIZE_FACT_FORMAL_CHARGE 0x0001U
#define FERRUM_CHEM_KEKULIZE_FACT_ISOTOPE 0x0002U
#define FERRUM_CHEM_KEKULIZE_FACT_EXPLICIT_HYDROGENS 0x0004U
#define FERRUM_CHEM_KEKULIZE_MAX_BACKTRACKS 1000000U
#define FERRUM_CHEM_KEKULIZE_MAX_ATOMS 1000000U
#define FERRUM_CHEM_KEKULIZE_MAX_BONDS 2000000U
#define FERRUM_CHEM_KEKULIZE_MAX_DETAIL_BYTES 4096U
#define FERRUM_CHEM_KEKULIZE_REQUEST_HEADER_BYTES 24U
#define FERRUM_CHEM_KEKULIZE_RESPONSE_HEADER_BYTES 32U
#define FERRUM_CHEM_KEKULIZE_ATOM_BYTES 12U
#define FERRUM_CHEM_KEKULIZE_BOND_BYTES 12U

/* FCS1 limits are independently validated by its Rust decoder. */
#define FERRUM_CHEM_SMILES_MAX_BYTES 1048576U
#define FERRUM_CHEM_SMILES_RESPONSE_HEADER_BYTES 24U
#define FERRUM_CHEM_COORDINATE_BYTES 16U

/*
 * Bond-type byte values are wire values, not RDKit enum values.  UNSPECIFIED
 * is the adapter's invalid/unsupported sentinel. QUADRUPLE is reserved by the
 * v1 record vocabulary but is not accepted by the current Kekulize operation;
 * this preserves the existing v1 behavior while giving later operations a
 * named, stable code. Aromatic remains 4 for wire compatibility.
 */
#define FERRUM_CHEM_KEKULIZE_BOND_TYPE_UNSPECIFIED 0U
#define FERRUM_CHEM_KEKULIZE_BOND_TYPE_SINGLE 1U
#define FERRUM_CHEM_KEKULIZE_BOND_TYPE_DOUBLE 2U
#define FERRUM_CHEM_KEKULIZE_BOND_TYPE_TRIPLE 3U
#define FERRUM_CHEM_KEKULIZE_BOND_TYPE_AROMATIC 4U
#define FERRUM_CHEM_KEKULIZE_BOND_TYPE_QUADRUPLE 5U

/* `data` is adapter-owned. Every output must be released with the v1 free API. */
typedef struct ferrum_chem_owned_buffer {
	uint8_t *data;
	uint64_t len;
} ferrum_chem_owned_buffer;

/*
 * Kekulize v1 wire protocol. All integer fields are little-endian. The caller
 * borrows the request for the duration of the call; the adapter owns response.
 *
 * Request header (24 bytes):
 *   [0]  "FCK1" magic bytes
 *   [4]  wire version:u32 (1)
 *   [8]  options:u32: bit 0 clear_aromatic_flags, bit 1 canonical;
 *        all remaining bits are reserved and must be zero
 *   [12] max_backtracks:u32 (explicit, 1 through 1,000,000)
 *   [16] atom_count:u32 (at most 1,000,000)
 *   [20] bond_count:u32 (at most 2,000,000)
 *
 * Atom record (12 bytes): atomic_number:u8, aromatic:u8 (0 or 1),
 * presence_flags:u16, formal_charge:i32, isotope:u16, explicit_hydrogens:u16.
 * presence_flags: FORMAL_CHARGE=0x0001, ISOTOPE=0x0002, EXPLICIT_HYDROGENS=
 * 0x0004. Reserved flags must be zero, and fields without their presence bit
 * must be zero. A present isotope must be nonzero. Atomic number is 1..118.
 *
 * Bond record (12 bytes): begin_atom:u32, end_atom:u32, bond_type:u8,
 * aromatic:u8 (0 or 1), reserved:u16. The bond_type byte is one of the
 * FERRUM_CHEM_KEKULIZE_BOND_TYPE_* macros. Kekulize v1 accepts SINGLE,
 * DOUBLE, TRIPLE, and AROMATIC; UNSPECIFIED and QUADRUPLE are rejected.
 * Request bonds are strict: aromatic=1 requires bond_type=AROMATIC, and
 * bond_type=AROMATIC requires aromatic=1. A successful response may retain
 * aromatic=1 on its resulting SINGLE or DOUBLE bonds when
 * clear_aromatic_flags is false.
 *
 * Response header (32 bytes): "FCR1", wire_version:u32, result_status:u32,
 * detail_length:u32, echoed_options:u32, echoed_max_backtracks:u32,
 * atom_count:u32, bond_count:u32; followed by UTF-8 detail bytes and the atom
 * and bond records above. Error responses have zero options, max-backtracks,
 * and topology. Successful responses echo every input atom fact and topology,
 * with RDKit's updated atom aromatic flags and bond order/aromatic facts.
 * Coordinates stay in safe Rust.
 * The adapter constructs only these facts and calls Kekulize with the explicit
 * request options; it never performs a sanitization pass.
 */

uint32_t ferrum_chem_abi_version(void) FERRUM_CHEM_NOEXCEPT;

/* Returns the explicitly compiled ABI-3 operation set. */
uint64_t ferrum_chem_capabilities_v1(void) FERRUM_CHEM_NOEXCEPT;

/* A zero call status always returns a structured response, including errors. */
uint32_t ferrum_chem_kekulize_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-3 generates a deterministic 2D depiction from the strict graph request.
 *
 * Generate-2D v1 receives the same strict graph record vocabulary as
 * Kekulize v1. Its request always has option bits zero and max_backtracks one:
 * those fields are a graph-envelope parser sentinel, not Kekulize controls.
 * It calls RDKit's built-in depicter with canonOrient=true,
 * forceRDKit=true, clearConfs=true, no random samples, and no templates.
 *
 * Response: "FCL1", wire_version:u32 (1), result_status:u32,
 * detail_length:u32, atom_count:u32, UTF-8 detail, then atom_count records of
 * x:f64,y:f64 little-endian. Success has an empty detail and preserves input
 * atom order. Failure has atom_count zero and no coordinate records.
 */
uint32_t ferrum_chem_generate_2d_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-3 SMILES vertical slice. Request is non-empty UTF-8 SMILES with no NUL
 * bytes and a maximum of FERRUM_CHEM_SMILES_MAX_BYTES. Response is "FCS1",
 * wire_version:u32 (1), result_status:u32,
 * detail_length:u32, smiles_length:u32, atom_count:u32, UTF-8 detail,
 * canonical UTF-8 SMILES, then atom_count x:f64,y:f64 records. Success has
 * empty detail and coordinates in RDKit atom order. The envelope is deliberately
 * extensible: SMARTS, mol/SDF, and InChI operations use distinct entry points
 * and response magics rather than overloading this record.
 */
uint32_t ferrum_chem_smiles_to_2d_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/* Idempotently releases a response owner and writes both fields to zero. */
void ferrum_chem_owned_buffer_free_v1(ferrum_chem_owned_buffer *owner)
	FERRUM_CHEM_NOEXCEPT;

#ifdef __cplusplus
}
#endif

#endif
