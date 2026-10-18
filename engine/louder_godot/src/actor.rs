//! Actors, and everything about an actor that changes when it travels.

use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// An entity that speaks as and listens to speakers within a conversation.
///
/// A snapshot of what the engine held, so writing to one of these changes the
/// copy rather than the conversation. To move an actor, ask the
/// [`LouderContext`](crate::context::LouderContext) to move it.
#[derive(Clone)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Actor {
  pub(crate) inner: engine::Actor,
}

godot_api! {
  impl Actor {
    /// An actor starting at `position` on `channel`.
    ///
    /// Its id is left unset: the context hands one out when the actor is added to
    /// a conversation.
    #[func]
    pub fn create(
      position: engine::LouderPosition,
      flags: i32,
      speak_as: Array<i64>,
      listen_to: Array<i64>,
      channel: engine::LouderName,
    ) -> Gd<Actor> {
      Gd::from_object(Actor {
        inner: engine::Actor::new(
          position,
          engine::ActorFlags::from_bits_truncate(flags),
          speak_as.iter_shared().map(|name| name as engine::LouderName).collect(),
          listen_to.iter_shared().map(|name| name as engine::LouderName).collect(),
          channel,
        ),
      })
    }

    /// The id the context gave this actor, or
    /// [`LOUDER_ACTOR_NONE`](engine::LOUDER_ACTOR_NONE).
    #[func]
    pub fn get_id(&self) -> engine::LouderActor {
      self.inner.id()
    }

    /// The speakers this actor speaks as.
    #[func]
    pub fn speak_as(&self) -> Array<i64> {
      self.inner.speak_as().iter().map(|name| *name as i64).collect()
    }

    /// The speakers this actor listens to directly.
    #[func]
    pub fn listen_to(&self) -> Array<i64> {
      self.inner.listen_to().iter().map(|name| *name as i64).collect()
    }

    /// The flag bits, as [`ActorFlags::bits`](engine::ActorFlags).
    #[func]
    pub fn get_flags(&self) -> i32 {
      self.inner.flags().bits()
    }

    /// Whether this actor sees every hub but decides nothing.
    #[func]
    pub fn is_spectator(&self) -> bool {
      self.inner.is_spectator()
    }

    /// Whether this actor sees auto hubs and every choice.
    #[func]
    pub fn is_debug(&self) -> bool {
      self.inner.is_debug()
    }

    /// Whether this actor takes part in channels and priority.
    #[func]
    pub fn in_flow(&self) -> bool {
      self.inner.flags().in_flow()
    }

    /// Where this actor is standing.
    #[func]
    pub fn get_position(&self) -> engine::LouderPosition {
      self.inner.position()
    }

    /// The channel this actor is in, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_channel(&self) -> engine::LouderName {
      self.inner.channel()
    }

    /// This actor's priority within its channel.
    #[func]
    pub fn get_priority(&self) -> i32 {
      self.inner.priority()
    }

    /// Everything about this actor that changes when it travels.
    #[func]
    pub fn get_state(&self) -> Gd<ActorState> {
      Gd::from_object(ActorState { inner: self.inner.state().clone() })
    }

    /// Every key of this actor's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.inner.additional().sorted_keys().into_iter().map(|key| key as i64).collect()
    }

    /// The additional value under `key`, read as an integer, or 0.
    #[func]
    pub fn get_additional_int(&self, key: engine::LouderName) -> i32 {
      self.inner.additional_get(key).map(engine::LouderVariant::as_int).unwrap_or(0)
    }

    /// The additional value under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_additional_float(&self, key: engine::LouderName) -> f64 {
      self.inner.additional_get(key).map(engine::LouderVariant::as_float).unwrap_or(0.0) as f64
    }

    /// The additional value under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_additional_name(&self, key: engine::LouderName) -> engine::LouderName {
      match self.inner.additional_get(key) {
        Some(engine::LouderVariant::String(name)) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }
  }
}

/// Everything about an actor that depends on the hub it is standing on.
///
/// Grouped apart from [`Actor`] because all of it changes together when the
/// actor travels.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct ActorState {
  pub(crate) inner: engine::ActorState,
}

godot_api! {
  impl ActorState {
    /// Where the actor is standing.
    #[func]
    pub fn get_position(&self) -> engine::LouderPosition {
      self.inner.position()
    }

    /// Move the actor to `position`, without telling anyone.
    #[func]
    pub fn set_position(&mut self, position: engine::LouderPosition) {
      self.inner.set_position(position);
    }

    /// The channel the actor is in.
    #[func]
    pub fn get_channel(&self) -> engine::LouderName {
      self.inner.channel()
    }

    /// Put the actor on `channel`, without telling anyone.
    #[func]
    pub fn set_channel(&mut self, channel: engine::LouderName) {
      self.inner.set_channel(channel);
    }

    /// The actor's priority within its channel.
    #[func]
    pub fn get_priority(&self) -> i32 {
      self.inner.priority()
    }

    /// Rank the actor at `priority`, without telling anyone.
    #[func]
    pub fn set_priority(&mut self, priority: i32) {
      self.inner.set_priority(priority);
    }

    /// Take the actor out of the flow: no channel and the default priority.
    #[func]
    pub fn clear_flow(&mut self) {
      self.inner.clear_flow();
    }

    /// Every key of this actor's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.inner.additional_data().sorted_keys().into_iter().map(|key| key as i64).collect()
    }
  }
}