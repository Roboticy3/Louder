//! Loading conversations and speaker configs out of JSON documents.

use crate::context::LouderContext;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The two loaders, and what their error codes mean.
///
/// Both take the document as a string. Godot has already decoded a `String` by
/// the time one exists, so there is nothing for the engine to be told about an
/// encoding here; a caller holding bytes has turned them into a string itself.
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderLoader {}

godot_api! {
  impl LouderLoader {
    /// Load a conversation from a JSON document.
    ///
    /// The document must be a list of objects: `hub` records become hubs,
    /// `connection` records become choices, and `camera` records are ignored.
    /// Problems with individual records are reported through
    /// [`LouderEvent`](crate::event::LouderEvent) events rather than failing the
    /// whole load, so one malformed hub does not cost the caller the rest of the
    /// conversation.
    #[func]
    pub fn load_conversation(
      mut context: Gd<LouderContext>,
      conversation: engine::LouderName,
      document: GString,
    ) -> i32 {
      engine::load_conversation(&mut context.bind_mut().inner, conversation, &document.to_string())
    }

    /// Load the speaker config from a JSON document.
    ///
    /// The config is a singleton: it is not tied to any one conversation, so a
    /// speaker declared here is visible from all of them. A speaker of the same
    /// name is replaced rather than treated as an error, so a config can be
    /// reloaded.
    #[func]
    pub fn load_config(mut context: Gd<LouderContext>, document: GString) -> i32 {
      engine::load_config(&mut context.bind_mut().inner, &document.to_string())
    }

    /// Load a conversation from a document in the readable text format.
    #[func]
    pub fn load_conversation_text(
      mut context: Gd<LouderContext>,
      conversation: engine::LouderName,
      document: GString,
    ) -> i32 {
      engine::text_loader::load_conversation(&mut context.bind_mut().inner, conversation, &document.to_string())
    }

    /// What a load error code means, for a host that would rather read a name
    /// than a number.
    #[func]
    pub fn error_name(code: i32) -> GString {
      GString::from(match code {
        engine::LOUDER_LOAD_OK => "OK",
        engine::LOUDER_LOAD_MALFORMED_JSON => "MALFORMED_JSON",
        engine::LOUDER_LOAD_NOT_A_LIST => "NOT_A_LIST",
        engine::LOUDER_LOAD_NOT_AN_OBJECT => "NOT_AN_OBJECT",
        engine::LOUDER_LOAD_ALREADY_LOADED => "ALREADY_LOADED",
        engine::LOUDER_LOAD_UNKNOWN_RECORD_TYPE => "UNKNOWN_RECORD_TYPE",
        engine::LOUDER_LOAD_BAD_CONNECTION => "BAD_CONNECTION",
        engine::LOUDER_LOAD_MISSING_HUB => "MISSING_HUB",
        engine::LOUDER_LOAD_DUPLICATE_POSITION => "DUPLICATE_POSITION",
        engine::LOUDER_LOAD_UNRESOLVED_TAG => "UNRESOLVED_TAG",
        _ => "UNKNOWN",
      })
    }
  }
}