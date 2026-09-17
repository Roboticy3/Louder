use super::object::{HomogeneousObject, LouderName};
use super::hub::LouderPosition;

/// An actor participating in conversations.
pub type LouderActor = i32;

bitflags::bitflags! {
  /// Flag bits describing an actor.
  #[repr(transparent)]
  pub struct ActorFlags: i32 {}
}

//Object representing a player in the conversation
pub struct Actor {
  id: LouderActor,
  speak_as: Vec<LouderName>,
  listen_to: Vec<LouderName>,
  flags: ActorFlags,

  state: ActorState,
}

pub struct ActorState {
  channel: LouderName,
  position: LouderPosition,
  priority: i32,
  additional_data: HomogeneousObject,
}