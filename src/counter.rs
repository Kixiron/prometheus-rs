use crate::{
    atomics::{AtomicF64, AtomicNum},
    error::{PromError, PromErrorKind},
    label::{Label, Labeled, Metric},
};
use std::sync::atomic::{AtomicI64, AtomicU64};

/// An unlabeled counter
#[derive(Debug)]
#[repr(transparent)]
pub struct Counter<Atomic: AtomicNum = AtomicU64> {
    /// The inner atomically manipulated value
    value: Atomic,
}

impl Counter<AtomicI64> {
    /// Initializing `i64` `Counter`s
    pub const fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }
}

impl Counter<AtomicU64> {
    /// Initializing `u64` `Counter`s
    pub const fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }
}

impl Counter<AtomicF64> {
    /// Initializing `f64` `Counter`s
    pub const fn new() -> Self {
        Self {
            value: AtomicF64::zeroed(),
        }
    }
}

impl<Atomic: AtomicNum> Counter<Atomic> {
    /// Increment the value by 1
    pub fn inc(&self) {
        self.value.inc();
    }

    /// Increment the value by `inc`
    pub fn inc_by(&self, inc: Atomic::Type) {
        self.value.inc_by(inc);
    }

    /// Get the value
    pub fn get(&self) -> Atomic::Type {
        self.value.get()
    }

    /// Reset the value to 0
    pub fn clear(&self) {
        self.value.clear()
    }

    /// Set the value to `val`
    pub fn set(&self, val: Atomic::Type) {
        self.value.set(val);
    }
}

impl<Atomic: AtomicNum> Metric for Counter<Atomic> {
    fn metric_kind(&self) -> &'static str {
        "counter"
    }
}

/// A monotonically increasing counter with the inner type of `u64`
///
/// To quote the [docs]:
///
/// > A counter is a cumulative metric that represents a single monotonically increasing counter whose value can only increase or be reset to zero on restart.
/// > For example, you can use a counter to represent the number of requests served, tasks completed, or errors.
/// >
/// > Do not use a counter to expose a value that can decrease. For example, do not use a counter for the number of currently running processes; instead use a gauge.
///
/// [docs]: https://prometheus.io/docs/concepts/metric_types/#counter
#[derive(Debug)]
#[repr(transparent)]
pub struct UintCounter(pub(crate) Labeled<Counter<AtomicU64>>);

impl UintCounter {
    /// Create a new `UintCounter`, using `name` as the label given to prometheus
    ///
    /// Returns `Err` if `name` doesn't follow `[a-zA-Z_:][a-zA-Z0-9_:]*`
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn new(name: &'static str, description: &'static str) -> Result<Self, PromError> {
        Ok(Self(Labeled::new(
            <Counter<AtomicU64>>::new(),
            name,
            description,
        )?))
    }

    /// Increment the current counter by 1
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc();
    /// assert_eq!(counter.get(), 1);
    /// ```
    pub fn inc(&self) {
        self.0.inc();
    }

    /// Increment the current counter by `inc`
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc_by(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn inc_by(&self, inc: u64) {
        self.0.inc_by(inc);
    }

    /// Get the value of the current counter
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn get(&self) -> u64 {
        self.0.get()
    }

    /// Reset the current counter's value to 0
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// counter.clear();
    /// assert_eq!(counter.get(), 0);
    /// ```
    pub fn clear(&self) {
        self.0.clear()
    }

    /// Set the current counter's value to `val`
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn set(&self, val: u64) {
        self.0.set(val)
    }

    /// Get the current counter's name
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn name(&self) -> &str {
        &self.0.name()
    }

    /// Get the current counter's description
    ///
    /// ```rust
    /// use prometheus_rs::UintCounter;
    ///
    /// let counter = UintCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.description(), "I am Count von Count!");
    /// ```
    pub fn description(&self) -> &str {
        &self.0.description()
    }

    /// Set the labels of the current counter
    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.0.labels = labels.into();
        self
    }
}

