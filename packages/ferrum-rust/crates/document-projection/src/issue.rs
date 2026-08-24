//! General immutable projection diagnostics.

use serde::Serialize;
use thiserror::Error;

/// Stable categories for facts not represented by the V1 document projection.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionIssueCodeV1 {
    MissingBondEndpoint,
    UnsupportedBondEndpoint,
    UnknownBondEndpoint,
    UnsupportedBondType,
    InvalidPresentationFact,
}

/// Failure while constructing a general projection diagnostic.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ProjectionIssueV1Error {
    #[error("projection issue path must not be empty")]
    EmptyPath,
    #[error("projection issue detail must not be empty")]
    EmptyDetail,
}

/// One recognized but non-renderable typed fact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectionIssueV1 {
    code: ProjectionIssueCodeV1,
    path: String,
    detail: String,
}

impl ProjectionIssueV1 {
    /// Construct one actionable source diagnostic from immutable primitive values.
    pub fn try_new(
        code: ProjectionIssueCodeV1,
        path: String,
        detail: String,
    ) -> Result<Self, ProjectionIssueV1Error> {
        if path.is_empty() {
            return Err(ProjectionIssueV1Error::EmptyPath);
        }
        if detail.is_empty() {
            return Err(ProjectionIssueV1Error::EmptyDetail);
        }
        Ok(Self { code, path, detail })
    }

    #[must_use]
    pub const fn code(&self) -> ProjectionIssueCodeV1 {
        self.code
    }

    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    #[must_use]
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

#[cfg(test)]
mod tests {
    use super::{ProjectionIssueCodeV1, ProjectionIssueV1, ProjectionIssueV1Error};

    #[test]
    fn issue_keeps_its_closed_category_and_actionable_source_text() {
        let issue = ProjectionIssueV1::try_new(
            ProjectionIssueCodeV1::InvalidPresentationFact,
            "/cdml/molecule[1]".to_owned(),
            "line_width must be positive and finite".to_owned(),
        )
        .expect("nonempty issue text is valid");

        assert_eq!(issue.code(), ProjectionIssueCodeV1::InvalidPresentationFact);
        assert_eq!(issue.path(), "/cdml/molecule[1]");
        assert_eq!(issue.detail(), "line_width must be positive and finite");
    }

    #[test]
    fn issue_refuses_empty_wire_text() {
        assert_eq!(
            ProjectionIssueV1::try_new(
                ProjectionIssueCodeV1::MissingBondEndpoint,
                String::new(),
                "start is absent".to_owned(),
            ),
            Err(ProjectionIssueV1Error::EmptyPath)
        );
        assert_eq!(
            ProjectionIssueV1::try_new(
                ProjectionIssueCodeV1::MissingBondEndpoint,
                "/cdml/molecule[1]/bond[1]".to_owned(),
                String::new(),
            ),
            Err(ProjectionIssueV1Error::EmptyDetail)
        );
    }
}
