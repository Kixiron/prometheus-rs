use crate::{
    atomics::{AtomicNum, Num},
    gauge::Gauge,
    histogram::Histogram,
};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Timer<'a, Target: Observable> {
    target: &'a Target,
    start_time: Instant,
}

impl<'a, Target: Observable> Timer<'a, Target> {
    pub fn new(target: &'a Target) -> Self {
        Self {
            target,
            start_time: Instant::now(),
        }
    }

    pub fn observe(self) {
        // Drops the instance, letting the Drop impl do its thing
    }
}

impl<Target: Observable> Drop for Timer<'_, Target> {
    fn drop(&mut self) {
        self.target.observe(self.start_time.elapsed().as_secs());
    }
}

pub trait Observable {
    fn observe(&self, val: u64);
}

impl<Atomic: AtomicNum> Observable for Histogram<Atomic> {
    #[inline(always)]
    fn observe(&self, val: u64) {
        self.observe(Num::from_u64(val));
    }
}

impl<Atomic: AtomicNum> Observable for Gauge<Atomic> {
    #[inline(always)]
    fn observe(&self, val: u64) {
        self.set(Num::from_u64(val));
    }
}
