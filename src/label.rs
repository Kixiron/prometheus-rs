use crate::error::{PromError, PromErrorKind, Result};
use std::{borrow::Cow, convert::TryFrom};

/// Label names follow the regex `[a-zA-Z_][a-zA-Z0-9_]*` with the exception that labels starting with `__` are reserved,
/// as well as the label name `le`
// TODO: Make this const when rust/#68983 and rust/#49146 land
fn valid_label_name(label: &str) -> bool {
    let mut chars = label.chars();

    !label.is_empty()
        && label != "le"
        && matches!(chars.next(), Some(next) if next.is_ascii_alphabetic() || next == '_')
        && match chars.next() {
            Some(next) if next.is_ascii_alphabetic() || next != '_' => true,
            None => true,
            _ => false,
        }
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Metric names follow the regex `[a-zA-Z_:][a-zA-Z0-9_:]*`
// TODO: Make this const when rust/#68983 and rust/#49146 land
pub(crate) fn valid_metric_name(metric: &str) -> bool {
    let mut chars = metric.chars();

    !metric.is_empty()
        && matches!(chars.next(), Some(next) if next.is_ascii_alphabetic() || next == '_' || next == ':')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub(crate) name: Cow<'static, str>,
    pub(crate) value: Cow<'static, str>,
}

impl Label {
    /// Create a new label with the given name and value.
    ///
    /// Returns `Err` if `label` doesn't follow the regex `[a-zA-Z_][a-zA-Z0-9_]*`
    pub fn new(
        name: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Result<Self> {
        let name = name.into();

        if valid_label_name(&name) {
            Ok(Self {
                name,
                value: value.into(),
            })
        } else {
            Err(PromError::new(
                "Label name contains invalid characters",
                PromErrorKind::InvalidLabelName,
            ))
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl<L, V> TryFrom<(L, V)> for Label
where
    L: Into<Cow<'static, str>>,
    V: Into<Cow<'static, str>>,
{
    type Error = PromError;

    fn try_from((label, value): (L, V)) -> Result<Self> {
        Self::new(label, value)
    }
}
