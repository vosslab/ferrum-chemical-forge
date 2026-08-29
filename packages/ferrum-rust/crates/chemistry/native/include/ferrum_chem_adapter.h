/* Ferrum-Chem C adapter ABI. RDKit types never cross this boundary. */
#ifndef FERRUM_CHEM_ADAPTER_H
#define FERRUM_CHEM_ADAPTER_H

#include <stdint.h>

/* The public header is the sole source of truth for the adapter ABI version. */
#define FERRUM_CHEM_ADAPTER_ABI_VERSION 6U

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
#define FERRUM_CHEM_RESULT_DEPICTION_FAILURE 3U
#define FERRUM_CHEM_RESULT_RESOURCE_LIMIT 4U
#define FERRUM_CHEM_RESULT_UNSUPPORTED_MOLECULE 5U
#define FERRUM_CHEM_RESULT_INTERNAL_FAILURE 6U

/* ABI-6 capability bits. Each adapter exports its supported subset. */
#define FERRUM_CHEM_CAPABILITY_KEKULIZE 0x0000000000000001ULL
#define FERRUM_CHEM_CAPABILITY_SMILES_MOLECULE 0x0000000000000002ULL
#define FERRUM_CHEM_CAPABILITY_GENERATE_2D 0x0000000000000004ULL
#define FERRUM_CHEM_CAPABILITY_SMARTS 0x0000000000000008ULL
#define FERRUM_CHEM_CAPABILITY_MOLFILE 0x0000000000000010ULL
#define FERRUM_CHEM_CAPABILITY_SDF_WRITE 0x0000000000000020ULL
#define FERRUM_CHEM_CAPABILITY_INCHI 0x0000000000000040ULL
#define FERRUM_CHEM_CAPABILITY_SDF_READ 0x0000000000000080ULL
#define FERRUM_CHEM_CAPABILITY_MOLFILE_READ 0x0000000000000100ULL
#define FERRUM_CHEM_CAPABILITY_COMPOSITION 0x0000000000000200ULL
#define FERRUM_CHEM_CAPABILITY_SMILES_WRITE 0x0000000000000400ULL
#define FERRUM_CHEM_CAPABILITY_MOLFILE_TITLE 0x0000000000000800ULL
#define FERRUM_CHEM_CAPABILITY_SMARTS_MATCH 0x0000000000001000ULL

/* ABI-6 bounded SMARTS matching FCQ1/FQM1 constants. */
#define FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION 1U
#define FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES 24U
#define FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES 28U
#define FERRUM_CHEM_SMARTS_MATCH_MAX_QUERY_BYTES 65536U
#define FERRUM_CHEM_SMARTS_MATCH_MAX_PARSED_QUERY_ATOMS 1024U
#define FERRUM_CHEM_SMARTS_MATCH_PROFILE_MAX_QUERY_ATOMS 64U
#define FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS 10000U
#define FERRUM_CHEM_SMARTS_MATCH_MAX_MATRIX_CELLS 1000000U
#define FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED 0x00000001U
#define FERRUM_CHEM_SMARTS_MATCH_STATUS_OK 0U
#define FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST 1U
#define FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY 2U
#define FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET 3U
#define FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED 4U
#define FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE 5U

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

/* FCM1 numeric vocabulary is independently validated by its Rust decoder. */
#define FERRUM_CHEM_MOLECULE_WIRE_VERSION 1U
#define FERRUM_CHEM_MOLECULE_FLAGS_NONE 0U
#define FERRUM_CHEM_MOLECULE_RESERVED 0U
#define FERRUM_CHEM_MOLECULE_STEREO_REFERENCE_NONE 0xFFFFFFFFU
#define FERRUM_CHEM_SMILES_MAX_BYTES 1048576U
#define FERRUM_CHEM_MOLECULE_RESPONSE_HEADER_BYTES 32U
#define FERRUM_CHEM_MOLECULE_ATOM_BYTES 20U
#define FERRUM_CHEM_MOLECULE_BOND_BYTES 24U
#define FERRUM_CHEM_COORDINATE_BYTES 16U

/* Complete graph input and bounded text output shared by ABI-6 codecs.
 * Every text writer requires a nonzero maximum_text_bytes no greater than
 * FERRUM_CHEM_MAX_RESPONSE_BYTES - FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES.
 * The FCT1 envelope reserves its header from the adapter-wide response limit;
 * larger or insufficient budgets refuse with FERRUM_CHEM_RESULT_RESOURCE_LIMIT
 * before the native writer is invoked. */
