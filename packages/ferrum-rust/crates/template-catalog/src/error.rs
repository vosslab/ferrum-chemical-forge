use std::io;
use thiserror::Error;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateCatalogRefusalCategoryV1 {
    DirectorySymlink,
    DirectoryNotDirectory,
    FilenameNonUtf8,
    CandidateSymlink,
    CandidateNotRegular,
    CandidateOpenFailed,
    CandidateReadFailed,
    FileTooLarge,
    CatalogLimitExceeded,
    Utf8Invalid,
    DocumentAdmission,
    DuplicateContent,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemplateCatalogRecoveryV1 {
    Refresh,
    FixDirectory,
    FixFile,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateCatalogRefusalV1 {
    category: TemplateCatalogRefusalCategoryV1,
    basename: Option<String>,
    recovery: TemplateCatalogRecoveryV1,
    occurrences: u64,
}
impl TemplateCatalogRefusalV1 {
    pub(crate) fn new(
        category: TemplateCatalogRefusalCategoryV1,
        basename: Option<String>,
    ) -> Self {
        let recovery = match category {
            TemplateCatalogRefusalCategoryV1::DuplicateContent => {
                TemplateCatalogRecoveryV1::Refresh
            }
            TemplateCatalogRefusalCategoryV1::DirectorySymlink
            | TemplateCatalogRefusalCategoryV1::DirectoryNotDirectory => {
                TemplateCatalogRecoveryV1::FixDirectory
            }
            _ => TemplateCatalogRecoveryV1::FixFile,
        };
        Self {
            category,
            basename,
            recovery,
            occurrences: 1,
        }
    }
    pub(crate) fn aggregate_limit_exceeded(occurrences: u64) -> Self {
        Self {
            category: TemplateCatalogRefusalCategoryV1::CatalogLimitExceeded,
            basename: None,
            recovery: TemplateCatalogRecoveryV1::FixFile,
            occurrences,
        }
    }
    #[must_use]
    pub const fn category(&self) -> TemplateCatalogRefusalCategoryV1 {
        self.category
    }
    #[must_use]
    pub fn basename(&self) -> Option<&str> {
        self.basename.as_deref()
    }
    #[must_use]
    pub const fn recovery(&self) -> TemplateCatalogRecoveryV1 {
        self.recovery
    }
    #[must_use]
    pub const fn occurrences(&self) -> u64 {
        self.occurrences
    }
}
#[derive(Debug, Error)]
pub enum TemplateCatalogErrorV1 {
    #[error("template catalog directory could not be opened")]
    DirectoryOpen(#[source] io::Error),
    #[error("template catalog allocation failed")]
    Allocation,
}
