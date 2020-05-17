use crate::{
    atomics::{AtomicF64, AtomicNum},
    error::Result,
    label::Label,
    registry::{Collectable, Descriptor},
};
use std::{
    borrow::Cow,
    fmt::{self, Write},
    sync::atomic::{AtomicI64, AtomicU64},
};

/// A [`Counter`] that stores a `u64`, see [`Counter`] for more information
///
/// [`Counter`]: crate::Counter
pub type UintCounter = Counter<AtomicU64>;

/// A [`Counter`] that stores an `i64`, see [`Counter`] for more information
///
/// [`Counter`]: crate::Counter
pub type IntCounter = Counter<AtomicI64>;

/// A [`Counter`] that stores a `f64`, see [`Counter`] for more information
///
/// [`Counter`]: crate::Counter
pub type FloatCounter = Counter<AtomicF64>;

/// A monotonically increasing counter. When in doubt of what type to choose, default to [`AtomicU64`]
///
/// To quote the [docs]:
///
/// > A counter is a cumulative metric that represents a single monotonically increasing counter whose value can only increase or be reset to zero on restart.
/// > For example, you can use a counter to represent the number of requests served, tasks completed, or errors.
/// >
/// > Do not use a counter to expose a value that can decrease. For example, do not use a counter for the number of currently running processes; instead use a [gauge].
///
/// [`AtomicU64`]: https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU64.html
/// [docs]: https://prometheus.io/docs/concepts/metric_types/#counter
/// [gauge]: crate::Gauge
#[derive(Debug)]
pub struct Counter<Atomic: AtomicNum = AtomicU64> {
    /// The inner atomically manipulated value
    value: Atomic,
    descriptor: Descriptor,
}

impl<Atomic: AtomicNum> Counter<Atomic> {
    pub fn new(name: impl Into<Cow<'static, str>>, help: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            value: Atomic::new(),
            descriptor: Descriptor::new(name, help, Vec::new())?,
        })
    }

    /// Increment the current counter by 1
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc();
    /// assert_eq!(counter.get(), 1);
    /// ```
    pub fn inc(&self) {
        self.value.inc();
    }

    /// Increment the current counter by `inc`
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.inc_by(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn inc_by(&self, inc: Atomic::Type) {
        self.value.inc_by(inc);
    }

    /// Get the value of the current counter
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn get(&self) -> Atomic::Type {
        self.value.get()
    }

    /// Reset the current counter's value to 0
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// counter.clear();
    /// assert_eq!(counter.get(), 0);
    /// ```
    pub fn clear(&self) {
        self.value.clear()
    }

    /// Set the current counter's value to `val`
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// counter.set(100);
    /// assert_eq!(counter.get(), 100);
    /// ```
    pub fn set(&self, val: Atomic::Type) {
        self.value.set(val)
    }

    /// Get the current counter's name
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.name(), "count_dracula");
    /// ```
    pub fn name(&self) -> &str {
        &self.descriptor.name()
    }

    /// Get the current counter's help
    ///
    /// ```rust
    /// use prometheus_rs::Counter;
    /// use std::sync::atomic::AtomicU64;
    ///
    /// let counter: Counter<AtomicU64> = Counter::new("count_dracula", "I am Count von Count!").unwrap();
    /// assert_eq!(counter.help(), "I am Count von Count!");
    /// ```
    pub fn help(&self) -> &str {
        &self.descriptor.help()
    }

    pub fn labels(&self) -> &[Label] {
        &self.descriptor.labels()
    }

    /// Set the labels of the current counter
    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.descriptor.labels = labels.into();
        self
    }
}

impl<Atomic: AtomicNum> Collectable for &'static Counter<Atomic> {
    fn encode_text<'a>(&'a self, buf: &mut String) -> fmt::Result {
        writeln!(buf, "# HELP {} {}", self.name(), self.help())?;
        writeln!(buf, "# TYPE {} counter", self.name())?;

        write!(buf, "{}", self.name())?;
        if !self.labels().is_empty() {
            write!(buf, "{{")?;

            let (last, labels) = self
                .labels()
                .split_last()
                .expect("There is at least 1 label");
            for label in labels {
                write!(buf, "{}={:?},", label.name(), label.value())?;
            }
            write!(buf, "{}={:?}", last.name(), last.value())?;

            write!(buf, "}} ")?;
        } else {
            write!(buf, " ")?;
        }

        Atomic::format(self.get(), buf, false)?;
        writeln!(buf)?;

        Ok(())
    }

    fn descriptor(&self) -> &Descriptor {
        &self.descriptor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::atomics::AtomicF64;
    use once_cell::sync::Lazy;
    use std::sync::atomic::{AtomicI64, AtomicU64};
    use std::thread;

    #[test]
    fn uint_counter() {
        let uint: Counter<AtomicU64> = Counter::new("some_uint", "Counts things").unwrap();

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
        static UINT: Lazy<Counter<AtomicU64>> =
            Lazy::new(|| Counter::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
        let float: Counter<AtomicF64> = Counter::new("some_float", "Counts things").unwrap();

        assert_eq!(float.name(), "some_float");

        float.inc();
        assert_eq!(float.get(), 1.0);

        float.inc();
        assert_eq!(float.get(), 2.0);

        float.clear();
        assert_eq!(float.get(), 0.0);

        float.inc_by(10.0);
        assert_eq!(float.get(), 10.0);

        float.inc_by(0.0);
        assert_eq!(float.get(), 10.0);

        float.set(999.999);
        assert_eq!(float.get(), 999.999);
    }

    #[test]
    #[cfg(not(miri))]
    fn float_threaded() {
        static FLOAT: Lazy<Counter<AtomicF64>> =
            Lazy::new(|| Counter::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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

    #[test]
    fn int_counter() {
        let int: Counter<AtomicI64> = Counter::new("some_int", "Counts things").unwrap();

        assert_eq!(int.name(), "some_int");

        int.inc();
        assert_eq!(int.get(), 1);

        int.inc();
        assert_eq!(int.get(), 2);

        int.clear();
        assert_eq!(int.get(), 0);

        int.inc_by(10);
        assert_eq!(int.get(), 10);

        int.inc_by(0);
        assert_eq!(int.get(), 10);

        int.set(999);
        assert_eq!(int.get(), 999);
    }

    #[test]
    #[cfg(not(miri))]
    fn int_threaded() {
        static INT: Lazy<Counter<AtomicI64>> =
            Lazy::new(|| Counter::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
}
