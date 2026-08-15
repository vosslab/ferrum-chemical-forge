use std::path::Path;

pub(crate) fn is_standard_stream(path: &Path) -> bool {
    path == Path::new("-")
}
