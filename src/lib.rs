#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    clippy::dbg_macro,
    clippy::missing_safety_doc,
    clippy::wildcard_imports,
    clippy::shadow_unrelated
)]

mod atomics;
pub mod counter;
mod error;
pub mod gauge;
pub mod histogram;
mod label;
mod registry;
mod timer;

pub use atomics::AtomicF64;
pub use counter::{Counter, FloatCounter, IntCounter, UintCounter};
pub use error::{PromError, PromErrorKind};
pub use gauge::{FloatGauge, Gauge, IntGauge, UintGauge};
pub use registry::{Registry, RegistryBuilder};
pub use timer::Timer;
