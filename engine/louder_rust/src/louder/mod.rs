//! The dialogue engine.
//!
//! Each module here holds one part of the engine, and each is reached through an
//! ordinary Rust API. None of them knows what host, if any, is driving it: a
//! host that wants the classes as Godot types declares them in the `louder_godot`
//! adapter over this crate, and a host that wants the `extern "C"` entry points
//! finds them in the crate root.
//!
//! The engine's own vocabulary is in [`context::LouderContext`]: one instance is
//! one running dialogue, holding its name table, its conversations, its actors,
//! its speaker config, its story state, its event queue, and its ask cache. The
//! types it hands out are plain Rust values, so a host is free to read them and
//! change them, and nothing here is tied to the shape any particular language
//! wants them in.

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
pub mod rule;
pub mod rules;
pub mod simulator;
pub mod speaker;