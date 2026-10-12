use serde_json::{Map as JsonMap, Value};

use super::context::LouderContext;
use super::conversation::Conversation;
use super::event::{EventData, LoadFailure, LouderEvent};
use super::hub::{Choice, Hub, HubFlags, LouderPosition};
use super::name::*;
use super::object::{HomogeneousObject, LouderVariant};
use super::speaker::{Speaker, SpeakerFlags};

/// An error code returned by load operations.
///
/// Standard JSON syntax errors are returned, as well as errors relating to the
/// shape of the loaded document.
pub type LoadError = i32;

/// The document loaded without incident.
pub const LOUDER_LOAD_OK: LoadError = 0;
/// The document is not valid JSON.
pub const LOUDER_LOAD_MALFORMED_JSON: LoadError = 1;
/// The document parsed, but is not a list of records.
pub const LOUDER_LOAD_NOT_A_LIST: LoadError = 2;
/// A record was not an object.
pub const LOUDER_LOAD_NOT_AN_OBJECT: LoadError = 3;
/// The conversation name was already occupied by a loaded conversation.
pub const LOUDER_LOAD_ALREADY_LOADED: LoadError = 4;
/// The requested encoding is not supported.
pub const LOUDER_LOAD_BAD_ENCODING: LoadError = 5;

/// The only encoding the engine decodes.
///
/// Documents are exchanged as UTF-8, so every other index is rejected rather
/// than silently mis-decoded.
pub const LOUDER_ENCODING_UTF8: LouderEncoding = 0;

/// Decode `bytes` under `encoding`.
///
/// Only UTF-8 is accepted, because the editor writes UTF-8 and re-encoding
/// here would just move the problem to the host.
pub fn decode(bytes: &[u8], encoding: LouderEncoding) -> Result<&str, LoadError> {
  if encoding != LOUDER_ENCODING_UTF8 {
    return Err(LOUDER_LOAD_BAD_ENCODING);
  }

  std::str::from_utf8(bytes).map_err(|_| LOUDER_LOAD_MALFORMED_JSON)
}

/// The keys of a hub record that are part of the schema. Anything else is
/// additional data.
const HUB_KEYS: &[&str] = &[
  "type",
  "conversation_id",
  "speaker",
  "text",
  "tags",
  "choices",
  "expressions",
  "callbacks",
  "flags",
  "recipients",
  "channel",
  "priority",
];

/// The keys of a speaker record that are part of the schema.
const SPEAKER_KEYS: &[&str] = &["type", "name", "flags", "listening"];

/// Load a conversation from a JSON document.
///
/// The document must be a list of objects. Records whose `type` is `hub`
/// become hubs, `connection` records become choices, and `camera` records are
/// ignored. Problems with individual records are reported through events
/// rather than failing the whole load, so one malformed hub does not cost the
/// caller the rest of the conversation.
pub fn load_conversation(
  context: &mut LouderContext,
  conversation: LouderName,
  document: &str,
) -> LoadError {
  if context.get_conversation(conversation).is_some() {
    return LOUDER_LOAD_ALREADY_LOADED;
  }

  let records = match parse_records(context, conversation, document) {
    Ok(records) => records,
    Err(error) => return error,
  };

  let mut loaded = Conversation::new(conversation);
  // Connections are collected before being resolved because a choice takes its
  // text from the hub it links to, which may not have been read yet.
  let mut connections: Map<LouderPosition, Vec<(usize, LouderPosition)>> = Map::new();

  for record in records {
    let Some(object) = record.as_object() else {
      report(context, conversation, LOUDER_LOAD_NOT_AN_OBJECT, crate::BLANK_POSITION);
      continue;
    };

    match record_type(object) {
      Some("hub") => {
        let position = position_of(object);
        let hub = read_hub(context, object);
        if loaded.add_hub(hub) {
          // Two hubs claimed the same position; the later one won.
          report(context, conversation, LOUDER_LOAD_NOT_AN_OBJECT, position);
        }
      }
      Some("connection") => match read_connection(object) {
        Some((from, port, to)) => connections.entry(from).or_default().push((port, to)),
        None => report(context, conversation, LOUDER_LOAD_NOT_AN_OBJECT, position_of(object)),
      },
      // The camera record only remembers where the editor was scrolled to.
      Some("camera") => {}
      _ => report(context, conversation, LOUDER_LOAD_NOT_AN_OBJECT, position_of(object)),
    }
  }

  // Ports are numbered in the document and may arrive out of order, so each
  // group is sorted and compacted into dense indices. A gap in the numbering
  // therefore closes up rather than leaving a hole choices cannot be indexed
  // through.
  for (from, mut ports) in connections {
    ports.sort_unstable_by_key(|(port, _)| *port);
    ports.dedup_by_key(|(port, _)| *port);

    for (_, to) in ports {
      let target_text = match loaded.hub(to) {
        Some(hub) => hub.text().to_string(),
        None => {
          // The choice points at a position that does not exist.
          report(context, conversation, LOUDER_LOAD_NOT_AN_OBJECT, from);
          continue;
        }
      };

      let target_text_name = context.intern_optional(&target_text);
      let Some(hub) = loaded.hub_mut(from) else {
        continue;
      };
      hub.push_choice(Choice::new(target_text, target_text_name, to));
    }
  }

  context.add_conversation(loaded);
  LOUDER_LOAD_OK
}

