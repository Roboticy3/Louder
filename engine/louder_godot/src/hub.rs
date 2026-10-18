//! Hubs, and the choices leading out of them.

use crate::object::HomogeneousObject;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// One link out of a hub, to another hub.
///
/// The schema gives choices no text of their own; [`Choice::get_text`] is the
/// text of the hub it links to, so that a choice can be described to the player
/// using what they are about to walk into.
#[derive(Clone)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Choice {
  pub(crate) inner: engine::Choice,
}

godot_api! {
  impl Choice {
    /// A choice leading to `position`, described by the text of that hub.
    #[func]
    pub fn create(
      text: GString,
      text_name: engine::LouderName,
      position: engine::LouderPosition,
    ) -> Gd<Choice> {
      Gd::from_object(Choice {
        inner: engine::Choice::new(text.to_string(), text_name, position),
      })
    }

    /// The text of the hub this choice leads to.
    #[func]
    pub fn get_text(&self) -> GString {
      GString::from(self.inner.text())
    }

    /// [`Choice::get_text`] interned as a name.
    #[func]
    pub fn get_text_name(&self) -> engine::LouderName {
      self.inner.text_name()
    }

    /// The position this choice leads to.
    #[func]
    pub fn get_position(&self) -> engine::LouderPosition {
      self.inner.position()
    }

    /// Link somewhere else.
    #[func]
    pub fn set_position(&mut self, position: engine::LouderPosition) {
      self.inner.set_position(position);
    }
  }
}

/// A single point in a conversation, with everything an actor needs to decide
/// whether it applies to them.
///
/// A snapshot of what the engine held, so writing to one of these changes the
/// copy rather than the conversation. To change what the engine holds, load a
/// document through [`LouderContext`](crate::context::LouderContext).
#[derive(Clone)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Hub {
  pub(crate) inner: engine::Hub,
}

