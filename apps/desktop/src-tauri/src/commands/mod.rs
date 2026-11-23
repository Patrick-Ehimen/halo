//! Thin IPC layer. Commands validate input, call into the domain modules,
//! and map errors to `IpcError`. No logic lives here.

pub mod app;
pub mod plan;
pub mod stats;
pub mod timer;
