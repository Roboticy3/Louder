//! Writing what the engine holds back out as JSON.
//!
//! This is the other half of [`super::json_loader`]: documents come in through
//! that module and leave through this one, in the shape a save file or a
//! debugging dump wants. Nothing here parses. Every document is assembled from
//! the context as it stands, and both are written in a fixed order so that
//! asking twice gives the same bytes.

use super::context::LouderContext;
use super::event::{EventData, LouderAsk, LouderEvent, Motion};
use super::name::{LouderInternContext, LouderName, LOUDER_NAME_NONE};
use super::rule::get_rule_string;
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

/// Render a name as a JSON string, or null when it names nothing.
fn name_value(context: &LouderContext, name: LouderName) -> String {
  match context.resolve_name(name) {
    Some(text) => quote(text),
    None => "null".to_string(),
  }
}

/// Render a motion as a JSON object.
fn motion(context: &LouderContext, motion: &Motion) -> String {
  format!(
    "{{\"source_position\":{},\"target_position\":{},\"choice_index\":{},\"speaker\":{},\"background\":{},\"rule\":\"{}\"}}",
    motion.source_position,
    motion.target_position,
    motion.choice_index,
    name_value(context, motion.speaker),
    motion.background,
    get_rule_string(motion.rule)
  )
}

/// Render one traced event as a JSON object.
fn event(context: &LouderContext, event: &LouderEvent) -> String {
  let mut out = String::from("{");
  out.push_str(&format!(
    "\"conversation\":{},\"channel\":{},\"actor\":{},\"rule\":\"{}\",\"kind\":{}",
    name_value(context, event.conversation),
    name_value(context, event.channel),
    event.actor,
    get_rule_string(event.rule),
    event.data.kind()
  ));

  if event.data.name() != LOUDER_NAME_NONE {
    out.push_str(&format!(",\"name\":{}", name_value(context, event.data.name())));
  }

  match &event.data {
    EventData::RegularMotion(entry) | EventData::Warp(entry) | EventData::ForceMotion(entry) => {
      out.push_str(&format!(",\"motion\":{}", motion(context, entry)));
    }
    EventData::ChoiceAppearance(appearance) => {
      out.push_str(&format!(
        ",\"choice_appearance\":{{\"position\":{},\"choice_index\":{},\"rule\":\"{}\"}}",
        appearance.position, appearance.choice_index, get_rule_string(appearance.rule)
      ));
    }
    EventData::AdditionalDataChange(change) => {
      out.push_str(&format!(",\"additional_data_key\":{},\"additional_data_rule\":\"{}\"", name_value(context, change.key), get_rule_string(change.rule)));
    }
    EventData::Condition(outcome) => {
      out.push_str(&format!(
        ",\"condition\":{{\"condition\":{},\"outcome\":{}}}",
        name_value(context, outcome.condition),
        variant(context, &outcome.outcome)
      ));
    }
    EventData::LoadError(failure) => {
      out.push_str(&format!(
        ",\"load_failure\":{{\"position\":{},\"error\":{}}}",
        failure.position, failure.error
      ));
    }
    _ => {}
  }

  out.push('}');
  out
}

/// Render one traced ask as a JSON object.
fn ask(context: &LouderContext, ask: &LouderAsk) -> String {
  format!(
    "{{\"conversation\":{},\"channel\":{},\"actor\":{},\"rule\":\"{}\",\"motion\":{},\"ask\":{}}}",
    name_value(context, ask.conversation),
    name_value(context, ask.channel),
    ask.actor,
    get_rule_string(ask.rule),
    motion(context, &ask.motion),
    name_value(context, ask.ask)
  )
}

/// Every event and ask the engine has created, as one JSON document.
///
/// The event queue and ask cache are works in progress that a host drains as it
/// handles them; this is the complete history instead, so flushing either one
/// leaves this intact. `events` and `asks` are both arrays in creation order.
pub fn trace(context: &LouderContext) -> String {
  let mut events = String::from("[");
  for (index, entry) in context.traced_events().iter().enumerate() {
    if index > 0 {
      events.push(',');
    }
    events.push_str(&event(context, entry));
  }
  events.push(']');

  let mut asks = String::from("[");
  for (index, entry) in context.traced_asks().iter().enumerate() {
    if index > 0 {
      asks.push(',');
    }
    asks.push_str(&ask(context, entry));
  }
  asks.push(']');

  format!("{{\"events\":{},\"asks\":{}}}", events, asks)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn trace_serializes_events_and_asks() {
    let mut context = LouderContext::new();
    let conversation = context.intern_name("conversation");
    let callback = context.intern_name("callback");

    context.push_event(LouderEvent {
      conversation,
      channel: LOUDER_NAME_NONE,
      actor: 4,
      rule: super::super::rule::LouderRule::NONE,
      data: EventData::ActorAdded,
    });
    context.push_ask(LouderAsk {
      conversation,
      channel: LOUDER_NAME_NONE,
      actor: 4,
      rule: super::super::rule::LouderRule::NONE,
      motion: Motion::default(),
      ask: callback,
    });

    let document = trace(&context);
    let parsed: serde_json::Value = serde_json::from_str(&document).expect("the trace should be JSON");

    let events = parsed["events"].as_array().expect("events should be an array");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["conversation"], "conversation");
    assert_eq!(events[0]["actor"], 4);
    assert_eq!(events[0]["kind"], 12);

    let asks = parsed["asks"].as_array().expect("asks should be an array");
    assert_eq!(asks.len(), 1);
    assert_eq!(asks[0]["ask"], "callback");
  }
}

