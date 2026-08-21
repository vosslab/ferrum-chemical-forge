//! Closed-profile CML decoding entry point.

#[path = "cml_decoder/parser.rs"]
mod parser;
#[path = "cml_decoder/values.rs"]
mod values;

use xmlparser::Tokenizer;

use super::*;

const MAX_INPUT_BYTES: usize = 1_048_576;
const MAX_IDENTIFIER_BYTES: usize = 128;
const MAX_COORDINATE: f64 = 100_000.0;

struct Pending {
    name: String,
    attributes: Vec<(String, String)>,
}

/// Decode one UTF-8 CML1 or CML2 document into bounded owned source graphs.
///
/// The input is bytes so non-UTF-8 input can receive the closed `invalid_utf8`
/// refusal before XML tokenization or retained graph allocation.
pub fn decode_cml_bytes_v1(input: &[u8]) -> Result<CmlDecodedDocumentV1> {
    let input = input.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(input);
    if input.len() > MAX_INPUT_BYTES {
        return refused(CmlRefusalReasonV1::InputBytesLimit);
    }
    let source = std::str::from_utf8(input).map_err(|_| CmlDecoderErrorV1 {
        reason: CmlRefusalReasonV1::InvalidUtf8,
    })?;
    let mut parser = parser::Parser::new();
    for token in Tokenizer::from(source) {
        parser.token(token.map_err(|_| CmlDecoderErrorV1 {
            reason: CmlRefusalReasonV1::InvalidXml,
        })?)?;
    }
    parser.finish()
}
