//! Product-profile checks for decoded local CD-SVG admission.

use std::path::{Path, PathBuf};

use ferrum_document::{DocumentIngressErrorV1, prepare_local_decoded_cdsvg_file_v1};

const CANONICAL_CDSVG: &str = concat!(
    "<svg xmlns=\"http://www.w3.org/2000/svg\">",
    "<cdml xmlns=\"http://www.freesoftware.fsf.org/bkchem/cdml\" version=\"1.0\">",
    "<plus id=\"payload-fact\"><point x=\"1\" y=\"2\"/></plus>",
    "</cdml><metadata>discarded wrapper content</metadata></svg>",
);

struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ferrum-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&path).expect("test directory must be creatable");
        Self { path }
    }

    fn write(&self, name: &str, source: &str) -> PathBuf {
        let path = self.path.join(name);
        std::fs::write(&path, source).expect("test SVG must be writable");
        path
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ignored = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn selected_profile_admits_only_the_canonical_cdml_payload() {
    let temporary = TemporaryDirectory::new("local-decoded-cdsvg-profile");
    let source = temporary.write("drawing.svg", CANONICAL_CDSVG);

    let (session, _origin) =
        prepare_local_decoded_cdsvg_file_v1(Path::new(&source)).expect("canonical payload admits");
    let snapshot = session.snapshot().expect("new admission has a snapshot");

    assert!(
        snapshot.cdml().contains("payload-fact") && !snapshot.cdml().contains("<svg"),
        "the durable session retains the payload fact but no wrapper",
    );
}

#[test]
fn selected_profile_refuses_invalid_svg_containers_before_session_creation() {
    let temporary = TemporaryDirectory::new("local-decoded-cdsvg-refusal");
    let cases = [
        ("missing.svg", "<svg xmlns=\"http://www.w3.org/2000/svg\"/>"),
        (
            "multiple.svg",
            concat!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\">",
                "<cdml xmlns=\"http://www.freesoftware.fsf.org/bkchem/cdml\" version=\"1.0\"/>",
                "<cdml xmlns=\"http://www.freesoftware.fsf.org/bkchem/cdml\" version=\"1.0\"/>",
                "</svg>",
            ),
        ),
        (
            "wrong-namespace.svg",
            "<svg xmlns=\"http://www.w3.org/2000/svg\"><cdml version=\"1.0\"/></svg>",
        ),
    ];

    for (name, source) in cases {
        let path = temporary.write(name, source);
        assert!(
            matches!(
                prepare_local_decoded_cdsvg_file_v1(&path),
                Err(DocumentIngressErrorV1::Cdsvg { .. })
            ),
            "{name} must be refused by the CD-SVG admission boundary",
        );
    }
}
