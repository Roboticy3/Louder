//! A readable text format for conversations.
//!
//! This is the other loader alongside [`super::json_loader`]: a document written
//! in the line-oriented format described in the issue is turned into the same
//! JSON-shaped record list the JSON loader already understands, so that all of
//! the hub, connection, and link-hub machinery is reused rather than duplicated.

use super::context::LouderContext;
use super::json_loader;
use super::name::Map;
use serde_json::{Map as JsonMap, Value};

/// Load a conversation from a document in the readable text format.
pub fn load_conversation(
  context: &mut LouderContext,
  conversation: super::name::LouderName,
  document: &str,
) -> json_loader::LoadError {
  match parse_to_json(context, conversation, document) {
    Ok(json) => json_loader::load_conversation(context, conversation, &json),
    Err(()) => {
      report(context, conversation, -1, json_loader::LOUDER_LOAD_MALFORMED_JSON);
      json_loader::LOUDER_LOAD_MALFORMED_JSON
    }
  }
}

fn report(context: &mut LouderContext, conversation: super::name::LouderName, position: i32, error: json_loader::LoadError) {
  use super::event::{EventData, LoadFailure, LouderEvent};
  use super::rule::LouderRule;
  context.push_event(LouderEvent {
    conversation,
    channel: super::name::LOUDER_NAME_NONE,
    actor: super::actor::LOUDER_ACTOR_NONE,
    rule: LouderRule::PARSE_ERROR,
    data: EventData::LoadError(LoadFailure {
      position,
      error,
    }),
  });
}

#[derive(Clone, Default)]
struct Draft {
  position: i32,
  speaker: Option<String>,
  text: Option<String>,
  tags_raw: Option<String>,
  recipients_raw: Option<String>,
  channel: Option<String>,
  priority: Option<i32>,
  expressions_raw: Option<String>,
  callbacks_raw: Option<String>,
  flags: Option<i32>,
  link_tag: Option<String>,
  link_position: Option<i32>,
  additional: Map<String, String>,
  choice_tags: Vec<String>,
  has_choices_property: bool,
}

fn transform(value: &str) -> String {
  value.trim().to_string()
}

fn is_key(text: &str) -> bool {
  !text.is_empty()
    && text
      .chars()
      .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
}

fn split_chunks(document: &str) -> Vec<Vec<String>> {
  let mut chunks: Vec<Vec<String>> = Vec::new();
  let mut current: Vec<String> = Vec::new();
  for line in document.lines() {
    if line.trim().is_empty() {
      if !current.is_empty() {
        chunks.push(std::mem::take(&mut current));
      }
    } else {
      current.push(line.to_string());
    }
  }
  if !current.is_empty() {
    chunks.push(current);
  }
  chunks
}

