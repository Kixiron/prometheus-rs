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
mod group;
pub mod histogram;
mod label;
mod registry;
mod timer;

pub use atomics::AtomicF64;
pub use counter::Counter;
pub use error::{PromError, PromErrorKind};
pub use gauge::Gauge;
pub use group::{CounterGroup, Group, HistogramGroup, Key};
pub use label::Label;
pub use registry::{Registry, RegistryBuilder};
pub use timer::Timer;
