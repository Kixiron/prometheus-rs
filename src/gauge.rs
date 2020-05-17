use crate::{
    atomics::{AtomicF64, AtomicNum, Num},
    error::PromError,
    label::{Label, Labeled, Metric},
    timer::Timer,
};
use std::{
    sync::atomic::{AtomicI64, AtomicU64},
    time::SystemTime,
};

/// [Definition](https://prometheus.io/docs/instrumenting/writing_clientlibs/#gauge)
#[derive(Debug)]
pub struct Gauge<Atomic: AtomicNum = AtomicU64> {
    value: Atomic,
}

impl Gauge<AtomicU64> {
    pub const fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }
}

impl Gauge<AtomicF64> {
    pub const fn new() -> Self {
        Self {
            value: AtomicF64::zeroed(),
        }
    }
}

impl Gauge<AtomicI64> {
    pub const fn new() -> Self {
        Self {
            value: AtomicI64::new(0),
        }
    }
}

impl<Atomic: AtomicNum> Gauge<Atomic> {
    pub fn inc(&self) {
        self.value.inc();
    }

    pub fn inc_by(&self, inc: Atomic::Type) {
        self.value.inc_by(inc);
    }

    pub fn dec(&self) {
        self.value.dec();
    }

    pub fn dec_by(&self, dec: Atomic::Type) {
        self.value.dec_by(dec);
    }

    pub fn set(&self, val: Atomic::Type) {
        self.value.set(val);
    }

    pub fn get(&self) -> Atomic::Type {
        self.value.get()
    }

    pub fn clear(&self) {
        self.value.clear()
    }

    pub fn set_to_current_time(&self) {
        let current_time = SystemTime::UNIX_EPOCH
            .elapsed()
            .expect("Impossible to fail, `UNIX_EPOCH` will never be sooner than the current system time")
            .as_secs();

        self.value.set(Atomic::Type::from_u64(current_time));
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Gauge<Atomic>> {
        Timer::new(self)
    }
}

impl<Atomic: AtomicNum> Metric for Gauge<Atomic> {
    fn metric_kind(&self) -> &'static str {
        "gauge"
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct UintGauge(Labeled<Gauge<AtomicU64>>);

impl UintGauge {
    pub fn new(name: &'static str, description: &'static str) -> Result<Self, PromError> {
        Ok(Self(Labeled::new(
            <Gauge<AtomicU64>>::new(),
            name,
            description,
        )?))
    }

    pub fn inc(&self) {
        self.0.inc();
    }

    pub fn inc_by(&self, inc: u64) {
        self.0.inc_by(inc);
    }

    pub fn dec(&self) {
        self.0.dec();
    }

    pub fn dec_by(&self, dec: u64) {
        self.0.dec_by(dec);
    }

    pub fn set(&self, val: u64) {
        self.0.set(val);
    }

    pub fn get(&self) -> u64 {
        self.0.get()
    }

    pub fn clear(&self) {
        self.0.clear()
    }

