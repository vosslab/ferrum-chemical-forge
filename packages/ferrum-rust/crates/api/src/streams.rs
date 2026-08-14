use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

use ferrum_document::DocumentSession;

use crate::errors::CliError;

pub(crate) fn read_input(path: &Path, stdin: &mut dyn Read) -> Result<(String, String), CliError> {
    let label = stream_label(path, "standard input");
    if is_standard_stream(path) {
        let mut source = String::new();
        stdin
            .read_to_string(&mut source)
            .map_err(|source| CliError::Read {
                input: label.clone(),
                source,
            })?;
        Ok((source, label))
    } else {
        fs::read_to_string(path)
            .map(|source| (source, label.clone()))
            .map_err(|source| CliError::Read {
                input: label,
                source,
            })
    }
}

pub(crate) fn read_input_bounded(
    path: &Path,
    stdin: &mut dyn Read,
    maximum_bytes: usize,
) -> Result<(String, String), CliError> {
    let label = stream_label(path, "standard input");
    let mut file;
    let reader: &mut dyn Read = if is_standard_stream(path) {
        stdin
    } else {
        file = File::open(path).map_err(|source| CliError::Read {
            input: label.clone(),
            source,
        })?;
        &mut file
    };
    let take_limit = u64::try_from(maximum_bytes)
        .expect("usize fits u64 on Ferrum targets")
        .saturating_add(1);
    let mut source = String::new();
    reader
        .take(take_limit)
        .read_to_string(&mut source)
        .map_err(|source| CliError::Read {
            input: label.clone(),
            source,
        })?;
    if source.len() > maximum_bytes {
        return Err(CliError::Read {
            input: label,
            source: std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("input exceeds the {maximum_bytes}-byte operation limit"),
            ),
        });
    }
    Ok((source, label))
}

pub(crate) fn write_report(contents: &[u8], stdout: &mut dyn Write) -> Result<(), CliError> {
    stdout
        .write_all(contents)
        .map_err(|source| CliError::Write {
            output: "standard output".to_owned(),
            source,
        })
}

pub(crate) fn write_rewrite(
    path: &Path,
    cdml: &str,
    stdout: &mut dyn Write,
) -> Result<(), CliError> {
    if is_standard_stream(path) {
        return write_report(cdml.as_bytes(), stdout);
    }
    let label = stream_label(path, "standard output");
    let mut session = DocumentSession::load(cdml).map_err(|source| CliError::Publish {
        output: label.clone(),
        source,
    })?;
    let revision = session
        .snapshot()
        .map_err(|source| CliError::Publish {
            output: label.clone(),
            source,
        })?
        .revision();
    session
        .save_atomic(path, revision)
        .map_err(|source| CliError::Publish {
            output: label,
            source,
        })?;
    Ok(())
}

pub(crate) fn is_standard_stream(path: &Path) -> bool {
    path == Path::new("-")
}

fn stream_label(path: &Path, standard_stream: &str) -> String {
    if is_standard_stream(path) {
        standard_stream.to_owned()
    } else {
        path.display().to_string()
    }
}

#[cfg(test)]
#[path = "streams_tests.rs"]
mod tests;
