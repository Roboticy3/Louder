#[cfg(feature = "godot-bindings")]
use super::godot_api::Gd;
use crate::louder::object::LouderVariant;

use super::callback::LouderCallback;
use super::expression::LouderExpression;

use super::actor::LouderActor;
use super::name::LouderName;
use super::hub::LouderPosition;

godot_class! {
  /// One move of an actor from one hub to another.
  ///
  /// Every event a move produces carries the same motion, which is how a host
  /// reads where an actor came from and where it went.
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct Motion {
    pub source_position: LouderPosition,
    pub target_position: LouderPosition,
    pub choice_index: i32,

    pub background: bool,
  }
}

godot_class! {
  /// A choice becoming available on a hub.
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct ChoiceAppearance {
    pub position: LouderPosition,
    pub choice_index: i32
  }
}

godot_class! {
  /// A key of an actor's additional data being replaced.
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct AdditionalDataChange {
    pub key: LouderName,
  }
}

godot_class! {
  /// What a condition on a hub came to, when an actor landed on it.
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct ConditionOutcome {
    pub condition: LouderName,
    pub outcome: LouderVariant,
  }
}

godot_class! {
  /// A record in a document the engine could not read.
  ///
  /// Carries the position of the record that was at fault, or
  /// [`LOUDER_POSITION_NONE`] when the problem was with the document as a whole.
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct LoadFailure {
    pub position: LouderPosition,
    pub error: i32,
  }
}

/// What an [`LouderEvent`] is about.
///
/// A tagged union has no Godot counterpart, so this is the discriminant of an
/// [`EventData`] as an `int`, and the payload is read through the accessors on
/// [`LouderEvent`] that match it. A host that does not recognise a value is
/// looking at something a newer engine produced.
#[derive(Debug, Clone)]
#[repr(C)]
pub enum EventData {
  None,
  RegularMotion(Motion),
  Warp(Motion),
  ForceMotion(Motion),
  ChoiceAppearance(ChoiceAppearance),

  ChannelChange,
  AdditionalDataChange(AdditionalDataChange),
  SpeakerChange(LouderName),
  TextChange(LouderName),

  Statement(LouderExpression),
  Condition(ConditionOutcome),
  Callback(LouderCallback),

  ActorAdded,
  ActorRemoved,
  SpeakerAdded(LouderName),
  SpeakerRemoved(LouderName),
  ConversationAdded(LouderName),
  ConversationRemoved(LouderName),

  /// A record of a loaded document was unusable. This is not a motion and is
  /// never attached to one, so a host can tell it apart without inspecting the
  /// fields of an event it did not expect.
  LoadError(LoadFailure),
}

impl EventData {
  /// Which variant this is, as an `int`.
  ///
  /// Written out rather than left to the compiler so that the numbering is part
  /// of the engine's vocabulary and not a property of the declaration order.
  pub fn kind(&self) -> i32 {
    match self {
      EventData::None => 0,
      EventData::RegularMotion(_) => 1,
      EventData::Warp(_) => 2,
      EventData::ForceMotion(_) => 3,
      EventData::ChoiceAppearance(_) => 4,

      EventData::ChannelChange => 5,
      EventData::AdditionalDataChange(_) => 6,
      EventData::SpeakerChange(_) => 7,
      EventData::TextChange(_) => 8,

      EventData::Statement(_) => 9,
      EventData::Condition(_) => 10,
      EventData::Callback(_) => 11,

      EventData::ActorAdded => 12,
      EventData::ActorRemoved => 13,
      EventData::SpeakerAdded(_) => 14,
      EventData::SpeakerRemoved(_) => 15,
      EventData::ConversationAdded(_) => 16,
      EventData::ConversationRemoved(_) => 17,

      EventData::LoadError(_) => 18,
    }
  }

