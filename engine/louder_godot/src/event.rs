//! The queue a host reads to find out what the engine did, and the cache of
//! questions it is waiting on.

use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

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
    pub fn get_motion(&self) -> Option<Gd<crate::Motion>> {
      self
        .inner
        .data
        .motion()
        .map(|motion| Gd::from_object(crate::Motion { inner: motion.clone() }))
    }

    /// The choice that became available, if one did.
    #[func]
    pub fn get_choice_appearance(&self) -> Option<Gd<crate::ChoiceAppearance>> {
      match &self.inner.data {
        engine::EventData::ChoiceAppearance(appearance) => {
          Some(Gd::from_object(crate::ChoiceAppearance { inner: appearance.clone() }))
        }
        _ => None,
      }
    }

    /// The additional data key that changed, if one did.
    #[func]
    pub fn get_additional_data_change(&self) -> Option<Gd<crate::AdditionalDataChange>> {
      match &self.inner.data {
        engine::EventData::AdditionalDataChange(change) => {
          Some(Gd::from_object(crate::AdditionalDataChange { inner: change.clone() }))
        }
        _ => None,
      }
    }

    /// The condition that came to something, if one did.
    #[func]
    pub fn get_condition(&self) -> Option<Gd<crate::ConditionOutcome>> {
      match &self.inner.data {
        engine::EventData::Condition(outcome) => {
          Some(Gd::from_object(crate::ConditionOutcome { inner: outcome.clone() }))
        }
        _ => None,
      }
    }

    /// The record of a loaded document the engine could not read, if one failed.
    #[func]
    pub fn get_load_failure(&self) -> Option<Gd<crate::LoadFailure>> {
      match &self.inner.data {
        engine::EventData::LoadError(failure) => Some(Gd::from_object(crate::LoadFailure { inner: failure.clone() })),
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
    pub fn get_motion(&self) -> Gd<crate::Motion> {
      Gd::from_object(crate::Motion { inner: self.inner.motion.clone() })
    }

    /// The callback to call to answer, and the text of its arguments.
    #[func]
    pub fn get_ask(&self) -> engine::LouderCallback {
      self.inner.ask
    }
  }
}

