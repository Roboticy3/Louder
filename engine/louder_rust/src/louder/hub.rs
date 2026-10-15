#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, GString, Gd};
use super::name::LouderName;
use super::object::HomogeneousObject;
use super::expression::LouderExpression;
use super::callback::LouderCallback;

bitflags::bitflags! {
  /// Flag bits describing a hub.
  #[repr(C)]
  #[derive(Clone, Copy, Default)]
  pub struct HubFlags: i32 {
    /// The hub picks its own path instead of showing to actors, resolving to
    /// the first choice available to whoever entered it.
    const AUTO = 0b00000001;
    /// The hub's expressions are expected to raise asks.
    const ASK = 0b00000010;
  }
}

/// A position within a conversation network of hubs.
///
/// Positions are chosen by the editor and are sparse: a conversation may hold
/// 65, 68, 71, and 75 with nothing in between. Iteration over positions
/// therefore walks hubs in ascending position order rather than counting.
pub type LouderPosition = i32;

/// The blank starting value for an iterator over positions.
///
/// Positions are chosen by the editor, and the engine requires them to be
/// non-negative so that this value is unambiguous.
pub(crate) const BLANK_POSITION: LouderPosition = -1;

/// The text content of a hub.
pub type LouderText = String;

godot_class! {
  /// One link out of a hub, to another hub.
  ///
  /// The schema gives choices no text of their own; [`Choice::text`] is the
  /// text of the hub it links to, so that a choice can be described to the
  /// player using what they are about to walk into.
  #[derive(Clone)]
  pub struct Choice {
    text: LouderText,
    text_name: LouderName,
    position: LouderPosition,
  }
}

impl Choice {
  pub fn new(text: LouderText, text_name: LouderName, position: LouderPosition) -> Choice {
    Choice {
      text,
      text_name,
      position,
    }
  }

  /// The text of the hub this choice leads to.
  pub fn text(&self) -> &str {
    &self.text
  }

  /// [`Self::text`] interned as a name, for handing to the FFI.
  pub fn text_name(&self) -> LouderName {
    self.text_name
  }

  /// The position this choice links to.
  pub fn position(&self) -> LouderPosition {
    self.position
  }
}

godot_api! {
  impl Choice {
    /// A choice leading to `position`, described by the text of that hub.
    #[func]
    pub fn create(text: GString, text_name: LouderName, position: LouderPosition) -> Gd<Choice> {
      Gd::from_object(Choice::new(text.to_string(), text_name, position))
    }

    /// The text of the hub this choice leads to.
    #[func]
    pub fn get_text(&self) -> GString {
      GString::from(self.text())
    }

    /// [`Choice::text`] interned as a name.
    #[func]
    pub fn get_text_name(&self) -> LouderName {
      self.text_name()
    }

    /// The position this choice links to.
    #[func]
    pub fn get_position(&self) -> LouderPosition {
      self.position()
    }

    /// Link somewhere else.
    #[func(rename = "set_position")]
    pub fn gd_set_position(&mut self, position: LouderPosition) {
      self.position = position;
    }
  }
}

godot_class! {
  /// A single point in a conversation, with everything an actor needs to decide
  /// whether it applies to them.
  ///
  /// The hub a conversation holds is handed to GDScript as a copy; to change
  /// what the engine holds, load a document through
  /// [`LouderContext`](super::context::LouderContext) rather than writing to the
  /// copy.
  #[derive(Clone)]
  pub struct Hub {
    position: LouderPosition,

    speaker: LouderName,
    tags: Vec<LouderName>,
    recipients: Vec<LouderName>,

    channel: LouderName,
    priority: i32,

    text: LouderText,
    text_name: LouderName,
    choices: Vec<Choice>,

    expressions: Vec<LouderExpression>,
    expressions_joined: LouderExpression,
    callbacks: Vec<LouderCallback>,
    callbacks_joined: LouderCallback,
    flags: HubFlags,

    additional_data: HomogeneousObject,
  }
}

impl Hub {
  #[allow(clippy::too_many_arguments)]
  pub fn new(
    position: LouderPosition,
    speaker: LouderName,
    text: LouderText,
    text_name: LouderName,
    tags: Vec<LouderName>,
    recipients: Vec<LouderName>,
    channel: LouderName,
    priority: i32,
    expressions: Vec<LouderExpression>,
    expressions_joined: LouderExpression,
    callbacks: Vec<LouderCallback>,
    callbacks_joined: LouderCallback,
    flags: HubFlags,
    additional_data: HomogeneousObject,
  ) -> Hub {
    Hub {
      position,
      speaker,
      tags,
      recipients,
      channel,
      priority,
      text,
      text_name,
      choices: Vec::new(),
      expressions,
      expressions_joined,
      callbacks,
      callbacks_joined,
      flags,
      additional_data,
    }
  }

