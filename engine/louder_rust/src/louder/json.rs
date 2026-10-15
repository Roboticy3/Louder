//! Writing what the engine holds back out as JSON.
//!
//! This is the other half of [`super::json_loader`]: documents come in through
//! that module and leave through this one, in the shape a save file or a
//! debugging dump wants. Nothing here parses. Every document is assembled from
//! the context as it stands, and both are written in a fixed order so that
//! asking twice gives the same bytes.

#[cfg(feature = "godot-bindings")]
use super::godot_api::{Gd, GString};
use super::context::LouderContext;
use super::name::{LouderInternContext, LouderName};
use super::object::LouderVariant;

/// Escape `text` as a JSON string, quotes included.
fn quote(text: &str) -> String {
  let mut out = String::with_capacity(text.len() + 2);
  out.push('"');
  for c in text.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
      c => out.push(c),
    }
  }
  out.push('"');
  out
}

/// Render a variant as JSON, resolving strings against the context.
fn variant(context: &LouderContext, value: &LouderVariant) -> String {
  match value {
    LouderVariant::Int(i) => i.to_string(),
    LouderVariant::Float(f) => {
      // Whole floats print without a trailing `.0` so that round-tripping an
      // integer stored as a float does not change how it reads.
      if f.fract() == 0.0 && f.is_finite() {
        format!("{}", *f as i64)
      } else {
        f.to_string()
      }
    }
    LouderVariant::String(name) => match context.resolve_name(*name) {
      Some(text) => quote(text),
      None => quote(""),
    },
  }
}

/// Render a set of name/variant pairs as a JSON object, in ascending key order.
fn object(
  context: &LouderContext,
  keys: &[LouderName],
  values: impl Fn(LouderName) -> LouderVariant,
) -> String {
  let mut out = String::from("{");
  for (index, key) in keys.iter().enumerate() {
    if index > 0 {
      out.push(',');
    }
    let name = match context.resolve_name(*key) {
      Some(text) => quote(text),
      None => quote(""),
    };
    out.push_str(&format!("{}:{}", name, variant(context, &values(*key))));
  }
  out.push('}');
  out
}

/// Render a list of names as a JSON array of strings.
///
/// A name with nothing behind it is written as null rather than as an empty
/// string, so that a list entry the engine cannot account for stays visible as
/// a gap instead of reading as text.
fn name_list(context: &LouderContext, names: &[LouderName]) -> String {
  let mut out = String::from("[");
  for (index, name) in names.iter().enumerate() {
    if index > 0 {
      out.push(',');
    }
    match context.resolve_name(*name) {
      Some(text) => out.push_str(&quote(text)),
      None => out.push_str("null"),
    }
  }
  out.push(']');
  out
}

/// The whole story state of a conversation, as a JSON object.
///
/// Every bucket is included: the conversation's own saved and unsaved variables,
/// and the shared ones.
pub fn state(context: &LouderContext, conversation: LouderName) -> String {
  let keys = context.all_state_keys(conversation);
  object(context, &keys, |key| context.get_state(conversation, key))
}

/// The story state and every actor of a conversation, as a JSON document.
///
/// `None` when the conversation is not loaded, since there is then nothing to
/// describe. The document has a `state` object and an `actors` array.
pub fn snapshot(context: &LouderContext, conversation: LouderName) -> Option<String> {
  context.get_conversation(conversation)?;

  let state = state(context, conversation);

  let mut actors = String::from("[");
  let entries = context.actors(conversation);
  for (index, entry) in entries.iter().enumerate() {
    if index > 0 {
      actors.push(',');
    }

    let channel = match context.resolve_name(entry.channel()) {
      Some(text) => quote(text),
      None => "null".to_string(),
    };

    let additional = object(context, &entry.additional().sorted_keys(), |key| {
      entry.additional_get(key).cloned().unwrap_or_default()
    });

    actors.push_str(&format!(
      "{{\"id\":{},\"position\":{},\"channel\":{},\"priority\":{},\"flags\":{},\"speak_as\":{},\"listen_to\":{},\"additional\":{}}}",
      entry.id(),
      entry.position(),
      channel,
      entry.priority(),
      entry.flags().bits(),
      name_list(context, entry.speak_as()),
      name_list(context, entry.listen_to()),
      additional
    ));
  }
  actors.push(']');

  Some(format!("{{\"state\":{},\"actors\":{}}}", state, actors))
}

godot_class! {
  /// What the engine holds, written back out as JSON.
  ///
  /// Every document is assembled from the context as it stands, and both are
  /// written in a fixed order, so that asking twice gives the same bytes. The
  /// C boundary returns these through a caller's buffer; here they are just
  /// strings.
  pub struct LouderJson {}
}

godot_api! {
  impl LouderJson {
    /// The whole story state of a conversation, as a JSON object.
    ///
    /// Every bucket is included: the conversation's own saved and unsaved
    /// variables, and the shared ones.
    #[func]
    pub fn state(context: Gd<LouderContext>, conversation: LouderName) -> GString {
      let context = context.bind();
      GString::from(state(&context, conversation).as_str())
    }

    /// The story state and every actor of a conversation, as a JSON document
    /// with a `state` object and an `actors` array.
    ///
    /// An empty string stands for a conversation that is not loaded, since
    /// there is then nothing to describe.
    #[func]
    pub fn snapshot(context: Gd<LouderContext>, conversation: LouderName) -> GString {
      let context = context.bind();
      match snapshot(&context, conversation) {
        Some(document) => GString::from(document.as_str()),
        None => GString::from(""),
      }
    }
  }
}