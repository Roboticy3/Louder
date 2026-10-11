use super::name::LouderName;
use super::object::{HomogeneousObject, LouderVariant};
use super::hub::LouderPosition;

/// An actor participating in conversations.
///
/// Ids are assigned per conversation by [`crate::add_actor`] and are the
/// canonical iteration order for the `next_actor_*` functions.
pub type LouderActor = i32;
pub const LOUDER_ACTOR_NONE : LouderActor = -1;

bitflags::bitflags! {
  /// Flag bits describing an actor.
  #[repr(C)]
  #[derive(Clone, Copy)]
  pub struct ActorFlags: i32 {
    /// All non-auto hubs appear to this actor. Choices only appear if
    /// [`ActorFlags::DEBUG`] is also set.
    const SPECTATOR = 0x1;
    /// Auto hubs and all choices appear to this actor.
    const DEBUG = 0x2;
  }
}

/// An entity that speaks as and listens to speakers within a conversation.
///
/// Its id is assigned by the context when it is added, since ids are per
/// conversation and the context is what hands them out.
pub struct Actor {
  id: LouderActor,
  speak_as: Vec<LouderName>,
  listen_to: Vec<LouderName>,
  flags: ActorFlags,

  state: ActorState,
}

impl Actor {
  pub fn new(
    position: LouderPosition,
    flags: ActorFlags,
    speak_as: Vec<LouderName>,
    listen_to: Vec<LouderName>,
    channel: LouderName,
  ) -> Actor {
    let mut state = ActorState::new(position, channel);
    // Spectators and debug actors are treated as having neither a channel nor a
    // priority, so they must not be given one at creation either.
    if !flags.in_flow() {
      state.clear_flow();
    }

    Actor {
      id: LOUDER_ACTOR_NONE,
      speak_as,
      listen_to,
      flags,
      state,
    }
  }

  pub fn id(&self) -> LouderActor {
    self.id
  }

  /// Assign this actor's id.
  ///
  /// Ids are handed out by [`crate::add_actor`], which creates the actor before
  /// it knows what the next id will be. Until then the actor has no id, and does
  /// not belong to a conversation anybody can look up.
  pub fn set_id(&mut self, id: LouderActor) {
    self.id = id;
  }

  /// The speakers this actor speaks as.
  pub fn speak_as(&self) -> &[LouderName] {
    &self.speak_as
  }

  /// The speakers this actor listens to directly, not counting the speakers
  /// reachable through [`crate::next_actor_listening`].
  pub fn listen_to(&self) -> &[LouderName] {
    &self.listen_to
  }

  pub fn flags(&self) -> ActorFlags {
    self.flags
  }

  pub fn is_spectator(&self) -> bool {
    self.flags.contains(ActorFlags::SPECTATOR)
  }

  pub fn is_debug(&self) -> bool {
    self.flags.contains(ActorFlags::DEBUG)
  }

  pub fn state(&self) -> &ActorState {
    &self.state
  }

  pub fn state_mut(&mut self) -> &mut ActorState {
    &mut self.state
  }

  pub fn position(&self) -> LouderPosition {
    self.state.position()
  }

  pub fn channel(&self) -> LouderName {
    self.state.channel()
  }

  pub fn priority(&self) -> i32 {
    self.state.priority()
  }

  pub fn additional(&self) -> &HomogeneousObject {
    self.state.additional_data()
  }

  pub fn additional_get(&self, key: LouderName) -> Option<&LouderVariant> {
    self.state.additional_data().get(key)
  }
}

impl ActorFlags {
  /// Whether an actor with these flags takes part in channels and priority.
  ///
  /// Spectators see every hub but are not in the flow, so the guide treats them
  /// as having no channel and no priority. This is the one place that rule is
  /// written down.
  pub fn in_flow(self) -> bool {
    !self.intersects(ActorFlags::SPECTATOR | ActorFlags::DEBUG)
  }
}

/// Everything about an actor that depends on the hub it is standing on.
///
/// Grouped apart from [`Actor`] because all of it changes together when the
/// actor travels.
pub struct ActorState {
  channel: LouderName,
  position: LouderPosition,
  priority: i32,
  additional_data: HomogeneousObject,
}

impl ActorState {
  pub fn new(position: LouderPosition, channel: LouderName) -> ActorState {
    ActorState {
      channel,
      position,
      priority: 0,
      additional_data: HomogeneousObject::new(),
    }
  }

  pub fn position(&self) -> LouderPosition {
    self.position
  }

  pub fn set_position(&mut self, position: LouderPosition) {
    self.position = position;
  }

  pub fn channel(&self) -> LouderName {
    self.channel
  }

  pub fn set_channel(&mut self, channel: LouderName) {
    self.channel = channel;
  }

  pub fn priority(&self) -> i32 {
    self.priority
  }

  pub fn set_priority(&mut self, priority: i32) {
    self.priority = priority;
  }

  /// Take the actor out of the flow: no channel and the default priority.
  ///
  /// Channel and priority only mean something together, since priority ranks an
  /// actor within its channel, so this clears both rather than leaving a
  /// priority behind with nothing to rank it in.
  pub fn clear_flow(&mut self) {
    self.channel = super::name::LOUDER_NAME_NONE;
    self.priority = 0;
  }

  pub fn additional_data(&self) -> &HomogeneousObject {
    &self.additional_data
  }

  /// Replace the additional data wholesale.
  ///
  /// A hub's additional data replaces the actor's rather than merging into it,
  /// so that an actor does not accumulate stale keys from hubs it has left.
  pub fn replace_additional_data(&mut self, data: HomogeneousObject) {
    self.additional_data = data;
  }
}