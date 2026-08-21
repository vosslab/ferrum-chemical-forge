//! Source-neutral authored formatted-text grammar shared by Text creation and editing.

use super::TextPropertiesPatchV1Error;

/// One closed authored formatting style independent of its UI source.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AuthoredTextStyleV1 {
    Bold,
    Italic,
    Subscript,
    Superscript,
}

/// One nonempty authored character-data run with canonical unique styles.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthoredTextRunV1 {
    text: String,
    styles: Vec<AuthoredTextStyleV1>,
}
impl AuthoredTextRunV1 {
    pub fn new(
        text: impl Into<String>,
        mut styles: Vec<AuthoredTextStyleV1>,
    ) -> Result<Self, TextPropertiesPatchV1Error> {
        let text = text.into();
        if text.is_empty() {
            return Err(TextPropertiesPatchV1Error::EmptyRun);
        }
        if text
            .chars()
            .any(|value| value.is_control() && value != '\n')
        {
            return Err(TextPropertiesPatchV1Error::UnsupportedControlCharacter);
        }
        styles.sort_unstable();
        if styles.windows(2).any(|pair| pair[0] == pair[1]) {
            return Err(TextPropertiesPatchV1Error::DuplicateRunStyle);
        }
        if styles.contains(&AuthoredTextStyleV1::Subscript)
            && styles.contains(&AuthoredTextStyleV1::Superscript)
        {
            return Err(TextPropertiesPatchV1Error::ConflictingScriptStyles);
        }
        Ok(Self { text, styles })
    }
    #[must_use]
    pub fn text(&self) -> &str {
        &self.text
    }
    #[must_use]
    pub fn styles(&self) -> &[AuthoredTextStyleV1] {
        &self.styles
    }
    fn append(&mut self, value: &str) {
        self.text.push_str(value);
    }
}

/// Validate the complete authored value and coalesce adjacent equal-style runs.
pub fn normalize_authored_text_runs_v1(
    runs: &mut Vec<AuthoredTextRunV1>,
) -> Result<(), TextPropertiesPatchV1Error> {
    if runs.is_empty()
        || !runs
            .iter()
            .flat_map(|run| run.text.chars())
            .any(|value| !value.is_whitespace())
    {
        return Err(TextPropertiesPatchV1Error::BlankText);
    }
    let mut result: Vec<AuthoredTextRunV1> = Vec::with_capacity(runs.len());
    for run in runs.drain(..) {
        if let Some(previous) = result
            .last_mut()
            .filter(|previous| previous.styles == run.styles)
        {
            previous.append(&run.text);
        } else {
            result.push(run);
        }
    }
    *runs = result;
    Ok(())
}
