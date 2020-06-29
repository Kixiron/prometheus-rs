use crate::{
    atomics::{AtomicF64, AtomicNum},
    error::{PromError, PromErrorKind, Result},
    label::Label,
    registry::{Collectable, Descriptor},
    timer::Timer,
};
use std::{borrow::Cow, cell::RefCell, fmt::Write, iter, sync::atomic::AtomicU64};

/// The default [`Histogram`] buckets. Meant to measure the response time in seconds of network operations
pub const DEFAULT_BUCKETS: &[f64; 12] = &[
    0.005,
    0.01,
    0.025,
    0.05,
    0.1,
    0.25,
    0.5,
    1.0,
    2.5,
    5.0,
    10.0,
    f64::INFINITY,
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistogramBuilder<Atomic: AtomicNum = AtomicF64> {
    name: Option<Cow<'static, str>>,
    help: Option<Cow<'static, str>>,
    labels: Option<Vec<Label>>,
    buckets: Option<Vec<Atomic::Type>>,
}

impl<Atomic: AtomicNum> HistogramBuilder<Atomic> {
    pub fn new() -> Self {
        Self {
            name: None,
            help: None,
            labels: None,
            buckets: None,
        }
    }

    pub fn name(mut self, name: impl Into<Cow<'static, str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn help(mut self, help: impl Into<Cow<'static, str>>) -> Self {
        self.help = Some(help.into());
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
        let help = self.help.ok_or_else(|| {
            PromError::new(
                "Histograms must have a help, but you didn't give one",
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
                descriptor: Descriptor::new(name, help, labels)?,
                core: HistogramCore::new(buckets),
            })
        }
    }
}

#[derive(Debug)]
pub struct HistogramCore<Atomic: AtomicNum> {
    pub(crate) buckets: Vec<Atomic::Type>,
    pub(crate) values: Vec<Atomic>,
    count: AtomicU64,
    sum: Atomic,
}

impl<Atomic: AtomicNum> HistogramCore<Atomic> {
    pub(crate) fn new(buckets: Vec<Atomic::Type>) -> Self {
        Self {
            values: iter::from_fn(|| Some(Atomic::new()))
                .take(buckets.len())
                .collect(),
            buckets,
            count: AtomicU64::new(0),
            sum: Atomic::new(),
        }
    }

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

    pub fn buckets(&self) -> &[Atomic::Type] {
        &self.buckets
    }

    pub fn values(&self) -> Vec<Atomic::Type> {
        self.values.iter().map(|v| v.get()).collect()
    }
}

#[derive(Debug)]
pub struct Histogram<Atomic: AtomicNum = AtomicF64> {
    descriptor: Descriptor,
    core: HistogramCore<Atomic>,
}

impl<Atomic: AtomicNum> Histogram<Atomic> {
    pub fn observe(&self, val: Atomic::Type) {
        self.core.observe(val)
    }

    pub fn clear(&self) {
        self.core.clear()
    }

    pub fn get_count(&self) -> u64 {
        self.core.get_count()
    }

    pub fn get_sum(&self) -> Atomic::Type {
        self.core.get_sum()
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Self> {
        Timer::new(self)
    }

    pub fn local<'a>(&'a self) -> LocalHistogram<'a, Atomic> {
        LocalHistogram::new(self)
    }

    pub fn name(&self) -> &str {
        &self.descriptor.name()
    }

    pub fn help(&self) -> &str {
        &self.descriptor.help()
    }

    pub fn labels(&self) -> &[Label] {
        &self.descriptor.labels()
    }

    pub fn buckets(&self) -> &[Atomic::Type] {
        self.core.buckets()
    }

    pub fn observe_bucket(&self, val: Atomic::Type, bucket: Atomic::Type) -> Result<()> {
        self.core.observe_bucket(val, bucket)
    }
}

