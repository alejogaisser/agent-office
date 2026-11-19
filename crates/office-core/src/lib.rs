//! Agent Office core: pure Rust, no GUI, no watcher. See docs/decisions/0003.
pub mod config;
pub mod discovery;
pub mod layout;
pub mod line;
pub mod model;
pub mod office;
pub mod probe;
pub mod reducer;
pub mod roles;
pub mod tail;

pub use config::Config;
pub use model::*;
pub use office::Office;
pub use probe::{ProcessProbe, SystemProbe};
