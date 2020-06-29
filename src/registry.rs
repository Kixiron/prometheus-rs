use crate::{
    error::{PromError, PromErrorKind, Result},
    label::{valid_metric_name, Label},
};
use std::{borrow::Cow, fmt};

pub struct RegistryBuilder {
    inputs: Option<Vec<Box<dyn Collectable + Send + Sync>>>,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        Self { inputs: None }
    }

    pub fn register_all(
        mut self,
        inputs: impl Into<Vec<Box<dyn Collectable + Send + Sync>>>,
    ) -> Self {
        self.inputs = Some(inputs.into());
        self
    }

    pub fn register(mut self, input: Box<dyn Collectable + Send + Sync>) -> Self {
        if let Some(ref mut inputs) = self.inputs {
            inputs.push(input);
        } else {
            self.inputs = Some(vec![input]);
        }

        self
    }

    pub fn build(self) -> Result<Registry> {
        let raw_inputs = self.inputs.ok_or_else(|| {
            PromError::new(
                "Registries must have at least one collection source",
                PromErrorKind::MissingComponent,
            )
        })?;

        if raw_inputs.is_empty() {
            return Err(PromError::new(
                "Registries must have at least one collection source",
                PromErrorKind::MissingComponent,
            ));
        }

        let mut inputs: Vec<Box<dyn Collectable + Send + Sync>> =
            Vec::with_capacity(raw_inputs.len());

        for input in raw_inputs {
            if inputs.iter().any(|coll| {
                coll.descriptor().name() == input.descriptor().name()
                    && coll.descriptor().labels() == input.descriptor().labels()
            }) {
                return Err(PromError::new(
                    format!("{} was registered twice", input.descriptor().name()),
                    PromErrorKind::DuplicatedCollector,
                ));
            } else {
                inputs.push(input);
            }
        }

        inputs.sort_unstable_by(|a, b| a.descriptor().name().cmp(b.descriptor().name()));

        Ok(Registry { inputs })
    }
}

impl fmt::Debug for RegistryBuilder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RegistryBuilder")
            .field(
                "inputs",
                &self.inputs.as_ref().map(|inputs| {
                    inputs
                        .iter()
                        .map(|coll| (coll.descriptor().name(), coll.descriptor().help()))
                        .collect::<Vec<_>>()
                }),
            )
            .finish()
    }
}

pub struct Registry {
    inputs: Vec<Box<dyn Collectable + Send + Sync>>,
}

impl Registry {
    pub fn collect<'a>(&'a self) -> Vec<Metric<'a>> {
        let mut metrics = Vec::with_capacity(self.inputs.len());
        for input in self.inputs.iter() {
            metrics.push(Metric::new(&**input, input.descriptor()));
        }

        metrics
    }

    pub fn collect_to_string(&self) -> Result<String> {
        let mut buf = String::new();
        for input in self.inputs.iter() {
            input.encode_text(&mut buf)?;
        }

        Ok(buf)
    }

    /// Initializes all registered collectors, useful for when the `Registry` is stored in a `once_cell::Lazy` or `lazy_static`
    pub fn init_registered(&self) {
        self.collect();
    }
}

impl fmt::Debug for Registry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Registry")
            .field(
                "inputs",
                &self
                    .inputs
                    .iter()
                    .map(|coll| (coll.descriptor().name(), coll.descriptor().help()))
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
}

#[derive(Clone)]
pub struct Metric<'a> {
    name: &'a str,
    help: &'a str,
    labels: &'a [Label],
    value: &'a dyn Collectable,
}

impl<'a> Metric<'a> {
    fn new(value: &'a dyn Collectable, descriptor: &'a Descriptor) -> Self {
        Self {
            name: descriptor.name(),
            help: descriptor.help(),
            labels: descriptor.labels(),
            value,
        }
    }

    pub fn encode_text(&self, buf: &mut String) -> Result<()> {
        self.value.encode_text(buf)
    }
}

impl fmt::Debug for Metric<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Metric")
            .field("name", &self.name)
            .field("help", &self.help)
            .field("labels", &self.labels)
            .finish()
    }
}

pub trait Collectable {
    fn encode_text<'a>(&'a self, buf: &mut String) -> Result<()>;
    fn descriptor(&self) -> &Descriptor;
}

impl<T> Collectable for T
where
    T: AsRef<dyn Collectable>,
{
    fn encode_text<'a>(&'a self, buf: &mut String) -> Result<()> {
        self.as_ref().encode_text(buf)
    }

    fn descriptor(&self) -> &Descriptor {
        self.as_ref().descriptor()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Descriptor {
    name: Cow<'static, str>,
    help: Cow<'static, str>,
    pub(crate) labels: Vec<Label>,
}

impl Descriptor {
    pub(crate) fn new(
        name: impl Into<Cow<'static, str>>,
        help: impl AsRef<str>,
        labels: impl Into<Vec<Label>>,
    ) -> Result<Self> {
        let name = name.into();

        if !valid_metric_name(&name) {
            return Err(PromError::new(
                "Metric name contains invalid characters",
                PromErrorKind::InvalidMetricName,
            ));
        }

        Ok(Self {
            name,
            help: help
                .as_ref()
                .replace("\\", "\\\\")
                .replace("\n", "\\n")
                .into(),
            labels: labels.into(),
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn help(&self) -> &str {
        &self.help
    }

    pub fn labels(&self) -> &[Label] {
        &self.labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        counter::Counter,
        gauge::Gauge,
        histogram::{Histogram, HistogramBuilder, DEFAULT_BUCKETS},
    };
    use once_cell::sync::Lazy;

    #[test]
    fn normal_use() {
        static COUNTER: Lazy<Counter> =
            Lazy::new(|| Counter::new("my_counter", "Counts things because I can't").unwrap());
        static GAUGE: Lazy<Gauge> = Lazy::new(|| Gauge::new("my_gauge", "Gagin' stuff").unwrap());
        static HISTOGRAM: Lazy<Histogram> = Lazy::new(|| {
            HistogramBuilder::new()
                .name("some_histogram")
                .help("It hist's grams")
                .with_buckets(DEFAULT_BUCKETS.to_vec())
                .with_labels(vec![Label::new("label", "value").unwrap()])
                .label(Label::new("name", "value").unwrap())
                .build()
                .unwrap()
        });

        static REGISTRY: Lazy<Registry> = Lazy::new(|| {
            RegistryBuilder::new()
                .register(Box::new(&*COUNTER))
                .register(Box::new(&*GAUGE))
                .register(Box::new(&*HISTOGRAM))
                .build()
                .unwrap()
        });

        GAUGE.set(10000);
        COUNTER.set(100);

        println!("{}", REGISTRY.collect_to_string().unwrap());
    }
}
