//! The queue a host reads to find out what the engine did, and the cache of
//! questions it is waiting on.

use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// One move of an actor from one hub to another.
///
/// Every event a move produces carries the same motion, which is how a host
/// reads where an actor came from and where it went.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Motion {
  pub(crate) inner: engine::Motion,
}

godot_api! {
  impl Motion {
    /// Where the actor came from.
    #[func]
    pub fn get_source_position(&self) -> engine::LouderPosition {
      self.inner.source_position
    }

    /// Where the actor went.
    #[func]
    pub fn get_target_position(&self) -> engine::LouderPosition {
      self.inner.target_position
    }

    /// The choice the actor picked, or -1 if it did not pick anything.
    #[func]
    pub fn get_choice_index(&self) -> i32 {
      self.inner.choice_index
    }

    /// The speaker the hub appears to have, for this move.
    ///
    /// A hub with a speaker of its own speaks as that speaker. A hub without one
    /// is anonymous, and the voice of the actor making the move is projected
    /// onto it for the length of the move, so the same hub carries a different
    /// speaker for each actor that walks onto it.
    #[func]
    pub fn get_speaker(&self) -> engine::LouderName {
      self.inner.speaker
    }

    /// Whether the actor saw this hub without moving onto it.
    #[func]
    pub fn is_background(&self) -> bool {
      self.inner.background
    }
  }
}

/// A choice becoming available on a hub.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct ChoiceAppearance {
  pub(crate) inner: engine::ChoiceAppearance,
}

godot_api! {
  impl ChoiceAppearance {
    /// The hub the choice appeared on.
    #[func]
    pub fn get_position(&self) -> engine::LouderPosition {
      self.inner.position
    }

    /// Which choice appeared.
    #[func]
    pub fn get_choice_index(&self) -> i32 {
      self.inner.choice_index
    }
  }
}

/// A key of an actor's additional data being replaced.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct AdditionalDataChange {
  pub(crate) inner: engine::AdditionalDataChange,
}

godot_api! {
  impl AdditionalDataChange {
    /// The key whose value was replaced.
    #[func]
    pub fn get_key(&self) -> engine::LouderName {
      self.inner.key
    }
  }
}

/// What a condition on a hub came to, when an actor landed on it.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct ConditionOutcome {
  pub(crate) inner: engine::ConditionOutcome,
}

godot_api! {
  impl ConditionOutcome {
    /// The condition that was evaluated.
    #[func]
    pub fn get_condition(&self) -> engine::LouderName {
      self.inner.condition
    }

    /// What the condition came to, read as an integer.
    #[func]
    pub fn get_value_int(&self) -> i32 {
      self.inner.outcome.as_int()
    }

    /// What the condition came to, read as a float.
    #[func]
    pub fn get_value_float(&self) -> f64 {
      self.inner.outcome.as_float() as f64
    }

    /// What the condition came to, read as a name.
    #[func]
    pub fn get_value_name(&self) -> engine::LouderName {
      match &self.inner.outcome {
        engine::LouderVariant::String(name) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }
  }
}

/// A record in a document the engine could not read.
///
/// Carries the position of the record that was at fault, or -1 when the problem
/// was with the document as a whole.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LoadFailure {
  pub(crate) inner: engine::LoadFailure,
}

godot_api! {
  impl LoadFailure {
    /// The record at fault, or -1 if the document was.
    #[func]
    pub fn get_position(&self) -> engine::LouderPosition {
      self.inner.position
    }

    /// The [`LoadError`](crate::loader::LouderLoader) code.
    #[func]
    pub fn get_error(&self) -> i32 {
      self.inner.error
    }
  }
}

/// One entry in the event queue.
///
/// The queue is polled from index 0 upwards and emptied with
/// [`LouderContext::flush_events`](crate::context::LouderContext).
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderEvent {
  pub(crate) inner: engine::LouderEvent,
}

