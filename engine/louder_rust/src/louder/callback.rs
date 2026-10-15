//! The callback vocabulary.
//!
//! A callback is named rather than carried: a hub lists the callbacks it should
//! run, and the engine interns each of those names and announces them through an
//! [`EventData::Callback`](super::event::EventData) event. The host looks the
//! name back up and calls whatever it means, which is what keeps the engine from
//! having to know anything about the game.
//!
//! There is no type here to register with Godot, since a callback is just a
//! name; see [`LouderNameContent`](super::name::LouderNameContent) and
//! [`LouderEvent`](super::event::LouderEvent) for the Godot side.

use super::name::LouderName;

/// A thing the host is asked to run when an actor first reaches a hub.
///
/// This is the interned copy of the callback's own text, which may be the text
/// of its arguments as well.
pub type LouderCallback = LouderName;