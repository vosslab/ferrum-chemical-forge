//! Private FCQ1/FQM1 transport for the owned SMARTS matcher.
//!
//! No byte sequence, native index, loader, or adapter allocation leaves this module.

use std::collections::HashSet;

use crate::{
    ChemistryError, FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES, FERRUM_CHEM_MAX_RESPONSE_BYTES,
    FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED, FERRUM_CHEM_SMARTS_MATCH_MAX_MATRIX_CELLS,
    FERRUM_CHEM_SMARTS_MATCH_MAX_QUERY_BYTES, FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS,
    FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES, FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY, FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE, FERRUM_CHEM_SMARTS_MATCH_STATUS_OK,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED,
    FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET, FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION,
    MolGraph, SmartsMatchOptions, SmartsMatchResult, SmartsMatchUnavailableReason,
};

use super::graph_wire;

const REQUEST_MAGIC: [u8; 4] = *b"FCQ1";
const RESPONSE_MAGIC: [u8; 4] = *b"FQM1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum WireError {
    NativeRejected,
    MalformedResponse,
}

pub(super) fn map_wire_error(error: WireError) -> ChemistryError {
    let reason = match error {
        WireError::NativeRejected => SmartsMatchUnavailableReason::NativeRejected,
        WireError::MalformedResponse => SmartsMatchUnavailableReason::MalformedNativeResponse,
    };
    ChemistryError::SmartsMatchUnavailable { reason }
}

pub(super) fn encode_request(
    query: &str,
    target: &MolGraph,
    options: SmartsMatchOptions,
) -> Result<Vec<u8>, WireError> {
    let bytes = query.as_bytes();
    if bytes.is_empty()
        || bytes.contains(&0)
        || bytes.len() > FERRUM_CHEM_SMARTS_MATCH_MAX_QUERY_BYTES
        || target.atoms().is_empty()
    {
        return Err(WireError::NativeRejected);
    }
    let graph = graph_wire::encode(target).map_err(|_| WireError::NativeRejected)?;
    let graph_len = u32::try_from(graph.len()).map_err(|_| WireError::NativeRejected)?;
    let query_len = u32::try_from(bytes.len()).map_err(|_| WireError::NativeRejected)?;
    let mut request = Vec::with_capacity(
        FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES + bytes.len() + graph.len(),
    );
    request.extend_from_slice(&REQUEST_MAGIC);
    put_u32(&mut request, FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION);
    put_u32(&mut request, query_len);
    put_u32(&mut request, graph_len);
    put_u32(&mut request, options.max_matches());
    put_u32(&mut request, 0);
    request.extend_from_slice(bytes);
    request.extend_from_slice(&graph);
    debug_assert_eq!(FERRUM_CHEM_SMARTS_MATCH_REQUEST_HEADER_BYTES, 24);
    debug_assert!(graph.len() >= FERRUM_CHEM_GRAPH_REQUEST_HEADER_BYTES);
    Ok(request)
}

pub(super) fn decode_response(
    bytes: &[u8],
    target_atom_count: usize,
    requested_max_matches: u32,
) -> Result<SmartsMatchResult, WireError> {
    if bytes.len() < FERRUM_CHEM_SMARTS_MATCH_RESPONSE_HEADER_BYTES
        || bytes.len() > FERRUM_CHEM_MAX_RESPONSE_BYTES
    {
        return Err(WireError::MalformedResponse);
    }
    let mut cursor = 0;
    if take(bytes, &mut cursor, 4)? != RESPONSE_MAGIC {
        return Err(WireError::MalformedResponse);
    }
    if read_u32(bytes, &mut cursor)? != FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION {
        return Err(WireError::MalformedResponse);
    }
    let status = read_u32(bytes, &mut cursor)?;
    let detail_length =
        usize::try_from(read_u32(bytes, &mut cursor)?).map_err(|_| WireError::MalformedResponse)?;
    let query_atom_count =
        usize::try_from(read_u32(bytes, &mut cursor)?).map_err(|_| WireError::MalformedResponse)?;
    let match_count =
        usize::try_from(read_u32(bytes, &mut cursor)?).map_err(|_| WireError::MalformedResponse)?;
    let flags = read_u32(bytes, &mut cursor)?;
    let detail = take(bytes, &mut cursor, detail_length)?;

    if status != FERRUM_CHEM_SMARTS_MATCH_STATUS_OK {
        let legal = match status {
            FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_REQUEST => {
                detail == b"invalid_request" || detail == b"query_atom_limit_exceeded"
            }
            FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY => detail == b"invalid_query",
            FERRUM_CHEM_SMARTS_MATCH_STATUS_UNSUPPORTED_TARGET => detail == b"unsupported_target",
            FERRUM_CHEM_SMARTS_MATCH_STATUS_RESOURCE_LIMITED => detail == b"resource_limited",
            FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE => detail == b"native_failure",
            _ => false,
        };
        if !legal
            || query_atom_count != 0
            || match_count != 0
            || flags != 0
            || cursor != bytes.len()
        {
            return Err(WireError::MalformedResponse);
        }
        return Err(WireError::NativeRejected);
    }

    if !detail.is_empty()
        || query_atom_count == 0
        || match_count > usize::try_from(FERRUM_CHEM_SMARTS_MATCH_MAX_ROWS).expect("u32 fits usize")
        || match_count > usize::try_from(requested_max_matches).expect("u32 fits usize")
        || flags & !FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED != 0
    {
        return Err(WireError::MalformedResponse);
    }
    let cells = query_atom_count
        .checked_mul(match_count)
        .ok_or(WireError::MalformedResponse)?;
    if cells > usize::try_from(FERRUM_CHEM_SMARTS_MATCH_MAX_MATRIX_CELLS).expect("u32 fits usize") {
        return Err(WireError::MalformedResponse);
    }
    let wire_bytes = cells.checked_mul(4).ok_or(WireError::MalformedResponse)?;
    if bytes.len().checked_sub(cursor) != Some(wire_bytes) {
        return Err(WireError::MalformedResponse);
    }
    let mut rows = Vec::with_capacity(match_count);
    for _ in 0..match_count {
        let mut row = Vec::with_capacity(query_atom_count);
        let mut unique = HashSet::with_capacity(query_atom_count);
        for _ in 0..query_atom_count {
            let position = usize::try_from(read_u32(bytes, &mut cursor)?)
                .map_err(|_| WireError::MalformedResponse)?;
            if position >= target_atom_count || !unique.insert(position) {
                return Err(WireError::MalformedResponse);
            }
            row.push(position);
        }
        rows.push(row);
    }
    if cursor != bytes.len() {
        return Err(WireError::MalformedResponse);
    }
    Ok(SmartsMatchResult::new(
        rows,
        flags == FERRUM_CHEM_SMARTS_MATCH_FLAG_TRUNCATED,
    ))
}