/// A monotonically increasing counter with the inner type of `f64`
///
/// To quote the [docs]:
///
/// > A counter is a cumulative metric that represents a single monotonically increasing counter whose value can only increase or be reset to zero on restart.
/// > For example, you can use a counter to represent the number of requests served, tasks completed, or errors.
/// >
/// > Do not use a counter to expose a value that can decrease. For example, do not use a counter for the number of currently running processes; instead use a gauge.
///
/// [docs]: https://prometheus.io/docs/concepts/metric_types/#counter
#[derive(Debug)]
#[repr(transparent)]
pub struct FloatCounter(pub(crate) Labeled<Counter<AtomicF64>>);

impl FloatCounter {
    /// Create a new `FloatCounter`, using `name` as the label given to prometheus
    ///
    /// Returns `Err` if `name` doesn't follow `[a-zA-Z_:][a-zA-Z0-9_:]*`
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn new(name: &'static str, description: &'static str) -> Result<Self, PromError> {
        Ok(Self(Labeled::new(
            <Counter<AtomicF64>>::new(),
            name,
            description,
        )?))
    }

    /// Increment the current counter by 1
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc();
    /// assert_eq!(counter.get(), 1.0);
    /// ```
    pub fn inc(&self) {
        self.0.inc();
    }

    /// Increment the current counter by `inc`
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc_by(100.0);
    /// assert_eq!(counter.get(), 100.0);
    /// ```
    ///
    /// ## Panics
    ///
    /// Panics if `inc` is negative as per [spec]
    ///
    /// ```rust,should_panic
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc_by(-100.0); // Panic!
    /// ```
    ///
    /// [spec]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    pub fn inc_by(&self, inc: f64) {
        assert!(
            inc.is_sign_positive(),
            "Can only increment `FloatCounter` by positive numbers"
        );

        self.0.inc_by(inc);
    }

    /// Increment the current counter by `inc`, returning `Err` if `inc` is negative as per [spec]
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    ///
    /// assert!(counter.try_inc_by(100.0).is_ok());
    /// assert_eq!(counter.get(), 100.0);
    ///
    /// assert!(counter.try_inc_by(-100.0).is_err());
    /// ```
    ///
    /// [spec]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    pub fn try_inc_by(&self, inc: f64) -> Result<(), PromError> {
        if inc.is_sign_positive() {
            self.0.inc_by(inc);

            Ok(())
        } else {
            Err(PromError::new(
                "Can only increment `FloatCounter` by positive numbers",
                PromErrorKind::IncrementNegative,
            ))
        }
    }

    /// Get the value of the current counter
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100.0);
    /// assert_eq!(counter.get(), 100.0);
    /// ```
    pub fn get(&self) -> f64 {
        self.0.get()
    }

    /// Reset the current counter's value to 0
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100.0);
    /// assert_eq!(counter.get(), 100.0);
    /// counter.clear();
    /// assert_eq!(counter.get(), 0.0);
    /// ```
    pub fn clear(&self) {
        self.0.clear()
    }

    /// Set the current counter's value to `val`
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100.0);
    /// assert_eq!(counter.get(), 100.0);
    /// ```
    pub fn set(&self, val: f64) {
        self.0.set(val)
    }

    /// Get the current counter's label name
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn name(&self) -> &str {
        &self.0.name()
    }

    /// Get the current counter's description
    ///
    /// ```rust
    /// use prometheus_rs::FloatCounter;
    ///
    /// let counter = FloatCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.description(), "I am Count von Count!");
    /// ```
    pub fn description(&self) -> &str {
        &self.0.description()
    }

    /// Set the labels of the current counter
    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.0.labels = labels.into();
        self
    }
}

