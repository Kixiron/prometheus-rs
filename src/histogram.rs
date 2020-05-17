use crate::{
    atomics::{AtomicF64, AtomicNum},
    counter::Counter,
    timer::Timer,
};
use std::{borrow::Cow, sync::atomic::AtomicU64};

/// [Definition](https://prometheus.io/docs/instrumenting/writing_clientlibs/#histogram)
#[derive(Debug)]
pub struct Histogram<Atomic: AtomicNum> {
    desc: Cow<'static, str>,
    buckets: Vec<f64>,
    values: Vec<Counter<Atomic>>,
    count: Counter<AtomicU64>,
    sum: Counter<AtomicF64>,
}

impl Histogram<AtomicU64> {
    pub const fn new(description: &'static str) -> Self {
        Self {
            desc: Cow::Borrowed(description),
            buckets: Vec::new(),
            values: Vec::new(),
            count: <Counter<AtomicU64>>::new(),
            sum: <Counter<AtomicF64>>::new(),
        }
    }
}

impl Histogram<AtomicF64> {
    pub const fn new(description: &'static str) -> Self {
        Self {
            desc: Cow::Borrowed(description),
            buckets: Vec::new(),
            values: Vec::new(),
            count: <Counter<AtomicU64>>::new(),
            sum: <Counter<AtomicF64>>::new(),
        }
    }
}

impl<Atomic: AtomicNum> Histogram<Atomic> {
    pub fn observe(&self, val: f64) {
        if let Some(idx) = self.buckets.iter().position(|b| val <= *b) {
            self.values[idx].inc();
        }

        self.count.inc();
        self.sum.inc_by(val);
    }

    pub fn start_timer<'a>(&'a self) -> Timer<'a, Histogram<Atomic>> {
        Timer::new(self)
    }
}
