use ferrum_core::{Identifier, RecordId, RecordKind};
use ferrum_geometry::Point2;

use crate::repair::{DepictionBond, DepictionVertex};

pub(super) fn id(kind: RecordKind, text: &str) -> RecordId {
    let source = Identifier::new(text).expect("test identifier must be nonblank");
    RecordId::new(kind, source).expect("test record identifier")
}

pub(super) fn point(x: f64, y: f64) -> Point2 {
    Point2::new(x, y).expect("test coordinate must be finite")
}

pub(super) fn vertex(text: &str, x: f64, y: f64) -> DepictionVertex {
    DepictionVertex::new(id(RecordKind::Atom, text), point(x, y))
        .expect("test vertex identity must be atom kind")
}

pub(super) fn bond(text: &str, start: &str, end: &str) -> DepictionBond {
    DepictionBond::new(
        id(RecordKind::Bond, text),
        id(RecordKind::Atom, start),
        id(RecordKind::Atom, end),
    )
    .expect("test bond must be valid")
}
