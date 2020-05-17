#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::dbg_macro,
    clippy::missing_safety_doc,
    clippy::wildcard_imports,
    clippy::shadow_unrelated
)]

mod atomics;
mod counter;
mod error;
mod export;
mod gauge;
mod histogram;
mod label;
mod timer;

pub use counter::{FloatCounter, IntCounter, UintCounter};
pub use error::{PromError, PromErrorKind};
pub use gauge::{FloatGauge, IntGauge, UintGauge};
pub use histogram::Histogram;
pub use timer::Timer;

/*
use std::{hash::Hash, ops};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Float64(u64);

impl Float64 {
    pub fn new(float: f64) -> Self {
        Self(f64::to_bits(float))
    }
}

impl ops::Add<f64> for Float64 {
    type Output = Self;

    fn add(self, rhs: f64) -> Self::Output {
        let val = f64::from_bits(self.0) + rhs;

        Self(f64::to_bits(val))
    }
}

impl ops::Sub<f64> for Float64 {
    type Output = Self;

    fn sub(self, rhs: f64) -> Self::Output {
        let val = f64::from_bits(self.0) - rhs;

        Self(f64::to_bits(val))
    }
}
*/