  /// The motion this event carries, if it is about one.
  pub fn motion(&self) -> Option<&Motion> {
    match self {
      EventData::RegularMotion(motion) | EventData::Warp(motion) | EventData::ForceMotion(motion) => Some(motion),
      _ => None,
    }
  }

  /// The name this event carries, whatever kind of name that is.
  ///
  /// A speaker, a text, an expression, and a callback are all names, so one
  /// accessor reads all of them; which of those it was says `kind`.
  pub fn name(&self) -> LouderName {
    match self {
      EventData::SpeakerChange(name)
      | EventData::TextChange(name)
      | EventData::SpeakerAdded(name)
      | EventData::SpeakerRemoved(name)
      | EventData::ConversationAdded(name)
      | EventData::ConversationRemoved(name)
      | EventData::Statement(name)
      | EventData::Callback(name) => *name,
      _ => super::name::LOUDER_NAME_NONE,
    }
  }
}

impl Default for EventData {
  fn default() -> Self {
    return EventData::None
  }
}

godot_class! {
  /// One entry in the event queue.
  ///
  /// The queue is polled from index 0 upwards and emptied with
  /// [`LouderContext::flush_events`](super::context::LouderContext).
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct LouderEvent {
    pub conversation: LouderName,
    pub channel: LouderName,
    pub actor: LouderActor,
    pub data: EventData,
  }
}

godot_class! {
  /// A question the engine has asked the host and is waiting on.
  ///
  /// Answering an ask moves it to the back of the cache, so indexes shift; read
  /// the cache again after each answer.
  #[derive(Debug, Clone, Default)]
  #[repr(C)]
  pub struct LouderAsk {
    pub conversation: LouderName,
    pub channel: LouderName,
    pub actor: LouderActor,
    pub motion: Motion,
    pub ask: LouderCallback,
  }
}

/// Which position of an event's data to read.
///
/// Every position an event can refer to is carried inside a [`Motion`], apart
/// from the hub a choice appeared on, which is in a [`ChoiceAppearance`]. Match
/// on the event's [`EventData`] and read the field this names.
#[repr(C)]
pub enum EventPosition {
  /// Where the actor of this event's motion starts from.
  ///
  /// [`Motion::source_position`].
  Source = 0,
  /// Where the actor of this event's motion moves to.
  ///
  /// [`Motion::target_position`].
  Target = 1,
  /// Which hub this event's choice appeared on.
  ///
  /// [`ChoiceAppearance::position`].
  ChoiceAppearance = 2,
}

godot_api! {
  impl Motion {
    /// Where the actor came from.
    #[func]
    pub fn get_source_position(&self) -> LouderPosition {
      self.source_position
    }

    /// Where the actor went.
    #[func]
    pub fn get_target_position(&self) -> LouderPosition {
      self.target_position
    }

    /// The choice the actor picked, or -1 if it did not pick anything.
    #[func]
    pub fn get_choice_index(&self) -> i32 {
      self.choice_index
    }

    /// Whether the actor saw this hub without moving onto it.
    #[func]
    pub fn is_background(&self) -> bool {
      self.background
    }
  }
}

godot_api! {
  impl ChoiceAppearance {
    /// The hub the choice appeared on.
    #[func]
    pub fn get_position(&self) -> LouderPosition {
      self.position
    }

    /// Which choice appeared.
    #[func]
    pub fn get_choice_index(&self) -> i32 {
      self.choice_index
    }
  }
}

godot_api! {
  impl AdditionalDataChange {
    /// The key whose value was replaced.
    #[func]
    pub fn get_key(&self) -> LouderName {
      self.key
    }
  }
}