#define FERRUM_CHEM_GRAPH_WIRE_VERSION 1U
#define FERRUM_CHEM_GRAPH_FLAGS_NONE 0U
#define FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES 20U
#define FERRUM_CHEM_GRAPH_ATOM_BYTES 24U
#define FERRUM_CHEM_GRAPH_BOND_BYTES 24U
#define FERRUM_CHEM_TEXT_WIRE_VERSION 1U
#define FERRUM_CHEM_TEXT_FLAGS_NONE 0U
#define FERRUM_CHEM_TEXT_RESPONSE_HEADER_BYTES 24U
#define FERRUM_CHEM_SMILES_WRITE_MAX_BYTES 39999976U

/* Bounded isotope-aware molecule composition response. */
#define FERRUM_CHEM_COMPOSITION_WIRE_VERSION 1U
#define FERRUM_CHEM_COMPOSITION_FLAGS_NONE 0U
#define FERRUM_CHEM_COMPOSITION_RESPONSE_HEADER_BYTES 56U
#define FERRUM_CHEM_COMPOSITION_ENTRY_BYTES 24U
#define FERRUM_CHEM_COMPOSITION_MAX_DETAIL_BYTES 4096U
#define FERRUM_CHEM_COMPOSITION_MAX_FORMULA_BYTES 16000000U

/* Closed InChI export request around one complete FCG1 graph. */
#define FERRUM_CHEM_INCHI_WIRE_VERSION 1U
#define FERRUM_CHEM_INCHI_MODE_STANDARD 1U
#define FERRUM_CHEM_INCHI_MODE_FIXED_HYDROGEN 2U
#define FERRUM_CHEM_INCHI_FLAGS_NONE 0U
#define FERRUM_CHEM_INCHI_REQUEST_HEADER_BYTES 20U
#define FERRUM_CHEM_INCHI_MAX_BYTES 40000000U
#define FERRUM_CHEM_INCHI_KEY_BYTES 27U

/* Molblock export carries one complete graph plus atom-aligned 2D coordinates. */
#define FERRUM_CHEM_MOLBLOCK_WIRE_VERSION 1U
#define FERRUM_CHEM_MOLBLOCK_FORMAT_V2000 1U
#define FERRUM_CHEM_MOLBLOCK_FORMAT_V3000 2U
#define FERRUM_CHEM_MOLBLOCK_FLAGS_NONE 0U
#define FERRUM_CHEM_MOLBLOCK_REQUEST_HEADER_BYTES 24U
#define FERRUM_CHEM_TITLED_MOLBLOCK_WIRE_VERSION 1U
#define FERRUM_CHEM_TITLED_MOLBLOCK_REQUEST_HEADER_BYTES 16U

/* Ordered multi-record SDF export embeds one FCB1 request per record. */
#define FERRUM_CHEM_SDF_WIRE_VERSION 1U
#define FERRUM_CHEM_SDF_FLAGS_NONE 0U
#define FERRUM_CHEM_SDF_REQUEST_HEADER_BYTES 16U
#define FERRUM_CHEM_SDF_RECORD_HEADER_BYTES 16U
#define FERRUM_CHEM_SDF_PROPERTY_HEADER_BYTES 8U
#define FERRUM_CHEM_SDF_RESPONSE_HEADER_BYTES 24U
#define FERRUM_CHEM_SDF_MAX_RECORDS 100000U
#define FERRUM_CHEM_SDF_MAX_PROPERTIES 1000000U

/* Stable ABI-6 vocabulary; these are wire values, never RDKit enum casts. */
#define FERRUM_CHEM_CHIRAL_UNSPECIFIED 0U
#define FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CW 1U
#define FERRUM_CHEM_CHIRAL_TETRAHEDRAL_CCW 2U
#define FERRUM_CHEM_CHIRAL_OTHER 3U
#define FERRUM_CHEM_BOND_STEREO_NONE 0U
#define FERRUM_CHEM_BOND_STEREO_ANY 1U
#define FERRUM_CHEM_BOND_STEREO_Z 2U
#define FERRUM_CHEM_BOND_STEREO_E 3U
#define FERRUM_CHEM_BOND_STEREO_CIS 4U
#define FERRUM_CHEM_BOND_STEREO_TRANS 5U
#define FERRUM_CHEM_BOND_STEREO_OTHER 6U
#define FERRUM_CHEM_BOND_DIRECTION_NONE 0U
#define FERRUM_CHEM_BOND_DIRECTION_BEGINWEDGE 1U
#define FERRUM_CHEM_BOND_DIRECTION_BEGINDASH 2U
#define FERRUM_CHEM_BOND_DIRECTION_ENDUPRIGHT 3U
#define FERRUM_CHEM_BOND_DIRECTION_ENDDOWNRIGHT 4U
#define FERRUM_CHEM_BOND_DIRECTION_OTHER 5U

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

