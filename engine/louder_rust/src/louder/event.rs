
use crate::louder::event::EventData::None;
use crate::louder::object::LouderVariant;

use super::callback::LouderCallback;
use super::expression::LouderExpression;

use super::actor::LouderActor;
use super::name::LouderName;
use super::hub::LouderPosition;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Motion {
  pub source_position: LouderPosition,
  pub target_position: LouderPosition,
  pub choice_index: i32,

  pub background: bool,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ChoiceAppearance {
  pub position: LouderPosition,
  pub choice_index: i32
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct AdditionalDataChange {
  pub key: LouderName,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct ConditionOutcome {
  pub condition:LouderName,
  pub outcome: LouderVariant,
}

/// A record in a document the engine could not read.
///
/// Carries the position of the record that was at fault, or
/// [`LOUDER_POSITION_NONE`] when the problem was with the document as a whole.
#[derive(Debug, Clone)]
#[repr(C)]
pub struct LoadFailure {
  pub position: LouderPosition,
  pub error: i32,
}

#[derive(Debug, Clone)]
#[repr(C)] //creates a union structure in the C FFI.
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

impl Default for EventData {
  fn default() -> Self {
    return None
  }
}

#[derive(Debug, Clone, Default)]
#[repr(C)]
pub struct LouderEvent {
  pub conversation: LouderName,
  pub channel: LouderName,
  pub actor: LouderActor,
  pub data: EventData,
}

#[derive(Debug, Clone)]
#[repr(C)]
pub struct LouderAsk {
  pub conversation: LouderName,
  pub channel: LouderName,
  pub actor: LouderActor,
  pub motion: Motion,
  pub ask: LouderCallback,
}