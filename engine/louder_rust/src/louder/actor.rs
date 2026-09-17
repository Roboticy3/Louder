use mod object::HomogeneousObject;
use mod object::LouderName;

use mod hub::LouderPosition;

/// An actor participating in conversations.
pub type LouderActor = i32;

/// Flag bits describing an actor.
pub type ActorFlags = i32;

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