use super::name::{LouderName, LOUDER_NAME_NONE};
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

/// One link out of a hub, to another hub.
///
/// The schema gives choices no text of their own; [`Choice::text`] is the
/// text of the hub it links to, so that a choice can be described to the
/// player using what they are about to walk into.
#[derive(Clone, Default)]
pub struct Choice {
  text: LouderText,
  text_name: LouderName,
  position: LouderPosition,
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

  /// Link somewhere else.
  pub fn set_position(&mut self, position: LouderPosition) {
    self.position = position;
  }
}


/// A single point in a conversation, with everything an actor needs to decide
/// whether it applies to them.
///
/// The hub a conversation holds is handed out as a copy; to change what the
/// engine holds, load a document through
/// [`LouderContext`](super::context::LouderContext) rather than writing to the
/// copy.
#[derive(Clone, Default)]
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

  /// The speaker this hub speaks as, as an actor would hear it.
  ///
  /// A hub with a speaker of its own speaks as that speaker. A hub without one
  /// is anonymous: it speaks as the first voice in the speak as list of whoever
  /// is standing on it, which is the projection the engine guide describes.
  /// That is why the answer needs the actor as well as the hub.
  pub fn speaker_for(&self, speak_as: &[LouderName]) -> LouderName {
    if self.speaker != LOUDER_NAME_NONE {
      return self.speaker;
    }

    speak_as
      .iter()
      .copied()
      .find(|name| *name != LOUDER_NAME_NONE)
      .unwrap_or(LOUDER_NAME_NONE)
  }

  /// Speak this hub as somebody else.
  pub fn set_speaker(&mut self, speaker: LouderName) {
    self.speaker = speaker;
  }

  pub fn text(&self) -> &str {
    &self.text
  }

  /// Say something else. The interned [`Self::text_name`] is left alone, since
  /// assigning it needs the name table.
  pub fn set_text(&mut self, text: LouderText) {
    self.text = text;
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

  /// Rank this hub at `priority`.
  pub fn set_priority(&mut self, priority: i32) {
    self.priority = priority;
  }

  pub fn additional_data(&self) -> &HomogeneousObject {
    &self.additional_data
  }

  pub fn additional_get(&self, key: LouderName) -> Option<&super::object::LouderVariant> {
    self.additional_data.get(key)
  }
}