fn parse_to_json(context: &mut LouderContext, conversation: super::name::LouderName, document: &str) -> Result<String, ()> {
  // Drop full-line comments before chunking.
  let filtered: String = document
    .lines()
    .filter(|line| !line.trim_start().starts_with('%'))
    .collect::<Vec<_>>()
    .join("\n");

  let chunks = split_chunks(&filtered);
  if chunks.is_empty() {
    return Ok("[]".to_string());
  }

  let mut drafts: Vec<Draft> = Vec::new();
  for (index, chunk) in chunks.iter().enumerate() {
    let mut draft = Draft {
      position: index as i32,
      ..Draft::default()
    };
    let mut unlabeled: Vec<String> = Vec::new();
    let mut props: Map<String, String> = Map::new();

    for raw in chunk {
      let line = raw.trim();
      if line.is_empty() {
        continue;
      }
      if line.starts_with('>') {
        for tag in super::name::split_inline(&line[1..]) {
          draft.choice_tags.push(transform(tag));
        }
        continue;
      }

      if let Some(colon) = line.find(':') {
        let key_raw = line[..colon].trim();
        let value_raw = line[colon + 1..].trim();
        if is_key(key_raw) {
          let key = transform(key_raw);
          let value = transform(value_raw);
          if let Some(existing) = props.get_mut(&key) {
            existing.push('\n');
            existing.push_str(&value);
          } else {
            props.insert(key, value);
          }
          continue;
        }
      }

      unlabeled.push(line.to_string());
    }

    // Unlabeled blocks: one means text, two or more means speaker + text.
    if unlabeled.len() == 1 {
      draft.text = Some(unlabeled[0].clone());
    } else if unlabeled.len() >= 2 {
      draft.speaker = Some(unlabeled[0].clone());
      draft.text = Some(unlabeled[1..].join("\n"));
    }

    for (key, value) in props {
      match key.as_str() {
        "speaker" => draft.speaker = Some(value),
        "text" => draft.text = Some(value),
        "tags" => draft.tags_raw = Some(value),
        "recipients" => draft.recipients_raw = Some(value.replace(',', "\n")),
        "channel" => draft.channel = Some(value),
        "priority" => {
          if let Ok(number) = value.trim().parse::<i32>() {
            draft.priority = Some(number);
          }
        }
        "expressions" => draft.expressions_raw = Some(value.replace(',', "\n")),
        "callbacks" => draft.callbacks_raw = Some(value.replace(',', "\n")),
        "flags" => {
          if let Ok(number) = value.trim().parse::<i32>() {
            draft.flags = Some(number);
          }
        }
        "link" => draft.link_tag = Some(value),
        "choices" => {
          draft.has_choices_property = true;
          for tag in super::name::split_inline(&value) {
            draft.choice_tags.push(tag.to_string());
          }
        }
        _ => {
          draft.additional.insert(key, value);
        }
      }
    }

    drafts.push(draft);
  }

  // Resolve tags to positions.
  let mut tag_positions: Map<String, Vec<i32>> = Map::new();
  for draft in &drafts {
    if let Some(raw) = &draft.tags_raw {
      for tag in super::name::split_inline(raw) {
        tag_positions
          .entry(transform(tag))
          .or_default()
          .push(draft.position);
      }
    }
  }

  // Resolve links.
  for draft in &mut drafts {
    if let Some(tag) = draft.link_tag.clone() {
      if let Some(positions) = tag_positions.get(&tag) {
        if let Some(first) = positions.first() {
          draft.link_position = Some(*first);
          let flags = draft.flags.get_or_insert(0);
          *flags |= 2; // LINK
        }
      } else if let Ok(position) = tag.parse::<i32>() {
        draft.link_position = Some(position);
        let flags = draft.flags.get_or_insert(0);
        *flags |= 2;
      } else {
        report(context, conversation, draft.position, json_loader::LOUDER_LOAD_UNRESOLVED_TAG);
      }
    }
  }

  // Build connections.
  let mut connections: Vec<(i32, i32, i32)> = Vec::new(); // from, port counter handled later
  // We'll collect per-from targets.
  let mut from_targets: Map<i32, Vec<i32>> = Map::new();

  for draft in &drafts {
    if !draft.choice_tags.is_empty() {
      for tag in &draft.choice_tags {
        if let Some(positions) = tag_positions.get(tag) {
          for target in positions {
            from_targets
              .entry(draft.position)
              .or_default()
              .push(*target);
          }
        } else {
          report(context, conversation, draft.position, json_loader::LOUDER_LOAD_UNRESOLVED_TAG);
        }
      }
    } else if !draft.has_choices_property {
      // Default: next hub in the document.
      let next = draft.position + 1;
      if drafts.iter().any(|other| other.position == next) {
        from_targets
          .entry(draft.position)
          .or_default()
          .push(next);
      }
    }
  }

  for (from, targets) in from_targets {
    for (port, to) in targets.into_iter().enumerate() {
      connections.push((from, port as i32, to));
    }
  }

  // Build JSON records.
  let mut records: Vec<Value> = Vec::new();
  for draft in &drafts {
    let mut object = JsonMap::new();
    object.insert("type".to_string(), Value::String("hub".to_string()));
    object.insert(
      "conversation_id".to_string(),
      Value::Number(serde_json::Number::from(draft.position)),
    );

    if let Some(speaker) = &draft.speaker {
      object.insert("speaker".to_string(), Value::String(speaker.clone()));
    }
    if let Some(text) = &draft.text {
      object.insert("text".to_string(), Value::String(text.clone()));
    }
    if let Some(raw) = &draft.tags_raw {
      object.insert("tags".to_string(), Value::String(raw.clone()));
    }
    if let Some(raw) = &draft.recipients_raw {
      object.insert("recipients".to_string(), Value::String(raw.clone()));
    }
    if let Some(channel) = &draft.channel {
      object.insert("channel".to_string(), Value::String(channel.clone()));
    }
    if let Some(priority) = draft.priority {
      object.insert(
        "priority".to_string(),
        Value::Number(serde_json::Number::from(priority)),
      );
    }
    if let Some(raw) = &draft.expressions_raw {
      object.insert("expressions".to_string(), Value::String(raw.clone()));
    }
    if let Some(raw) = &draft.callbacks_raw {
      object.insert("callbacks".to_string(), Value::String(raw.clone()));
    }
    if let Some(flags) = draft.flags {
      object.insert(
        "flags".to_string(),
        Value::Number(serde_json::Number::from(flags)),
      );
    }
    if let Some(link) = draft.link_position {
      object.insert(
        "link".to_string(),
        Value::Number(serde_json::Number::from(link)),
      );
    }

    // Additional data values: try number/bool, else string.
    for (key, value) in &draft.additional {
      let json_value = if value == "true" {
        Value::Bool(true)
      } else if value == "false" {
        Value::Bool(false)
      } else if let Ok(number) = value.parse::<i64>() {
        Value::Number(serde_json::Number::from(number))
      } else if let Ok(number) = value.parse::<f64>() {
        serde_json::Number::from_f64(number)
          .map(Value::Number)
          .unwrap_or_else(|| Value::String(value.clone()))
      } else {
        Value::String(value.clone())
      };
      object.insert(key.clone(), json_value);
    }

    records.push(Value::Object(object));
  }

  for (from, _port, to) in &connections {
    // Port assigned below; placeholder replaced.
    let _ = (from, to);
  }

  // Assign ports per source.
  let mut port_counter: Map<i32, i32> = Map::new();
  for (from, to) in connections
    .iter()
    .map(|(from, _port, to)| (*from, *to))
    .collect::<Vec<_>>()
  {
    let port = port_counter.entry(from).or_insert(0);
    let current = *port;
    *port += 1;

    let mut object = JsonMap::new();
    object.insert("type".to_string(), Value::String("connection".to_string()));
    object.insert(
      "from_id".to_string(),
      Value::Number(serde_json::Number::from(from)),
    );
    object.insert(
      "from_port".to_string(),
      Value::Number(serde_json::Number::from(current)),
    );
    object.insert("to_id".to_string(), Value::Number(serde_json::Number::from(to)));
    records.push(Value::Object(object));
  }

  let serialized = serde_json::to_string(&records).map_err(|_| ())?;
  Ok(serialized)
}

#[cfg(test)]
mod tmp_repro {
  use super::*;
  #[test]
  fn repro() {
    let mut ctx = LouderContext::new();
    let conv = 0;
    let doc = "tags: a\r\na\r\n\r\ntags: e\r\nspeaker\r\ntext\r\n> a\r\n";
    let err = load_conversation(&mut ctx, conv, doc);
    println!("ERR: {}", err);
    println!("EVENTS: {}", ctx.event_count());
  }
}
