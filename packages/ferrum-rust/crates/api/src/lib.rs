//! Public integration surface and command-line application for Ferrum clients.

mod cdml;
mod cdsvg;
mod cli;
mod errors;
mod reports;
mod streams;

#[cfg(test)]
mod cdsvg_tests;

pub use cdml::{inspect_cdml, rewrite_cdml, validate_cdml, verify_cdml_rewrite};
pub use cdsvg::extract_cdsvg;
pub use cli::Cli;
pub use errors::{CdmlError, CdsvgError, CliError};
pub use reports::{CdmlInspection, CdmlValidation, RewriteCheck};

use std::io::{Read, Write};

/// Execute accepted CLI arguments with caller-owned standard streams.
pub fn run(cli: Cli, stdin: &mut dyn Read, stdout: &mut dyn Write) -> Result<(), CliError> {
    cli::run(cli, stdin, stdout)
}