  pub fn position(&self) -> LouderPosition {
    self.position
  }

  /// The speaker who "says" this hub, or [`super::name::LOUDER_NAME_NONE`]
  /// when the hub has no speaker and is therefore public to everyone.
  pub fn speaker(&self) -> LouderName {
    self.speaker
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  /// [`Self::text`] interned as a name, for handing to the FFI.
  pub fn text_name(&self) -> LouderName {
    self.text_name
  }

  pub fn tags(&self) -> &[LouderName] {
    &self.tags
  }

  pub fn choices(&self) -> &[Choice] {
    &self.choices
  }

  pub fn choice(&self, index: usize) -> Option<&Choice> {
    self.choices.get(index)
  }

  /// Append a choice.
  ///
  /// Choices are indexed by the order they were added, which is the port order
  /// of the source document.
  pub fn push_choice(&mut self, choice: Choice) {
    self.choices.push(choice);
  }

  pub fn choice_count(&self) -> usize {
    self.choices.len()
  }

  /// Conditions and statements attached to this hub, one name per line.
  pub fn expressions(&self) -> &[LouderExpression] {
    &self.expressions
  }

  /// Every expression joined back into one newline separated block,
  /// interned as a single name.
  pub fn expressions_joined(&self) -> LouderExpression {
    self.expressions_joined
  }

  /// Callbacks attached to this hub, one name per line.
  pub fn callbacks(&self) -> &[LouderCallback] {
    &self.callbacks
  }

  /// Every callback joined back into one newline separated block, interned as
  /// a single name.
  pub fn callbacks_joined(&self) -> LouderCallback {
    self.callbacks_joined
  }

  pub fn flags(&self) -> HubFlags {
    self.flags
  }

  pub fn is_auto(&self) -> bool {
    self.flags.contains(HubFlags::AUTO)
  }

  /// Speakers this hub is restricted to.
  ///
  /// When non-empty these override the normal visibility rules, so the hub
  /// only appears to actors speaking as or listening to one of them.
  pub fn recipients(&self) -> &[LouderName] {
    &self.recipients
  }

  pub fn has_recipients(&self) -> bool {
    !self.recipients.is_empty()
  }

  /// The channel this hub moves an actor into, or
  /// [`super::name::LOUDER_NAME_NONE`] to leave the actor where it is.
  pub fn channel(&self) -> LouderName {
    self.channel
  }

  pub fn priority(&self) -> i32 {
    self.priority
  }

  pub fn additional_data(&self) -> &HomogeneousObject {
    &self.additional_data
  }

  pub fn additional_get(&self, key: LouderName) -> Option<&super::object::LouderVariant> {
    self.additional_data.get(key)
  }
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
      position: LouderPosition,
      speaker: LouderName,
      text: GString,
      text_name: LouderName,
      tags: Array<i64>,
      recipients: Array<i64>,
      channel: LouderName,
      priority: i32,
      expressions: Array<i64>,
      expressions_joined: LouderExpression,
      callbacks: Array<i64>,
      callbacks_joined: LouderCallback,
      flags: i32,
      additional: Gd<HomogeneousObject>,
    ) -> Gd<Hub> {
      Gd::from_object(Hub::new(
        position,
        speaker,
        text.to_string(),
        text_name,
        tags.iter_shared().map(|name| name as LouderName).collect(),
        recipients.iter_shared().map(|name| name as LouderName).collect(),
        channel,
        priority,
        expressions.iter_shared().map(|name| name as LouderExpression).collect(),
        expressions_joined,
        callbacks.iter_shared().map(|name| name as LouderCallback).collect(),
        callbacks_joined,
        HubFlags::from_bits_truncate(flags),
        additional.bind().clone(),
      ))
    }

    /// Where this hub stands in the network.
    #[func]
    pub fn get_position(&self) -> LouderPosition {
      self.position()
    }

    /// The speaker who says this hub, or
    /// [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE).
    #[func]
    pub fn get_speaker(&self) -> LouderName {
      self.speaker()
    }

    /// Speak this hub as somebody else.
    #[func(rename = "set_speaker")]
    pub fn gd_set_speaker(&mut self, speaker: LouderName) {
      self.speaker = speaker;
    }

