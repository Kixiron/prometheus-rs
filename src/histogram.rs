use crate::{
    atomics::{AtomicF64, AtomicNum},
    error::{PromError, PromErrorKind, Result},
    label::Label,
    timer::Timer,
};
use std::{borrow::Cow, cell::RefCell, iter, sync::atomic::AtomicU64};

/// The default [`Histogram`] buckets. Meant to measure the response time in seconds of network operations
pub const DEFAULT_BUCKETS: &[f64; 11] = &[
    0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramBuilder<Atomic: AtomicNum = AtomicF64> {
    name: Option<Cow<'static, str>>,
    description: Option<Cow<'static, str>>,
    labels: Option<Vec<Label>>,
    buckets: Option<Vec<Atomic::Type>>,
}

impl<Atomic: AtomicNum> HistogramBuilder<Atomic> {
    pub fn new() -> Self {
        Self {
            name: None,
            description: None,
            labels: None,
            buckets: None,
        }
    }

    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(mut self, description: impl Into<Cow<'static, str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.labels = Some(labels.into());
        self
    }

    pub fn label(mut self, label: Label) -> Self {
        if let Some(ref mut labels) = self.labels {
            labels.push(label);
        } else {
            self.labels = Some(vec![label]);
        }

        self
    }

    pub fn with_buckets(mut self, buckets: impl Into<Vec<Atomic::Type>>) -> Self {
        self.buckets = Some(buckets.into());
        self
    }

    pub fn bucket(mut self, bucket: impl Into<Atomic::Type>) -> Self {
        if let Some(ref mut buckets) = self.buckets {
            buckets.push(bucket.into());
        } else {
            self.buckets = Some(vec![bucket.into()]);
        }

        self
    }

    pub fn build(self) -> Result<Histogram<Atomic>> {
        let name = self.name.ok_or_else(|| {
            PromError::new(
                "Histograms must have a name, but you didn't give one",
                PromErrorKind::MissingComponent,
            )
        })?;
        let description = self.description.ok_or_else(|| {
            PromError::new(
                "Histograms must have a description, but you didn't give one",
                PromErrorKind::MissingComponent,
            )
        })?;
        let buckets = self.buckets.ok_or_else(|| {
            PromError::new(
                "Histograms must have buckets, but you didn't give any",
                PromErrorKind::MissingComponent,
            )
        })?;
        let labels = self.labels.unwrap_or_default();

        if buckets.is_empty() {
            Err(PromError::new(
                "Histograms cannot have empty buckets",
                PromErrorKind::MissingComponent,
            ))
        } else {
            Ok(Histogram {
                name,
                description,
                labels,
                values: iter::from_fn(|| Some(Atomic::new()))
                    .take(buckets.len())
                    .collect(),
                buckets,
                count: AtomicU64::new(0),
                sum: Atomic::new(),
            })
        }
    }
}

#[derive(Debug)]
pub struct Histogram<Atomic: AtomicNum> {
    name: Cow<'static, str>,
    description: Cow<'static, str>,
    labels: Vec<Label>,
    buckets: Vec<Atomic::Type>,
    values: Vec<Atomic>,
    count: AtomicU64,
    sum: Atomic,
}

impl<Atomic: AtomicNum> Histogram<Atomic> {
    pub fn observe(&self, val: Atomic::Type) {
        if let Some(idx) = self.buckets.iter().position(|b| val <= *b) {
            self.values[idx].inc();
        }

        self.count.inc();
        self.sum.inc_by(val);
    }

    pub fn clear(&self) {
        for val in self.values.iter() {
            val.clear();
        }

        self.count.clear();
        self.sum.clear();
    }

    pub fn get_count(&self) -> u64 {
        self.count.get()
    }

    pub fn get_sum(&self) -> Atomic::Type {
        self.sum.get()
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Self> {
        Timer::new(self)
    }

    pub fn local<'a>(&'a self) -> LocalHistogram<'a, Atomic> {
        LocalHistogram::new(self)
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

    pub fn buckets(&self) -> &[Atomic::Type] {
        &self.buckets
    }

    pub fn observe_bucket(&self, val: Atomic::Type, bucket: Atomic::Type) -> Result<()> {
        if let Some(idx) = self.buckets.iter().position(|b| val <= *b) {
            self.values[idx].inc();
            self.count.inc();
            self.sum.inc_by(val);

            Ok(())
        } else {
            Err(PromError::new(
                format!("The bucket {:?} doesn't exist", bucket),
                PromErrorKind::BucketNotFound,
            ))
        }
    }
}

#[derive(Debug)]
pub struct LocalHistogram<'a, Atomic: AtomicNum> {
    pub(crate) inner: RefCell<InnerLocalHist<'a, Atomic>>,
}

#[derive(Debug, Clone)]
pub(crate) struct InnerLocalHist<'a, Atomic: AtomicNum> {
    histogram: &'a Histogram<Atomic>,
    values: Vec<Atomic::Type>,
    count: u64,
    sum: Atomic::Type,
}

impl<'a, Atomic: AtomicNum> InnerLocalHist<'a, Atomic> {
    pub(crate) fn observe(&mut self, val: Atomic::Type) {
        if let Some(idx) = self.histogram.buckets.iter().position(|b| val <= *b) {
            self.values[idx] += val;
        }

        self.count += 1;
        self.sum += val;
    }
}

impl<'a, Atomic: AtomicNum> LocalHistogram<'a, Atomic> {
    pub(crate) fn new(histogram: &'a Histogram<Atomic>) -> Self {
        Self {
            inner: RefCell::new(InnerLocalHist {
                histogram,
                values: vec![Atomic::Type::default(); histogram.values.len()],
                count: 0,
                sum: Atomic::Type::default(),
            }),
        }
    }

    pub fn observe(&mut self, val: Atomic::Type) {
        self.inner.borrow_mut().observe(val);
    }

    pub fn clear(&mut self) {
        let mut inner = self.inner.borrow_mut();
        for val in inner.values.iter_mut() {
            *val = Atomic::Type::default();
        }

        inner.count = 0;
        inner.sum = Atomic::Type::default();
    }

    pub fn get_count(&self) -> u64 {
        self.inner.borrow().count
    }

    pub fn get_sum(&self) -> Atomic::Type {
        self.inner.borrow().sum
    }

    pub fn start_timer<'b>(&'b self) -> Timer<'b, Self> {
        Timer::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build() {
        let built: Histogram<AtomicF64> = HistogramBuilder::new()
            .name("some_histogram")
            .description("It hist's grams")
            .with_buckets(vec![-1.0, -0.0, 0.0, 1.0])
            .with_labels(vec![Label::new("label", "value").unwrap()])
            .label(Label::new("name", "value").unwrap())
            .build()
            .unwrap();

        assert_eq!(built.name(), "some_histogram");
        assert_eq!(built.description(), "It hist's grams");
        assert_eq!(built.buckets(), &[-1.0, -0.0, 0.0, 1.0]);
        assert_eq!(
            built.labels(),
            &[
                Label::new("label", "value").unwrap(),
                Label::new("name", "value").unwrap()
            ]
        );
    }
}