/// Load the speaker config from a JSON document.
///
/// The config is a singleton: it is not tied to any one conversation, so a
/// speaker declared here is visible from all of them. Records replace any
/// speaker of the same name rather than failing, so a config can be reloaded.
///
/// A record the engine cannot read is reported and skipped rather than costing
/// the caller the rest of the config. The config belongs to no conversation, so
/// its events carry no conversation name.
pub fn load_config(context: &mut LouderContext, document: &str) -> LoadError {
  let records = match parse_records(context, LOUDER_NAME_NONE, document) {
    Ok(records) => records,
    Err(error) => return error,
  };

  for record in records {
    let Some(object) = record.as_object() else {
      report(context, LOUDER_NAME_NONE, LOUDER_LOAD_NOT_AN_OBJECT, crate::BLANK_POSITION);
      continue;
    };

    if record_type(object) != Some("speaker") {
      continue;
    }

    let name_text = string_field(object, "name");
    if name_text.is_empty() {
      // A speaker with no name is not a speaker the host could ever ask for.
      report(context, LOUDER_NAME_NONE, LOUDER_LOAD_NOT_AN_OBJECT, crate::BLANK_POSITION);
      continue;
    }

    let name = context.intern_name(&name_text);
    let flags = SpeakerFlags::from_bits_truncate(int_field(object, "flags"));
    let listening = context.intern_list(&split_lines(&string_field(object, "listening")));
    let additional = read_additional(context, object, SPEAKER_KEYS);

    context.set_speaker(Speaker::new(name, flags, listening, additional));
  }

  LOUDER_LOAD_OK
}

fn record_type(object: &JsonMap<String, Value>) -> Option<&str> {
  object.get("type").and_then(Value::as_str)
}

/// Read a scalar field as text.
///
/// Values that the schema calls numbers are still accepted as strings, so that
/// documents saved by an older editor revision keep loading.
fn string_field(object: &JsonMap<String, Value>, key: &str) -> String {
  match object.get(key) {
    Some(Value::String(text)) => text.clone(),
    Some(Value::Number(number)) => number.to_string(),
    Some(Value::Bool(value)) => value.to_string(),
    _ => String::new(),
  }
}

fn int_field(object: &JsonMap<String, Value>, key: &str) -> i32 {
  match object.get(key) {
    Some(Value::Number(number)) => number.as_i64().unwrap_or(0) as i32,
    Some(Value::String(text)) => text.trim().parse::<f64>().unwrap_or(0.0) as i32,
    Some(Value::Bool(value)) => {
      if *value {
        1
      } else {
        0
      }
    }
    _ => 0,
  }
}

fn position_of(object: &JsonMap<String, Value>) -> LouderPosition {
  int_field(object, "conversation_id")
}

fn read_connection(
  object: &JsonMap<String, Value>,
) -> Option<(LouderPosition, usize, LouderPosition)> {
  if !object.contains_key("from_id") || !object.contains_key("to_id") {
    return None;
  }

  let from = int_field(object, "from_id");
  let port = int_field(object, "from_port");
  let to = int_field(object, "to_id");

  if port < 0 || to < 0 {
    return None;
  }

  Some((from, port as usize, to))
}