    /// What this hub says.
    #[func]
    pub fn get_text(&self) -> GString {
      GString::from(self.text())
    }

    /// Say something else. The interned `text_name` is left alone, since
    /// assigning it needs the name table.
    #[func(rename = "set_text")]
    pub fn gd_set_text(&mut self, text: GString) {
      self.text = text.to_string();
    }

    /// [`Hub::text`] interned as a name.
    #[func]
    pub fn get_text_name(&self) -> LouderName {
      self.text_name()
    }

    /// The tags on this hub.
    #[func(rename = "tags")]
    pub fn gd_tags(&self) -> Array<i64> {
      self.tags().iter().map(|name| *name as i64).collect()
    }

    /// The speakers this hub is restricted to.
    #[func(rename = "recipients")]
    pub fn gd_recipients(&self) -> Array<i64> {
      self.recipients().iter().map(|name| *name as i64).collect()
    }

    /// Whether this hub is restricted to anybody at all.
    #[func(rename = "has_recipients")]
    pub fn gd_has_recipients(&self) -> bool {
      Hub::has_recipients(self)
    }

    /// The channel this hub moves an actor into.
    #[func]
    pub fn get_channel(&self) -> LouderName {
      self.channel()
    }

    /// This hub's priority.
    #[func]
    pub fn get_priority(&self) -> i32 {
      self.priority()
    }

    /// Rank this hub at `priority`.
    #[func(rename = "set_priority")]
    pub fn gd_set_priority(&mut self, priority: i32) {
      self.priority = priority;
    }

    /// The flag bits, as [`HubFlags::bits`].
    #[func]
    pub fn get_flags(&self) -> i32 {
      self.flags().bits()
    }

    /// Whether this hub resolves on its own instead of being shown.
    #[func(rename = "is_auto")]
    pub fn gd_is_auto(&self) -> bool {
      Hub::is_auto(self)
    }

    /// How many choices leave this hub.
    #[func]
    pub fn get_choice_count(&self) -> i32 {
      self.choice_count() as i32
    }

    /// The choice at `index`, if there is one.
    #[func]
    pub fn get_choice(&self, index: i32) -> Option<Gd<Choice>> {
      if index < 0 {
        return None;
      }
      self.choice(index as usize).map(|choice| Gd::from_object(choice.clone()))
    }

    /// Every choice leaving this hub.
    #[func(rename = "choices")]
    pub fn gd_choices(&self) -> Array<Gd<Choice>> {
      self.choices().iter().map(|choice| Gd::from_object(choice.clone())).collect()
    }

    /// Add a choice, indexed by the order choices were added.
    #[func(rename = "push_choice")]
    pub fn gd_push_choice(&mut self, choice: Gd<Choice>) {
      Hub::push_choice(self, choice.bind().clone());
    }

    /// The conditions and statements on this hub, one per entry.
    #[func(rename = "expressions")]
    pub fn gd_expressions(&self) -> Array<i64> {
      self.expressions().iter().map(|name| *name as i64).collect()
    }

    /// Every expression joined into one newline separated block.
    #[func]
    pub fn get_expressions_joined(&self) -> LouderExpression {
      self.expressions_joined()
    }

    /// The callbacks on this hub, one per entry.
    #[func(rename = "callbacks")]
    pub fn gd_callbacks(&self) -> Array<i64> {
      self.callbacks().iter().map(|name| *name as i64).collect()
    }

    /// Every callback joined into one newline separated block.
    #[func]
    pub fn get_callbacks_joined(&self) -> LouderCallback {
      self.callbacks_joined()
    }

    /// Every key of this hub's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.additional_data().sorted_keys().into_iter().map(|key| key as i64).collect()
    }

    /// The additional value under `key`, read as an integer, or 0.
    #[func]
    pub fn get_additional_int(&self, key: LouderName) -> i32 {
      self.additional_get(key).map(super::object::LouderVariant::as_int).unwrap_or(0)
    }

    /// The additional value under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_additional_float(&self, key: LouderName) -> f64 {
      self.additional_get(key).map(super::object::LouderVariant::as_float).unwrap_or(0.0) as f64
    }

    /// The additional value under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE).
    #[func]
    pub fn get_additional_name(&self, key: LouderName) -> LouderName {
      match self.additional_get(key) {
        Some(super::object::LouderVariant::String(name)) => *name,
        _ => super::name::LOUDER_NAME_NONE,
      }
    }
  }
}