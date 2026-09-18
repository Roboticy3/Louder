use super::object::{LouderName, LouderNameMap, Map};
use super::speaker::Speaker;
use super::actor::Actor;
use super::conversation::Conversation;
use super::event::{LouderEvent, LouderAsk};

/// The opaque handle to the engine.
///
/// Obtained from [`crate::init_louder`] and released with
/// [`crate::deinit_louder`]. The contents are internal to the engine and not
/// part of the public API.
pub struct LouderContext {
  names: LouderNameMap,
  speakers: Map<LouderName, Speaker>,
  actors: Map<LouderName, Actor>,
  conversations: Map<LouderName, Conversation>,
  events: Vec<LouderEvent>,
  asks: Vec<LouderAsk>,
}