/* Returns the explicitly compiled ABI-6 operation set. */
uint64_t ferrum_chem_capabilities_v1(void) FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 FCQ1/FQM1 SMARTS matching. FCQ1 is exactly consumed: "FCQ1",
 * wire_version:u32, smarts_length:u32, fcg1_length:u32, max_matches:u32,
 * flags:u32 (zero), then non-empty UTF-8 SMARTS and one complete FCG1 graph.
 * FQM1 is "FQM1", wire_version:u32, result_status:u32, detail_length:u32,
 * query_atom_count:u32, match_count:u32, flags:u32, fixed ASCII detail, then
 * query-ordered target-index u32 rows. This entry point never exposes RDKit
 * diagnostics: all non-success details are closed contract codes.
 */
uint32_t ferrum_chem_smarts_match_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/* A zero call status always returns a structured response, including errors. */
uint32_t ferrum_chem_kekulize_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 generates a deterministic 2D depiction from the strict graph request.
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
 * ABI-6 complete SMILES molecule slice. Request is non-empty UTF-8 SMILES with
 * no NUL bytes and at most FERRUM_CHEM_SMILES_MAX_BYTES. A zero call status
 * returns FCM1: a 32-byte little-endian header followed by UTF-8 detail,
 * canonical SMILES, atom records, bond records, and atom-aligned finite x/y
 * records. Header fields are magic, wire_version, result_status, detail_len,
 * smiles_len, atom_count, bond_count, flags (zero). All atom/bond reserved
 * fields and flags are zero. Errors have non-empty detail and no graph data.
 */
uint32_t ferrum_chem_smiles_to_molecule_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 SMARTS export receives FCG1, Ferrum's complete graph envelope. All
 * integer fields are little-endian. Coordinates are intentionally absent.
 *
 * FCG1 header (20 bytes): magic, wire_version, atom_count, bond_count, flags.
 * flags must be FERRUM_CHEM_GRAPH_FLAGS_NONE. Atom records contain atomic
 * number:u8, aromatic:u8, chirality:u8, reserved:u8, presence_flags:u32,
 * formal_charge:i32, isotope:u16, explicit_hydrogens:u16,
 * radical_electrons:u8, no_implicit:u8, reserved:u16, atom_map_number:u32.
 * Presence flags use the FERRUM_CHEM_KEKULIZE_FACT_* bits. Bond records use
 * the FCM1 24-byte bond vocabulary and stereo-reference sentinel.
 *
 * FCT1 response (24 bytes): magic, wire_version, result_status,
 * detail_length, text_length, flags, followed by UTF-8 detail and text. A
 * successful response has empty detail and nonempty SMARTS. A failed response
 * has nonempty detail and empty text. All returned bytes are adapter-owned.
 */
uint32_t ferrum_chem_molecule_to_smarts_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 canonical-isomeric SMILES export receives one complete FCG1 graph.
 * The adapter sanitizes its private molecule and invokes the pinned writer with
 * isomeric=true, Kekule=false, canonical=true, cleanStereo=true, ordinary
 * implicit bond/hydrogen spelling, random=false, rootedAtAtom=-1, dative bonds
 * enabled, and atom-map numbers included in canonicalization. FCT1 returns one
 * nonempty printable ASCII line of at most FERRUM_CHEM_SMILES_WRITE_MAX_BYTES.
 */