godot_api! {
  impl ConditionOutcome {
    /// The condition that was evaluated.
    #[func]
    pub fn get_condition(&self) -> LouderName {
      self.condition
    }

    /// What the condition came to, read as an integer.
    #[func]
    pub fn get_value_int(&self) -> i32 {
      self.outcome.as_int()
    }

    /// What the condition came to, read as a float.
    #[func]
    pub fn get_value_float(&self) -> f64 {
      self.outcome.as_float() as f64
    }

    /// What the condition came to, read as a name.
    #[func]
    pub fn get_value_name(&self) -> LouderName {
      match &self.outcome {
        LouderVariant::String(name) => *name,
        _ => super::name::LOUDER_NAME_NONE,
      }
    }
  }
}

godot_api! {
  impl LoadFailure {
    /// The record at fault, or `BLANK_POSITION` if the document was.
    #[func]
    pub fn get_position(&self) -> LouderPosition {
      self.position
    }

    /// The [`LoadError`](super::json_loader::LoadError) code.
    #[func]
    pub fn get_error(&self) -> i32 {
      self.error
    }
  }
}

godot_api! {
  impl LouderEvent {
    /// The conversation this happened in, or
    /// [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE).
    #[func]
    pub fn get_conversation(&self) -> LouderName {
      self.conversation
    }

    /// The channel the actor was on.
    #[func]
    pub fn get_channel(&self) -> LouderName {
      self.channel
    }

    /// The actor this happened to, or
    /// [`LOUDER_ACTOR_NONE`](super::actor::LOUDER_ACTOR_NONE).
    #[func]
    pub fn get_actor(&self) -> LouderActor {
      self.actor
    }

    /// Which kind of event this is, as an [`EventData::kind`].
    #[func]
    pub fn get_kind(&self) -> i32 {
      self.data.kind()
    }

    /// The move this event describes, if it describes one.
    #[func]
    pub fn get_motion(&self) -> Option<Gd<Motion>> {
      self.data.motion().map(|motion| Gd::from_object(motion.clone()))
    }

    /// The choice that became available, if one did.
    #[func]
    pub fn get_choice_appearance(&self) -> Option<Gd<ChoiceAppearance>> {
      match &self.data {
        EventData::ChoiceAppearance(appearance) => Some(Gd::from_object(appearance.clone())),
        _ => None,
      }
    }

    /// The additional data key that changed, if one did.
    #[func]
    pub fn get_additional_data_change(&self) -> Option<Gd<AdditionalDataChange>> {
      match &self.data {
        EventData::AdditionalDataChange(change) => Some(Gd::from_object(change.clone())),
        _ => None,
      }
    }

    /// The condition that came to something, if one did.
    #[func]
    pub fn get_condition(&self) -> Option<Gd<ConditionOutcome>> {
      match &self.data {
        EventData::Condition(outcome) => Some(Gd::from_object(outcome.clone())),
        _ => None,
      }
    }

    /// The record of a loaded document the engine could not read, if one failed.
    #[func]
    pub fn get_load_failure(&self) -> Option<Gd<LoadFailure>> {
      match &self.data {
        EventData::LoadError(failure) => Some(Gd::from_object(failure.clone())),
        _ => None,
      }
    }

    /// The speaker, text, expression, or callback this event names, if it names
    /// one. [`LouderEvent::get_kind`] says which of those it is.
    #[func]
    pub fn get_name(&self) -> LouderName {
      self.data.name()
    }
  }
}

godot_api! {
  impl LouderAsk {
    /// The conversation the ask came from.
    #[func]
    pub fn get_conversation(&self) -> LouderName {
      self.conversation
    }

    /// The channel the actor was on.
    #[func]
    pub fn get_channel(&self) -> LouderName {
      self.channel
    }

    /// The actor that raised the ask.
    #[func]
    pub fn get_actor(&self) -> LouderActor {
      self.actor
    }

    /// The move the ask interrupted.
    #[func]
    pub fn get_motion(&self) -> Gd<Motion> {
      Gd::from_object(self.motion.clone())
    }

    /// The callback to call to answer, and the text of its arguments.
    #[func]
    pub fn get_ask(&self) -> LouderCallback {
      self.ask
    }
  }
}