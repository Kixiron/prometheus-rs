use crate::error::{PromError, PromErrorKind, Result};
use std::{borrow::Cow, convert::TryFrom, ops};

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
fn valid_metric_name(metric: &str) -> bool {
    let mut chars = metric.chars();

    !metric.is_empty()
        && matches!(chars.next(), Some(next) if next.is_ascii_alphabetic() || next == '_' || next == ':')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':')
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Label {
    pub(crate) label: Cow<'static, str>,
    pub(crate) value: Cow<'static, str>,
}

impl Label {
    /// Create a new label with the given name and value.
    ///
    /// Returns `Err` if `label` doesn't follow the regex `[a-zA-Z_][a-zA-Z0-9_]*`
    pub fn new(
        label: impl Into<Cow<'static, str>>,
        value: impl Into<Cow<'static, str>>,
    ) -> Result<Self> {
        let label = label.into();

        if valid_label_name(&label) {
            Ok(Self {
                label,
                value: value.into(),
            })
        } else {
            Err(PromError::new(
                "Label name contains invalid characters",
                PromErrorKind::InvalidLabelName,
            ))
        }
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

#[derive(Debug)]
pub(crate) struct Labeled<T> {
    data: T,
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    pub(crate) labels: Vec<Label>,
}

impl<T> Labeled<T> {
    pub fn new(
        data: T,
        name: impl Into<Cow<'static, str>>,
        description: impl Into<Cow<'static, str>>,
    ) -> Result<Self> {
        let (name, description) = (name.into(), description.into());

        if valid_metric_name(&name) {
            Ok(Self {
                data,
                name,
                description,
                labels: Vec::new(),
            })
        } else {
            Err(PromError::new(
                "Metric name contains invalid characters",
                PromErrorKind::InvalidMetricName,
            ))
        }
    }

    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }
}

impl<T> ops::Deref for Labeled<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<T> ops::DerefMut for Labeled<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
