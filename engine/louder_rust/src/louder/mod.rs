//! The dialogue engine, and the Godot classes it is registered as.
//!
//! Each module here holds one part of the engine and, beside it, the
//! declarations that make that part reachable from GDScript. Those declarations
//! are pure additions: everything the engine does is still available to Rust
//! through the ordinary methods next to them, and with the `godot-bindings`
//! feature off the registered methods are not compiled at all and no class is
//! registered.
//!
//! See [`godot_api`] for how the two halves of that work, and in particular for
//! why the classes carry `cfg_attr` rather than sitting inside a macro.

// First, so that `godot_api!` is in scope for every module below.
#[macro_use]
mod godot_api;

pub mod actor;
pub mod callback;
pub mod conversation;
pub mod context;
pub mod event;
pub mod expression;
pub mod ffi;
pub mod hub;
pub mod json;
pub mod json_loader;
pub mod name;
pub mod object;
pub mod query;
pub mod rules;
pub mod simulator;
pub mod speaker;