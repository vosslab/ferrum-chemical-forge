use std::io::Cursor;
use std::path::Path;

use super::read_input_bounded;

#[test]
fn bounded_standard_input_stops_after_the_operation_limit() {
    let mut input = Cursor::new(b"123456".as_slice());
    let error = read_input_bounded(Path::new("-"), &mut input, 5)
        .expect_err("oversized input must be rejected");

    assert!(error.to_string().contains("5-byte operation limit"));
    assert_eq!(input.position(), 6);
}

#[test]
fn bounded_standard_input_retains_valid_utf8_exactly() {
    let mut input = Cursor::new("molecule\n$$$$\n".as_bytes());
    let (source, label) =
        read_input_bounded(Path::new("-"), &mut input, 64).expect("bounded input");

    assert_eq!(source, "molecule\n$$$$\n");
    assert_eq!(label, "standard input");
}
