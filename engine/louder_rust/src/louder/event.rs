
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

#[derive(Debug, Clone)]
#[repr(C)] //creates a union structure in the C FFI.
pub enum EventData {
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
}

#[derive(Debug, Clone)]
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

pub trait EventCache<E> {
  fn get_event(&self, index: usize) -> Option<&E>;

  fn clear_events(&mut self);
}

pub trait EventMap<K, T : Clone> {
  fn get_event_key(&self, index: usize, key: &K) -> T;
}

pub trait AskListener {
  fn answer_ask(&self, index: usize, verdict: i32) -> bool;
}