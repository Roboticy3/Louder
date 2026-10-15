#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, Gd};
use super::name::LouderName;
use super::object::{HomogeneousObject, LouderVariant};
use super::hub::LouderPosition;
use super::hub::BLANK_POSITION;

/// An actor participating in conversations.
///
/// Ids are assigned per conversation by [`crate::add_actor`] and are the
/// canonical iteration order for the `next_actor_*` functions.
pub type LouderActor = i32;
pub const LOUDER_ACTOR_NONE : LouderActor = -1;

bitflags::bitflags! {
  /// Flag bits describing an actor.
  #[repr(C)]
  #[derive(Clone, Copy, Default)]
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
///
/// The actor a context holds is handed to GDScript as a copy, so a host that
/// wants one to move says so through
/// [`LouderContext`](super::context::LouderContext) rather than by writing to
/// the copy it was given.
#[derive(Clone)]
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
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

impl Default for Actor {
  /// An actor belonging to nothing: no id, no channel, off the flow.
  fn default() -> Self {
    Actor::new(
      BLANK_POSITION,
      ActorFlags::empty(),
      Vec::new(),
      Vec::new(),
      super::name::LOUDER_NAME_NONE,
    )
  }
}

/// Everything about an actor that depends on the hub it is standing on.
///
/// Grouped apart from [`Actor`] because all of it changes together when the
/// actor travels.
#[derive(Default, Clone)]
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
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

godot_api! {
  impl Actor {
    /// An actor starting at `position` on `channel`.
    ///
    /// Its id is left unset: the context hands one out when the actor is added to
    /// a conversation.
    #[func]
    pub fn create(
      position: LouderPosition,
      flags: i32,
      speak_as: Array<i64>,
      listen_to: Array<i64>,
      channel: LouderName,
    ) -> Gd<Actor> {
      Gd::from_object(Actor::new(
        position,
        ActorFlags::from_bits_truncate(flags),
        speak_as.iter_shared().map(|name| name as LouderName).collect(),
        listen_to.iter_shared().map(|name| name as LouderName).collect(),
        channel,
      ))
    }

    /// The id the context gave this actor, or [`LOUDER_ACTOR_NONE`].
    #[func]
    pub fn get_id(&self) -> LouderActor {
      self.id()
    }

    /// The speakers this actor speaks as.
    #[func(rename = "speak_as")]
    pub fn gd_speak_as(&self) -> Array<i64> {
      self.speak_as().iter().map(|name| *name as i64).collect()
    }

    /// The speakers this actor listens to directly.
    #[func(rename = "listen_to")]
    pub fn gd_listen_to(&self) -> Array<i64> {
      self.listen_to().iter().map(|name| *name as i64).collect()
    }

    /// The flag bits, as [`ActorFlags::bits`].
    #[func]
    pub fn get_flags(&self) -> i32 {
      self.flags().bits()
    }

    /// Whether this actor sees every hub but decides nothing.
    #[func(rename = "is_spectator")]
    pub fn gd_spectator(&self) -> bool {
      Actor::is_spectator(self)
    }

    /// Whether this actor sees auto hubs and every choice.
    #[func(rename = "is_debug")]
    pub fn gd_debug(&self) -> bool {
      Actor::is_debug(self)
    }

    /// Whether this actor takes part in channels and priority.
    #[func]
    pub fn in_flow(&self) -> bool {
      self.flags().in_flow()
    }

    /// Where this actor is standing.
    #[func]
    pub fn get_position(&self) -> LouderPosition {
      self.position()
    }

    /// The channel this actor is in, or [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE).
    #[func]
    pub fn get_channel(&self) -> LouderName {
      self.channel()
    }

    /// This actor's priority within its channel.
    #[func]
    pub fn get_priority(&self) -> i32 {
      self.priority()
    }

    /// Everything about this actor that changes when it travels.
    #[func]
    pub fn get_state(&self) -> Gd<ActorState> {
      Gd::from_object(self.state().clone())
    }

    /// Every key of this actor's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.additional().sorted_keys().into_iter().map(|key| key as i64).collect()
    }

    /// The additional value under `key`, read as an integer, or 0.
    #[func]
    pub fn get_additional_int(&self, key: LouderName) -> i32 {
      self.additional_get(key).map(LouderVariant::as_int).unwrap_or(0)
    }

    /// The additional value under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_additional_float(&self, key: LouderName) -> f64 {
      self.additional_get(key).map(LouderVariant::as_float).unwrap_or(0.0) as f64
    }

    /// The additional value under `key` as a name, or [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE).
    #[func]
    pub fn get_additional_name(&self, key: LouderName) -> LouderName {
      match self.additional_get(key) {
        Some(LouderVariant::String(name)) => *name,
        _ => super::name::LOUDER_NAME_NONE,
      }
    }
  }
}

godot_api! {
  impl ActorState {
    /// Where the actor is standing.
    #[func]
    pub fn get_position(&self) -> LouderPosition {
      ActorState::position(self)
    }

    /// Move the actor to `position`, without telling anyone.
    #[func(rename = "set_position")]
    pub fn gd_set_position(&mut self, position: LouderPosition) {
      ActorState::set_position(self, position);
    }

    /// The channel the actor is in.
    #[func]
    pub fn get_channel(&self) -> LouderName {
      ActorState::channel(self)
    }

    /// Put the actor on `channel`, without telling anyone.
    #[func(rename = "set_channel")]
    pub fn gd_set_channel(&mut self, channel: LouderName) {
      ActorState::set_channel(self, channel);
    }

    /// The actor's priority within its channel.
    #[func]
    pub fn get_priority(&self) -> i32 {
      ActorState::priority(self)
    }

    /// Rank the actor at `priority`, without telling anyone.
    #[func(rename = "set_priority")]
    pub fn gd_set_priority(&mut self, priority: i32) {
      ActorState::set_priority(self, priority);
    }

    /// Take the actor out of the flow: no channel and the default priority.
    #[func(rename = "clear_flow")]
    pub fn gd_clear_flow(&mut self) {
      ActorState::clear_flow(self);
    }

    /// Every key of this actor's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.additional_data().sorted_keys().into_iter().map(|key| key as i64).collect()
    }
  }
}