/// A monotonically increasing counter with the inner type of `i64`. Using [`UintCounter`] instead of this is recommended.
///
/// To quote the [docs]:
///
/// > A counter is a cumulative metric that represents a single monotonically increasing counter whose value can only increase or be reset to zero on restart.
/// > For example, you can use a counter to represent the number of requests served, tasks completed, or errors.
/// >
/// > Do not use a counter to expose a value that can decrease. For example, do not use a counter for the number of currently running processes; instead use a gauge.
///
/// [docs]: https://prometheus.io/docs/concepts/metric_types/#counter
///
/// [`UintCounter`]: crate::UintCounter
#[derive(Debug)]
#[repr(transparent)]
pub struct IntCounter(pub(crate) Labeled<Counter<AtomicI64>>);

impl IntCounter {
    /// Create a new `IntCounter`, using `name` as the label given to prometheus
    ///
    /// Returns `Err` if `name` doesn't follow `[a-zA-Z_:][a-zA-Z0-9_:]*`
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn new(name: &'static str, description: &'static str) -> Result<Self, PromError> {
        Ok(Self(Labeled::new(
            <Counter<AtomicI64>>::new(),
            name,
            description,
        )?))
    }

    /// Increment the current counter by 1
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc();
    /// assert_eq!(counter.get(), 1);
    /// ```
    pub fn inc(&self) {
        self.0.inc();
    }

    /// Increment the current counter by `inc`
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc_by(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    ///
    /// ## Panics
    ///
    /// Panics if `inc` is negative as per [spec]
    /// ```rust,should_panic
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc_by(-100); // Panic!
    /// ```
    ///
    /// [spec]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    pub fn inc_by(&self, inc: i64) {
        assert!(
            !inc.is_negative(),
            "Can only increment `IntCounter` by positive numbers"
        );

        self.0.inc_by(inc);
    }