fn read_hub(context: &mut LouderContext, object: &JsonMap<String, Value>) -> Hub {
  let position = position_of(object);

  let speaker = context.intern_optional(&string_field(object, "speaker"));

  let text = string_field(object, "text");
  let text_name = context.intern_optional(&text);

  let tags = context.intern_list(&split_inline(&string_field(object, "tags")));
  let recipients = context.intern_list(&split_lines(&string_field(object, "recipients")));
  let channel = context.intern_optional(&string_field(object, "channel"));

  let expressions_text = string_field(object, "expressions");
  let expressions = context.intern_list(&split_lines(&expressions_text));
  let expressions_joined = context.intern_optional(&expressions_text);

  let callbacks_text = string_field(object, "callbacks");
  let callbacks = context.intern_list(&split_lines(&callbacks_text));
  let callbacks_joined = context.intern_optional(&callbacks_text);

  let flags = HubFlags::from_bits_truncate(int_field(object, "flags"));
  let priority = int_field(object, "priority");
  let additional = read_additional(context, object, HUB_KEYS);

  Hub::new(
    position,
    speaker,
    text,
    text_name,
    tags,
    recipients,
    channel,
    priority,
    expressions,
    expressions_joined,
    callbacks,
    callbacks_joined,
    flags,
    additional,
  )
}

/// Turn a JSON value into a variant.
///
/// Objects and arrays have no variant of their own, so they are interned as
/// their compact JSON text, which keeps them readable without inventing a
/// fourth variant kind.
fn read_variant(context: &mut LouderContext, value: &Value) -> LouderVariant {
  match value {
    Value::Null => LouderVariant::Int(0),
    Value::Bool(value) => LouderVariant::from_bool(*value),
    Value::Number(number) => match number.as_i64() {
      Some(value) => LouderVariant::Int(value as i32),
      None => LouderVariant::Float(number.as_f64().unwrap_or(0.0) as f32),
    },
    Value::String(text) => LouderVariant::String(context.intern_optional(text)),
    other => LouderVariant::String(context.intern_optional(&other.to_string())),
  }
}

/// Report a load problem as an event, so that one bad record does not abort the
/// whole load.
///
/// `position` is the record at fault, or [`crate::BLANK_POSITION`] when the problem was
/// with the document as a whole.
fn report(context: &mut LouderContext, conversation: LouderName, error: LoadError, position: LouderPosition) {
  context.push_event(LouderEvent {
    conversation,
    channel: LOUDER_NAME_NONE,
    actor: super::actor::LOUDER_ACTOR_NONE,
    data: EventData::LoadError(LoadFailure { position, error }),
  });
}

/// Parse `document` as the list of records both loaders expect.
///
/// A problem with the document as a whole is reported as a load error event
/// before the error is returned, so that a host reading events rather than
/// return codes still learns that the load failed and why.
fn parse_records(
  context: &mut LouderContext,
  conversation: LouderName,
  document: &str,
) -> Result<Vec<Value>, LoadError> {
  let parsed: Value = match serde_json::from_str(document) {
    Ok(parsed) => parsed,
    Err(_) => {
      report(context, conversation, LOUDER_LOAD_MALFORMED_JSON, crate::BLANK_POSITION);
      return Err(LOUDER_LOAD_MALFORMED_JSON);
    }
  };

  match parsed {
    Value::Array(records) => Ok(records),
    _ => {
      report(context, conversation, LOUDER_LOAD_NOT_A_LIST, crate::BLANK_POSITION);
      Err(LOUDER_LOAD_NOT_A_LIST)
    }
  }
}

/// Everything on a record the schema does not name, read as a homogeneous
/// object.
///
/// `schema` is the set of fields the loader has already taken, so this picks up
/// the user-defined properties without any of them being listed twice.
fn read_additional(
  context: &mut LouderContext,
  object: &JsonMap<String, Value>,
  schema: &[&str],
) -> HomogeneousObject {
  let mut additional = HomogeneousObject::new();
  for (key, value) in object {
    if schema.contains(&key.as_str()) {
      continue;
    }
    let key_name = context.intern_name(key);
    additional.set(key_name, read_variant(context, value));
  }
  additional
}