godot_api! {
  impl Hub {
    /// A hub standing at `position`.
    ///
    /// `expressions_joined` and `callbacks_joined` are the whole newline
    /// separated blocks, which is how the engine hands them back out; the lists
    /// beside them are the same expressions one entry at a time.
    #[func]
    #[allow(clippy::too_many_arguments)]
    pub fn create(
      position: engine::LouderPosition,
      speaker: engine::LouderName,
      text: GString,
      text_name: engine::LouderName,
      tags: Array<i64>,
      recipients: Array<i64>,
      channel: engine::LouderName,
      priority: i32,
      expressions: Array<i64>,
      expressions_joined: engine::LouderExpression,
      callbacks: Array<i64>,
      callbacks_joined: engine::LouderCallback,
      flags: i32,
      additional: Gd<HomogeneousObject>,
    ) -> Gd<Hub> {
      Gd::from_object(Hub {
        inner: engine::Hub::new(
          position,
          speaker,
          text.to_string(),
          text_name,
          tags.iter_shared().map(|name| name as engine::LouderName).collect(),
          recipients.iter_shared().map(|name| name as engine::LouderName).collect(),
          channel,
          priority,
          expressions.iter_shared().map(|name| name as engine::LouderExpression).collect(),
          expressions_joined,
          callbacks.iter_shared().map(|name| name as engine::LouderCallback).collect(),
          callbacks_joined,
          engine::HubFlags::from_bits_truncate(flags),
          additional.bind().inner.clone(),
        ),
      })
    }

    /// Where this hub stands in the network.
    #[func]
    pub fn get_position(&self) -> engine::LouderPosition {
      self.inner.position()
    }

    /// The speaker who says this hub, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_speaker(&self) -> engine::LouderName {
      self.inner.speaker()
    }

    /// Speak this hub as somebody else.
    #[func]
    pub fn set_speaker(&mut self, speaker: engine::LouderName) {
      self.inner.set_speaker(speaker);
    }

    /// What this hub says.
    #[func]
    pub fn get_text(&self) -> GString {
      GString::from(self.inner.text())
    }

    /// Say something else. The interned `text_name` is left alone, since
    /// assigning it needs the name table.
    #[func]
    pub fn set_text(&mut self, text: GString) {
      self.inner.set_text(text.to_string());
    }

    /// [`Hub::get_text`] interned as a name.
    #[func]
    pub fn get_text_name(&self) -> engine::LouderName {
      self.inner.text_name()
    }

    /// The tags on this hub.
    #[func]
    pub fn tags(&self) -> Array<i64> {
      self.inner.tags().iter().map(|name| *name as i64).collect()
    }

    /// The speakers this hub is restricted to.
    #[func]
    pub fn recipients(&self) -> Array<i64> {
      self.inner.recipients().iter().map(|name| *name as i64).collect()
    }

    /// Whether this hub is restricted to anybody at all.
    #[func]
    pub fn has_recipients(&self) -> bool {
      self.inner.has_recipients()
    }

    /// The channel this hub moves an actor into.
    #[func]
    pub fn get_channel(&self) -> engine::LouderName {
      self.inner.channel()
    }

    /// This hub's priority.
    #[func]
    pub fn get_priority(&self) -> i32 {
      self.inner.priority()
    }

    /// Rank this hub at `priority`.
    #[func]
    pub fn set_priority(&mut self, priority: i32) {
      self.inner.set_priority(priority);
    }

    /// The flag bits, as [`HubFlags::bits`](engine::HubFlags).
    #[func]
    pub fn get_flags(&self) -> i32 {
      self.inner.flags().bits()
    }

    /// Whether this hub resolves on its own instead of being shown.
    #[func]
    pub fn is_auto(&self) -> bool {
      self.inner.is_auto()
    }

    /// How many choices leave this hub.
    #[func]
    pub fn get_choice_count(&self) -> i32 {
      self.inner.choice_count() as i32
    }

    /// The choice at `index`, if there is one.
    #[func]
    pub fn get_choice(&self, index: i32) -> Option<Gd<Choice>> {
      if index < 0 {
        return None;
      }
      self
        .inner
        .choice(index as usize)
        .map(|choice| Gd::from_object(Choice { inner: choice.clone() }))
    }

    /// Every choice leaving this hub.
    #[func]
    pub fn choices(&self) -> Array<Gd<Choice>> {
      self
        .inner
        .choices()
        .iter()
        .map(|choice| Gd::from_object(Choice { inner: choice.clone() }))
        .collect()
    }

    /// Add a choice, indexed by the order choices were added.
    #[func]
    pub fn push_choice(&mut self, choice: Gd<Choice>) {
      self.inner.push_choice(choice.bind().inner.clone());
    }

    /// The conditions and statements on this hub, one per entry.
    #[func]
    pub fn expressions(&self) -> Array<i64> {
      self.inner.expressions().iter().map(|name| *name as i64).collect()
    }

    /// Every expression joined into one newline separated block.
    #[func]
    pub fn get_expressions_joined(&self) -> engine::LouderExpression {
      self.inner.expressions_joined()
    }

    /// The callbacks on this hub, one per entry.
    #[func]
    pub fn callbacks(&self) -> Array<i64> {
      self.inner.callbacks().iter().map(|name| *name as i64).collect()
    }

    /// Every callback joined into one newline separated block.
    #[func]
    pub fn get_callbacks_joined(&self) -> engine::LouderCallback {
      self.inner.callbacks_joined()
    }

    /// Every key of this hub's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.inner.additional_data().sorted_keys().into_iter().map(|key| key as i64).collect()
    }

    /// The additional value under `key`, read as an integer, or 0.
    #[func]
    pub fn get_additional_int(&self, key: engine::LouderName) -> i32 {
      self.inner.additional_get(key).map(engine::LouderVariant::as_int).unwrap_or(0)
    }

    /// The additional value under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_additional_float(&self, key: engine::LouderName) -> f64 {
      self.inner.additional_get(key).map(engine::LouderVariant::as_float).unwrap_or(0.0) as f64
    }

    /// The additional value under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_additional_name(&self, key: engine::LouderName) -> engine::LouderName {
      match self.inner.additional_get(key) {
        Some(engine::LouderVariant::String(name)) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }
  }
}