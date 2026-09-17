//Object representing a player in the conversation
struct Actor {
  id: LouderActor,
  speak_as: Vec<LouderName>,
  listen_to: Vec<LouderName>,
  flags: ActorFlags,

  state: ActorState,
}

struct ActorState {
  channel: LouderName,
  position: LouderPosition,
  priority: i32,
  additional_data: HomogeneousObject,
}