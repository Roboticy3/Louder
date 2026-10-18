//! Names: the integers the engine stores strings under, and how to get at the
//! strings again.

use crate::context::LouderContext;
use crate::engine;
use crate::engine::{LouderInternContext, LouderNameContext};
use louder_rust::godot_api;

use godot::prelude::*;

/// The value behind a [`LouderName`](engine::LouderName).
///
/// The engine stores names as bytes, so what comes back out of one is those same
/// bytes: [`LouderNameContent::bytes`] for a host that wants them as they are,
/// and [`LouderNameContent::text`] for one that reads them as UTF-8. Nothing
/// here converts on the way through.
#[derive(Clone, Default, PartialEq, Eq)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderNameContent {
  pub(crate) inner: engine::LouderNameContent,
}

godot_api! {
  impl LouderNameContent {
    /// A name holding `text`.
    #[func]
    pub fn from_text(text: GString) -> Gd<LouderNameContent> {
      Gd::from_object(LouderNameContent {
        inner: engine::LouderNameContent::new(&text.to_string()),
      })
    }

    /// The content as text, or an empty string if the bytes are not UTF-8.
    #[func]
    pub fn text(&self) -> GString {
      GString::from(self.inner.as_str().unwrap_or(""))
    }

    /// Replace the content with `text`.
    #[func]
    pub fn set_text(&mut self, text: GString) {
      self.inner = engine::LouderNameContent::new(&text.to_string());
    }

    /// The raw content, byte for byte.
    #[func]
    pub fn bytes(&self) -> PackedByteArray {
      self.inner.bytes.iter().copied().collect()
    }

    /// How many bytes the content is.
    #[func]
    pub fn size(&self) -> i32 {
      self.inner.bytes.len() as i32
    }

    /// The id `context` already gives `text`, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn find(context: Gd<LouderContext>, text: GString) -> engine::LouderName {
      let content = engine::LouderNameContent::new(&text.to_string());
      match context.bind().inner.find_name(&content) {
        engine::LouderContextResult::Found(name) => name,
        engine::LouderContextResult::NotFound(_) => engine::LOUDER_NAME_NONE,
      }
    }

    /// The id `context` gives `text`, assigning one if it has none.
    ///
    /// A name that was already declared keeps the id it had.
    #[func]
    pub fn declare(mut context: Gd<LouderContext>, text: GString) -> engine::LouderName {
      let content = engine::LouderNameContent::new(&text.to_string());
      match context.bind_mut().inner.add_name(&content) {
        engine::LouderContextResult::Found(_) => engine::LOUDER_NAME_NONE,
        engine::LouderContextResult::NotFound(name) => name,
      }
    }

    /// The text a name id stands for, or an empty string if there is none.
    #[func]
    pub fn resolve(context: Gd<LouderContext>, name: engine::LouderName) -> GString {
      match context.bind().inner.resolve_name(name) {
        Some(text) => GString::from(text),
        None => GString::from(""),
      }
    }

    /// `text` split on commas and/or runs of whitespace.
    #[func]
    pub fn split_inline(text: GString) -> Array<GString> {
      engine::split_inline(&text.to_string())
        .into_iter()
        .map(GString::from)
        .collect()
    }

    /// `text` split on newlines, dropping empty entries.
    #[func]
    pub fn split_lines(text: GString) -> Array<GString> {
      engine::split_lines(&text.to_string())
        .into_iter()
        .map(GString::from)
        .collect()
    }
  }
}