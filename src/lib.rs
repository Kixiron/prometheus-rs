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
mod gauge;
mod histogram;
mod label;
mod timer;

pub use counter::{Counter, FloatCounter, IntCounter, UintCounter};
pub use error::{PromError, PromErrorKind};
pub use gauge::{FloatGauge, Gauge, IntGauge, UintGauge};
pub use histogram::{Histogram, HistogramBuilder, LocalHistogram, DEFAULT_BUCKETS};
pub use timer::Timer;
