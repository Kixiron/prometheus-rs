use crate::{
    atomics::{AtomicF64, AtomicNum, Num},
    error::Result,
    label::Label,
    registry::{Collectable, Descriptor},
    timer::Timer,
};
use std::{
    borrow::Cow,
    fmt::{self, Write},
    sync::atomic::{AtomicI64, AtomicU64},
    time::{Instant, SystemTime},
};

pub type UintGauge = Gauge<AtomicU64>;
pub type FloatGauge = Gauge<AtomicF64>;
pub type IntGauge = Gauge<AtomicI64>;

/// [Definition](https://prometheus.io/docs/instrumenting/writing_clientlibs/#gauge)
#[derive(Debug)]
pub struct Gauge<Atomic: AtomicNum = AtomicU64> {
    value: Atomic,
    descriptor: Descriptor,
}

impl<Atomic: AtomicNum> Gauge<Atomic> {
    pub fn new(name: impl Into<Cow<'static, str>>, help: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            value: Atomic::new(),
            descriptor: Descriptor::new(name, help, Vec::new())?,
        })
    }

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

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Self> {
        Timer::new(self)
    }

    pub fn time_closure(&self, closure: impl Fn()) {
        let start = Instant::now();
        closure();
        let elapsed = start.elapsed().as_secs();

        self.set(Atomic::Type::from_u64(elapsed))
    }

    pub fn name(&self) -> &str {
        &self.descriptor.name()
    }

    pub fn help(&self) -> &str {
        &self.descriptor.help()
    }

    pub fn labels(&self) -> &[Label] {
        &self.descriptor.labels
    }

    pub fn with_labels(mut self, labels: impl Into<Vec<Label>>) -> Self {
        self.descriptor.labels = labels.into();
        self
    }
}

impl<Atomic: AtomicNum> Collectable for &Gauge<Atomic> {
    fn encode_text<'a>(&'a self, buf: &mut String) -> fmt::Result {
        writeln!(buf, "# HELP {} {}", self.name(), self.help())?;
        writeln!(buf, "# TYPE {} gauge", self.name())?;

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
    use once_cell::sync::Lazy;
    use std::{thread, time::Duration};

    #[test]
    fn uint_gauge() {
        let uint: Gauge<AtomicU64> = Gauge::new("some_uint", "Counts things").unwrap();

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
        let uint: Gauge<AtomicU64> = Gauge::new("some_uint", "Counts things").unwrap();

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
        static UINT: Lazy<Gauge<AtomicU64>> =
            Lazy::new(|| Gauge::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
        let float: Gauge<AtomicF64> = Gauge::new("some_float", "Counts things").unwrap();

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
        let float: Gauge<AtomicF64> = Gauge::new("some_float", "Counts things").unwrap();

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
        static FLOAT: Lazy<Gauge<AtomicF64>> =
            Lazy::new(|| Gauge::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
        let int: Gauge<AtomicI64> = Gauge::new("some_int", "Counts things").unwrap();

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
        let int: Gauge<AtomicI64> = Gauge::new("some_int", "Counts things").unwrap();

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
        static INT: Lazy<Gauge<AtomicI64>> =
            Lazy::new(|| Gauge::new("surfin_the_world_wide_thread", "Counts things").unwrap());

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
