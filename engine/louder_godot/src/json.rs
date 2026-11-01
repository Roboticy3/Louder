//! What the engine holds, written back out as JSON.

use crate::context::LouderContext;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The engine's own serialisations.
///
/// Every document is assembled from the context as it stands and written in a
/// fixed order, so asking twice gives the same bytes. The C boundary returns
/// these through a caller's buffer; here they are strings.
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderJson {}

godot_api! {
  impl LouderJson {
    /// The whole story state of a conversation, as a JSON object.
    ///
    /// Every bucket is included: the conversation's own saved and unsaved
    /// variables, and the shared ones.
    #[func]
    pub fn state(context: Gd<LouderContext>, conversation: engine::LouderName) -> GString {
      GString::from(engine::json::state(&context.bind().inner, conversation).as_str())
    }

    /// The story state and every actor of a conversation, as a JSON document
    /// with a `state` object and an `actors` array.
    ///
    /// An empty string stands for a conversation that is not loaded, since
    /// there is then nothing to describe.
    #[func]
    pub fn snapshot(context: Gd<LouderContext>, conversation: engine::LouderName) -> GString {
      match engine::json::snapshot(&context.bind().inner, conversation) {
        Some(document) => GString::from(document.as_str()),
        None => GString::from(""),
      }
    }

    /// Every event and ask the engine has created, as one JSON document with
    /// `events` and `asks` arrays.
    ///
    /// This is the trace rather than the current queue and cache, so it is not
    /// changed by flushing events or answering asks.
    #[func]
    pub fn trace(context: Gd<LouderContext>) -> GString {
      GString::from(engine::json::trace(&context.bind().inner).as_str())
    }
  }
}