uint32_t ferrum_chem_molecule_to_smiles_v1(
	const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
	ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 molecule composition receives one complete FCG1 graph and sanitizes a
 * private RDKit molecule. FCS1 has a 56-byte little-endian header: magic,
 * wire_version, result_status, detail_length, formula_length, entry_count,
 * flags, reserved, net_charge:i64, average_mass:f64, exact_mass:f64. It is
 * followed by UTF-8 detail, UTF-8 isotope-aware charged formula, and 24-byte
 * entries: atomic_number:u8, isotope_present:u8, reserved:u16, isotope:u16,
 * reserved:u16, count:u64, average_mass_contribution:f64. Success has empty
 * detail and nonempty formula/entries; failure has only nonempty detail.
 */
uint32_t ferrum_chem_molecule_composition_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 molblock export receives FCB1, a coordinate-bearing complete graph.
 * Its 24-byte little-endian header contains magic, wire_version, format,
 * atom_count, bond_count, and zero flags. Atom and bond records use FCG1's
 * exact vocabulary, followed by one finite x:f64,y:f64 record per atom.
 * Format is explicitly V2000 or V3000; the adapter never silently promotes a
 * requested V2000 result. FCT1 returns the multiline UTF-8 molblock text.
 */
uint32_t ferrum_chem_molecule_to_molblock_v1(
	const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
	ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * Optional ABI-6 titled molblock export receives FBT1. Its 16-byte header
 * contains magic, wire_version, embedded_FCB1_length, and UTF-8 title length,
 * followed by exactly those two byte sequences. The title may be empty but
 * must contain no NUL or line-break bytes. FCT1 returns the molblock with that
 * exact title as its first line.
 */
uint32_t ferrum_chem_molecule_to_molblock_with_title_v1(
	const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
	ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 SDF export receives FSD1. Its 16-byte header contains magic,
 * wire_version, record_count, and zero flags. Each record has FCB1 length,
 * title length, property count (at most FERRUM_CHEM_SDF_MAX_PROPERTIES), and zero flags, followed by one complete FCB1
 * request, UTF-8 title bytes, then ordered property pairs. Each pair has
 * name_length and value_length followed by UTF-8 name and value bytes. Names
 * are unique per record. FCT1 returns newline-terminated multi-record SDF text.
 */
uint32_t ferrum_chem_records_to_sdf_v1(
	const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
	ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 SDF import accepts bounded UTF-8 SDF text and returns FSI1. Its
 * 24-byte little-endian header contains magic, wire_version, result_status,
 * detail_length, record_count, and zero flags. Each successful record has a
 * 16-byte header containing an embedded FCM1 length, title length, property
 * count, and zero flags, followed by the complete FCM1 molecule, UTF-8 title,
 * and ordered property pairs using FSD1's property header. Duplicate property
 * names remain separate ordered entries. Three-dimensional conformers are
 * rejected because Ferrum's current owned graph contains Point2 coordinates.
 */
uint32_t ferrum_chem_sdf_to_records_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 molblock import accepts one bounded UTF-8 V2000 or V3000 molblock and
 * returns the same complete FCM1 owned-molecule envelope used by SMILES.
 * Strict parsing and sanitization are enabled, authored hydrogen atoms remain
 * explicit, and text after the sole M  END terminator is rejected. A
 * three-dimensional conformer is rejected because Ferrum's current owned
 * graph contains Point2 coordinates.
 */
uint32_t ferrum_chem_molblock_to_molecule_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 InChI import accepts one bounded ASCII InChI=1S/ or InChI=1/ line,
 * sanitizes it through RDKit's bundled InChI implementation, generates
 * deterministic two-dimensional coordinates, and returns FCM1. The adapter
 * rejects NUL, control bytes, line breaks, empty molecules, and oversized
 * input before invoking InChI.
 */
uint32_t ferrum_chem_inchi_to_molecule_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 InChI export accepts FCI1: magic, wire_version, closed output mode,
 * embedded FCG1 byte length, and zero flags, followed by exactly one complete
 * FCG1 graph. STANDARD requires an InChI=1S/ result. FIXED_HYDROGEN passes only
 * the fixed-hydrogen InChI option and requires the non-standard InChI=1/
 * prefix. FCT1 returns one nonempty InChI line; caller-supplied option strings
 * never cross the ABI.
 */
uint32_t ferrum_chem_molecule_to_inchi_v1(
	const uint8_t *request, uint64_t request_len, uint64_t maximum_text_bytes,
	ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/*
 * ABI-6 InChIKey generation accepts the same validated bounded InChI line as
 * import. FCT1 returns the official 27-byte uppercase InChIKey layout. A key
 * derived from InChI=1S/ carries the standard marker; InChI=1/ carries the
 * non-standard marker. The adapter exposes no toolkit option string.
 */
uint32_t ferrum_chem_inchi_to_inchi_key_v1(
	const uint8_t *request, uint64_t request_len, ferrum_chem_owned_buffer *response)
	FERRUM_CHEM_NOEXCEPT;

/* Idempotently releases a response owner and writes both fields to zero. */
void ferrum_chem_owned_buffer_free_v1(ferrum_chem_owned_buffer *owner)
	FERRUM_CHEM_NOEXCEPT;

#ifdef __cplusplus
}
#endif

#endif