    /// Increment the current counter by `inc`, returning `Err` if `inc` is negative as per [spec]
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    ///
    /// assert!(counter.try_inc_by(100).is_ok());
    /// assert_eq!(counter.get(), 100);
    ///
    /// assert!(counter.try_inc_by(-100).is_err());
    /// ```
    ///
    /// [spec]: https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    pub fn try_inc_by(&self, inc: i64) -> Result<(), PromError> {
        // `.is_positive` has subtle semantics, it returns false if the number is 0 or negative,
        // while 0 is an acceptable value here
        if !inc.is_negative() {
            self.0.inc_by(inc);

            Ok(())
        } else {
            Err(PromError::new(
                "Can only increment `IntCounter` by positive numbers",
                PromErrorKind::IncrementNegative,
            ))
        }
    }

    /// Get the value of the current counter
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn get(&self) -> i64 {
        self.0.get()
    }

    /// Reset the current counter's value to 0
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// counter.clear();
    /// assert_eq!(counter.get(), 0);
    /// ```
    pub fn clear(&self) {
        self.0.clear()
    }

    /// Set the current counter's value to `val`
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn set(&self, val: i64) {
        self.0.set(val)
    }

    /// Get the current counter's label name
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn name(&self) -> &str {
        &self.0.name()
    }

    /// Get the current counter's description
    ///
    /// ```rust
    /// use prometheus_rs::IntCounter;
    ///
    /// let counter = IntCounter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.description(), "I am Count von Count!");
    /// ```
    pub fn description(&self) -> &str {
        &self.0.description()
    }

    /// Set the labels of the current counter
    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.0.labels = labels.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::thread;

    #[test]
    fn uint_counter() {
        let uint = UintCounter::new("some_uint", "Counts things").unwrap();

        assert_eq!(uint.name(), "some_uint");

        uint.inc();
        assert_eq!(uint.get(), 1);

        uint.inc();
        assert_eq!(uint.get(), 2);

        uint.clear();
        assert_eq!(uint.get(), 0);

        uint.inc_by(10);
        assert_eq!(uint.get(), 10);

        uint.inc_by(0);
        assert_eq!(uint.get(), 10);

        uint.set(999);
        assert_eq!(uint.get(), 999);
    }

    #[test]
    #[cfg(not(miri))]
    fn uint_threaded() {
        static UINT: Lazy<UintCounter> = Lazy::new(|| {
            UintCounter::new("surfin_the_world_wide_thread", "Counts things").unwrap()
        });

        let mut threads = Vec::with_capacity(5);
        for _ in 0..5 {
            threads.push(thread::spawn(|| {
                UINT.inc();
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(UINT.get(), 5);
    }

    #[test]
    fn float_counter() {
        let float = FloatCounter::new("some_float", "Counts things").unwrap();

        assert_eq!(float.name(), "some_float");

        float.inc();
        assert_eq!(float.get(), 1.0);

        float.inc();
        assert_eq!(float.get(), 2.0);

        float.clear();
        assert_eq!(float.get(), 0.0);

        float.inc_by(10.0);
        assert_eq!(float.get(), 10.0);

        assert!(float.try_inc_by(10.0).is_ok());
        assert_eq!(float.get(), 20.0);

        float.inc_by(0.0);
        assert_eq!(float.get(), 20.0);

        float.set(999.999);
        assert_eq!(float.get(), 999.999);
    }

    #[test]
    #[cfg(not(miri))]
    fn float_threaded() {
        static FLOAT: Lazy<FloatCounter> = Lazy::new(|| {
            FloatCounter::new("surfin_the_world_wide_thread", "Counts things").unwrap()
        });

        let mut threads = Vec::with_capacity(5);
        for _ in 0..5 {
            threads.push(thread::spawn(|| {
                FLOAT.inc();
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(FLOAT.get(), 5.0);
    }

    // Behavior defined by https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    #[test]
    #[should_panic(expected = "Can only increment `FloatCounter` by positive numbers")]
    fn negative_float_panics() {
        let float = FloatCounter::new("some_float", "Counts things").unwrap();
        float.inc_by(-10.0);
    }

    // Behavior defined by https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    #[test]
    fn negative_float_err() {
        let float = FloatCounter::new("some_float", "Counts things").unwrap();
        assert!(float.try_inc_by(-10.0).is_err());
    }

    #[test]
    fn int_counter() {
        let int = IntCounter::new("some_int", "Counts things").unwrap();

        assert_eq!(int.name(), "some_int");

        int.inc();
        assert_eq!(int.get(), 1);

        int.inc();
        assert_eq!(int.get(), 2);

        int.clear();
        assert_eq!(int.get(), 0);

        int.inc_by(10);
        assert_eq!(int.get(), 10);

        assert!(int.try_inc_by(10).is_ok());
        assert_eq!(int.get(), 20);

        int.inc_by(0);
        assert_eq!(int.get(), 20);

        int.set(999);
        assert_eq!(int.get(), 999);
    }

    #[test]
    #[cfg(not(miri))]
    fn int_threaded() {
        static INT: Lazy<IntCounter> =
            Lazy::new(|| IntCounter::new("surfin_the_world_wide_thread", "Counts things").unwrap());

        let mut threads = Vec::with_capacity(5);
        for _ in 0..5 {
            threads.push(thread::spawn(|| {
                INT.inc();
            }));
        }

        for thread in threads {
            thread.join().unwrap();
        }

        assert_eq!(INT.get(), 5);
    }

    // Behavior defined by https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    #[test]
    #[should_panic(expected = "Can only increment `IntCounter` by positive numbers")]
    fn negative_int_panics() {
        let int = IntCounter::new("some_int", "Counts things").unwrap();
        int.inc_by(-10);
    }

    // Behavior defined by https://prometheus.io/docs/instrumenting/writing_clientlibs/#counter
    #[test]
    fn negative_int_err() {
        let int = IntCounter::new("some_int", "Counts things").unwrap();
        assert!(int.try_inc_by(-10).is_err());
    }
}