fn take<'a>(bytes: &'a [u8], cursor: &mut usize, length: usize) -> Result<&'a [u8], WireError> {
    let end = cursor
        .checked_add(length)
        .ok_or(WireError::MalformedResponse)?;
    let result = bytes
        .get(*cursor..end)
        .ok_or(WireError::MalformedResponse)?;
    *cursor = end;
    Ok(result)
}

fn read_u32(bytes: &[u8], cursor: &mut usize) -> Result<u32, WireError> {
    Ok(u32::from_le_bytes(
        take(bytes, cursor, 4)?
            .try_into()
            .expect("fixed-width slice"),
    ))
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(
        status: u32,
        detail: &[u8],
        query_atoms: u32,
        rows: &[&[u32]],
        flags: u32,
    ) -> Vec<u8> {
        let mut output = Vec::from(RESPONSE_MAGIC);
        put_u32(&mut output, FERRUM_CHEM_SMARTS_MATCH_WIRE_VERSION);
        put_u32(&mut output, status);
        put_u32(
            &mut output,
            u32::try_from(detail.len()).expect("small detail"),
        );
        put_u32(&mut output, query_atoms);
        put_u32(&mut output, u32::try_from(rows.len()).expect("few rows"));
        put_u32(&mut output, flags);
        output.extend_from_slice(detail);
        for row in rows {
            for position in *row {
                put_u32(&mut output, *position);
            }
        }
        output
    }

    #[test]
    fn decoder_returns_owned_query_ordered_caller_positions() {
        let bytes = response(FERRUM_CHEM_SMARTS_MATCH_STATUS_OK, b"", 2, &[&[3, 1]], 0);
        assert_eq!(
            decode_response(&bytes, 4, 1),
            Ok(SmartsMatchResult::new(vec![vec![3, 1]], false))
        );
    }

    #[test]
    fn decoder_rejects_duplicate_rows_trailing_bytes_and_hostile_detail() {
        let duplicate = response(FERRUM_CHEM_SMARTS_MATCH_STATUS_OK, b"", 2, &[&[1, 1]], 0);
        assert_eq!(
            decode_response(&duplicate, 3, 1),
            Err(WireError::MalformedResponse)
        );
        let mut trailing = response(FERRUM_CHEM_SMARTS_MATCH_STATUS_OK, b"", 1, &[&[1]], 0);
        trailing.push(0);
        assert_eq!(
            decode_response(&trailing, 3, 1),
            Err(WireError::MalformedResponse)
        );
        let hostile = response(
            FERRUM_CHEM_SMARTS_MATCH_STATUS_NATIVE_FAILURE,
            b"/private/FCQ1/FQM1",
            0,
            &[],
            0,
        );
        assert_eq!(
            decode_response(&hostile, 3, 1),
            Err(WireError::MalformedResponse)
        );
    }

    #[test]
    fn closed_native_rejection_never_carries_native_detail() {
        let bytes = response(
            FERRUM_CHEM_SMARTS_MATCH_STATUS_INVALID_QUERY,
            b"invalid_query",
            0,
            &[],
            0,
        );
        assert_eq!(
            decode_response(&bytes, 3, 1),
            Err(WireError::NativeRejected)
        );
        let error = map_wire_error(WireError::NativeRejected);
        assert_eq!(
            error,
            ChemistryError::SmartsMatchUnavailable {
                reason: SmartsMatchUnavailableReason::NativeRejected,
            }
        );
    }

    #[test]
    fn decoder_rejects_rows_over_the_requested_cap() {
        let bytes = response(FERRUM_CHEM_SMARTS_MATCH_STATUS_OK, b"", 1, &[&[0], &[1]], 0);
        assert_eq!(
            decode_response(&bytes, 2, 1),
            Err(WireError::MalformedResponse)
        );
        assert_eq!(
            map_wire_error(WireError::MalformedResponse),
            ChemistryError::SmartsMatchUnavailable {
                reason: SmartsMatchUnavailableReason::MalformedNativeResponse,
            }
        );
    }
}