godot_api! {
  impl LouderEvent {
    /// The conversation this happened in, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_conversation(&self) -> engine::LouderName {
      self.inner.conversation
    }

    /// The channel the actor was on.
    #[func]
    pub fn get_channel(&self) -> engine::LouderName {
      self.inner.channel
    }

    /// The actor this happened to, or
    /// [`LOUDER_ACTOR_NONE`](engine::LOUDER_ACTOR_NONE).
    #[func]
    pub fn get_actor(&self) -> engine::LouderActor {
      self.inner.actor
    }

    /// Which kind of event this is, as an [`EventData::kind`](engine::EventData).
    #[func]
    pub fn get_kind(&self) -> i32 {
      self.inner.data.kind()
    }

    /// The move this event describes, if it describes one.
    #[func]
    pub fn get_motion(&self) -> Option<Gd<Motion>> {
      self
        .inner
        .data
        .motion()
        .map(|motion| Gd::from_object(Motion { inner: motion.clone() }))
    }

    /// The choice that became available, if one did.
    #[func]
    pub fn get_choice_appearance(&self) -> Option<Gd<ChoiceAppearance>> {
      match &self.inner.data {
        engine::EventData::ChoiceAppearance(appearance) => {
          Some(Gd::from_object(ChoiceAppearance { inner: appearance.clone() }))
        }
        _ => None,
      }
    }

    /// The additional data key that changed, if one did.
    #[func]
    pub fn get_additional_data_change(&self) -> Option<Gd<AdditionalDataChange>> {
      match &self.inner.data {
        engine::EventData::AdditionalDataChange(change) => {
          Some(Gd::from_object(AdditionalDataChange { inner: change.clone() }))
        }
        _ => None,
      }
    }

    /// The condition that came to something, if one did.
    #[func]
    pub fn get_condition(&self) -> Option<Gd<ConditionOutcome>> {
      match &self.inner.data {
        engine::EventData::Condition(outcome) => {
          Some(Gd::from_object(ConditionOutcome { inner: outcome.clone() }))
        }
        _ => None,
      }
    }

    /// The record of a loaded document the engine could not read, if one failed.
    #[func]
    pub fn get_load_failure(&self) -> Option<Gd<LoadFailure>> {
      match &self.inner.data {
        engine::EventData::LoadError(failure) => Some(Gd::from_object(LoadFailure { inner: failure.clone() })),
        _ => None,
      }
    }

    /// The speaker, text, expression, or callback this event names, if it names
    /// one. [`LouderEvent::get_kind`] says which of those it is.
    #[func]
    pub fn get_name(&self) -> engine::LouderName {
      self.inner.data.name()
    }

    /// The whole event as a dictionary, so it can cross a network boundary
    /// without being taken apart field by field.
    #[func]
    pub fn to_dictionary(&self) -> Dictionary<Variant, Variant> {
      let mut dict = Dictionary::<Variant, Variant>::new();
      dict.set("conversation", self.inner.conversation);
      dict.set("channel", self.inner.channel);
      dict.set("actor", self.inner.actor);
      dict.set("kind", self.inner.data.kind());

      if let Some(motion) = self.inner.data.motion() {
        let mut motion_dict = Dictionary::<Variant, Variant>::new();
        motion_dict.set("source_position", motion.source_position);
        motion_dict.set("target_position", motion.target_position);
        motion_dict.set("choice_index", motion.choice_index);
        motion_dict.set("speaker", motion.speaker);
        motion_dict.set("background", motion.background);
        dict.set("motion", &motion_dict);
      }

      dict.set("name", self.inner.data.name());

      match &self.inner.data {
        engine::EventData::ChoiceAppearance(appearance) => {
          let mut appearance_dict = Dictionary::<Variant, Variant>::new();
          appearance_dict.set("position", appearance.position);
          appearance_dict.set("choice_index", appearance.choice_index);
          dict.set("choice_appearance", &appearance_dict);
        }
        engine::EventData::AdditionalDataChange(change) => {
          dict.set("additional_data_key", change.key);
        }
        engine::EventData::Condition(outcome) => {
          let mut condition_dict = Dictionary::<Variant, Variant>::new();
          condition_dict.set("condition", outcome.condition);
          condition_dict.set("outcome_int", match &outcome.outcome {
            engine::LouderVariant::Int(value) => *value,
            _ => 0,
          });
          dict.set("condition", &condition_dict);
        }
        engine::EventData::LoadError(failure) => {
          let mut failure_dict = Dictionary::<Variant, Variant>::new();
          failure_dict.set("position", failure.position);
          failure_dict.set("error", failure.error);
          dict.set("load_failure", &failure_dict);
        }
        _ => {}
      }

      dict
    }
  }
}

/// A question the engine has asked the host and is waiting on.
///
/// Answering an ask moves it to the back of the cache, so indexes shift; read
/// the cache again after each answer.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderAsk {
  pub(crate) inner: engine::LouderAsk,
}

godot_api! {
  impl LouderAsk {
    /// The conversation the ask came from.
    #[func]
    pub fn get_conversation(&self) -> engine::LouderName {
      self.inner.conversation
    }

    /// The channel the actor was on.
    #[func]
    pub fn get_channel(&self) -> engine::LouderName {
      self.inner.channel
    }

    /// The actor that raised the ask.
    #[func]
    pub fn get_actor(&self) -> engine::LouderActor {
      self.inner.actor
    }

    /// The move the ask interrupted.
    #[func]
    pub fn get_motion(&self) -> Gd<Motion> {
      Gd::from_object(Motion { inner: self.inner.motion.clone() })
    }

    /// The callback to call to answer, and the text of its arguments.
    #[func]
    pub fn get_ask(&self) -> engine::LouderCallback {
      self.inner.ask
    }
  }
}
/// An interrupt: an actor and the expression it is waiting on.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Interrupt {
  pub(crate) inner: engine::Interrupt,
}

godot_api! {
  impl Interrupt {
    #[func]
    pub fn get_actor(&self) -> engine::LouderActor {
      self.inner.actor
    }

    #[func]
    pub fn get_expression(&self) -> engine::LouderName {
      self.inner.expression
    }
  }
}