    pub fn set_to_current_time(&self) {
        self.0.set_to_current_time();
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Gauge<AtomicU64>> {
        self.0.start_timer()
    }

    pub fn name(&self) -> &str {
        &self.0.name()
    }

    pub fn description(&self) -> &str {
        &self.0.description()
    }

    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.0.labels = labels.into();
        self
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct IntGauge(Labeled<Gauge<AtomicI64>>);

impl IntGauge {
    pub fn new(name: &'static str, description: &'static str) -> Result<Self, PromError> {
        Ok(Self(Labeled::new(
            <Gauge<AtomicI64>>::new(),
            name,
            description,
        )?))
    }

    pub fn inc(&self) {
        self.0.inc();
    }

    pub fn inc_by(&self, inc: i64) {
        self.0.inc_by(inc);
    }

    pub fn dec(&self) {
        self.0.dec();
    }

    pub fn dec_by(&self, dec: i64) {
        self.0.dec_by(dec);
    }

    pub fn set(&self, val: i64) {
        self.0.set(val);
    }

    pub fn get(&self) -> i64 {
        self.0.get()
    }

    pub fn clear(&self) {
        self.0.clear()
    }

    pub fn set_to_current_time(&self) {
        self.0.set_to_current_time();
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Gauge<AtomicI64>> {
        self.0.start_timer()
    }

    pub fn name(&self) -> &str {
        &self.0.name()
    }

    pub fn description(&self) -> &str {
        &self.0.description()
    }

    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.0.labels = labels.into();
        self
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct FloatGauge(Labeled<Gauge<AtomicF64>>);

impl FloatGauge {
    pub fn new(name: &'static str, description: &'static str) -> Result<Self, PromError> {
        Ok(Self(Labeled::new(
            <Gauge<AtomicF64>>::new(),
            name,
            description,
        )?))
    }

    pub fn inc(&self) {
        self.0.inc();
    }

    pub fn inc_by(&self, inc: f64) {
        self.0.inc_by(inc);
    }

    pub fn dec(&self) {
        self.0.dec();
    }

    pub fn dec_by(&self, dec: f64) {
        self.0.dec_by(dec);
    }

    pub fn set(&self, val: f64) {
        self.0.set(val);
    }

    pub fn get(&self) -> f64 {
        self.0.get()
    }

    pub fn clear(&self) {
        self.0.clear()
    }

    pub fn set_to_current_time(&self) {
        self.0.set_to_current_time();
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Gauge<AtomicF64>> {
        self.0.start_timer()
    }

    pub fn name(&self) -> &str {
        &self.0.name()
    }

    pub fn description(&self) -> &str {
        &self.0.description()
    }

    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.0.labels = labels.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use once_cell::sync::Lazy;
    use std::{thread, time::Duration};

    #[test]
    fn uint_gauge() {
        let uint = UintGauge::new("some_uint", "Counts things").unwrap();

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
    fn uint_gauge_timer() {
        let uint = UintGauge::new("some_uint", "Counts things").unwrap();

        {
            let _timer = uint.start_timer();
            thread::sleep(Duration::from_millis(100));
        }

        assert_eq!(Duration::from_millis(100).as_secs(), uint.get());

        let timer = uint.start_timer();
        thread::sleep(Duration::from_millis(100));
        timer.observe();

        assert_eq!(Duration::from_millis(100).as_secs(), uint.get());
    }

    #[test]
    #[cfg(not(miri))]
    fn uint_threaded() {
        static UINT: Lazy<UintGauge> =
            Lazy::new(|| UintGauge::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
    fn float_gauge() {
        let float = FloatGauge::new("some_float", "Counts things").unwrap();

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
    fn float_gauge_timer() {
        let float = FloatGauge::new("some_float", "Counts things").unwrap();

        {
            let _timer = float.start_timer();
            thread::sleep(Duration::from_millis(100));
        }

        assert_eq!(Duration::from_millis(100).as_secs() as f64, float.get());

        let timer = float.start_timer();
        thread::sleep(Duration::from_millis(100));
        timer.observe();

        assert_eq!(Duration::from_millis(100).as_secs() as f64, float.get());
    }

    #[test]
    #[cfg(not(miri))]
    fn float_threaded() {
        static FLOAT: Lazy<FloatGauge> =
            Lazy::new(|| FloatGauge::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
    fn int_gauge() {
        let int = IntGauge::new("some_int", "Counts things").unwrap();

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
    fn int_gauge_timer() {
        let int = IntGauge::new("some_int", "Counts things").unwrap();

        {
            let _timer = int.start_timer();
            thread::sleep(Duration::from_millis(100));
        }

        assert_eq!(Duration::from_millis(100).as_secs() as i64, int.get());

        let timer = int.start_timer();
        thread::sleep(Duration::from_millis(100));
        timer.observe();

        assert_eq!(Duration::from_millis(100).as_secs() as i64, int.get());
    }

    #[test]
    #[cfg(not(miri))]
    fn int_threaded() {
        static INT: Lazy<IntGauge> =
            Lazy::new(|| IntGauge::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