impl<Atomic: AtomicNum> Collectable for &Histogram<Atomic> {
    fn encode_text<'a>(&'a self, buf: &mut String) -> Result<()> {
        writeln!(buf, "# HELP {} {}", self.name(), self.help())?;
        writeln!(buf, "# TYPE {} histogram", self.name())?;

        let row = |buf: &mut String, name| -> Result<()> {
            write!(buf, "{}_{}", self.name(), name)?;

            if !self.labels().is_empty() {
                write!(buf, "{{")?;

                let mut labels = self.labels().iter();
                let last = labels.next_back();

                for label in labels {
                    write!(buf, "{}={:?},", label.name(), label.value())?;
                }

                if let Some(last) = last {
                    write!(buf, "{}={:?}", last.name(), last.value())?;
                }

                write!(buf, "}} ")?;
            } else {
                write!(buf, " ")?;
            }

            Ok(())
        };

        row(buf, "sum")?;
        Atomic::format(self.get_sum(), buf, false)?;
        writeln!(buf)?;

        row(buf, "count")?;
        <AtomicU64 as AtomicNum>::format(self.get_count(), buf, false)?;
        writeln!(buf)?;

        for (i, bucket) in self.core.buckets.iter().enumerate() {
            write!(buf, "{}_bucket", self.name())?;

            if !self.labels().is_empty() {
                write!(buf, "{{")?;

                for label in self.labels() {
                    write!(buf, "{}={:?},", label.name(), label.value())?;
                }
                write!(buf, "le=")?;
                Atomic::format(*bucket, buf, true)?;

                write!(buf, "}} ")?;
            } else {
                write!(buf, " ")?;
            }

            Atomic::format(self.core.values[i].get(), buf, false)?;
            writeln!(buf)?;
        }

        Ok(())
    }

    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
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
        if let Some(idx) = self.histogram.core.buckets.iter().position(|b| val <= *b) {
            self.values[idx] += val;
        }

        self.count += 1;
        self.sum += val;
    }

    pub(crate) fn clear(&mut self) {
        for val in self.values.iter_mut() {
            *val = Atomic::Type::default();
        }

        self.count = 0;
        self.sum = Atomic::Type::default();
    }

    pub(crate) fn flush(&mut self) {
        if self.count == 0 {
            return;
        }

        for (i, val) in self.values.iter().enumerate() {
            self.histogram.core.values[i].inc_by(*val);
        }

        self.histogram.core.count.inc_by(self.count);
        self.histogram.core.sum.inc_by(self.sum);
        self.clear();
    }
}

impl<'a, Atomic: AtomicNum> LocalHistogram<'a, Atomic> {
    pub(crate) fn new(histogram: &'a Histogram<Atomic>) -> Self {
        Self {
            inner: RefCell::new(InnerLocalHist {
                histogram,
                values: vec![Atomic::Type::default(); histogram.core.values.len()],
                count: 0,
                sum: Atomic::Type::default(),
            }),
        }
    }

    pub fn flush(&mut self) {
        self.inner.borrow_mut().flush();
    }

    pub fn observe(&mut self, val: Atomic::Type) {
        self.inner.borrow_mut().observe(val);
    }

    pub fn clear(&mut self) {
        self.inner.borrow_mut().clear();
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
            .help("It hist's grams")
            .with_buckets(vec![-1.0, -0.0, 0.0, 1.0])
            .with_labels(vec![Label::new("some_random_label", "whee").unwrap()])
            .label(Label::new("another_label", "I ran out of ideas").unwrap())
            .build()
            .unwrap();

        assert_eq!(built.name(), "some_histogram");
        assert_eq!(built.help(), "It hist's grams");
        assert_eq!(built.buckets(), &[-1.0, -0.0, 0.0, 1.0]);
        assert_eq!(
            built.labels(),
            &[
                Label::new("some_random_label", "whee").unwrap(),
                Label::new("another_label", "I ran out of ideas").unwrap()
            ]
        );
    }
}
