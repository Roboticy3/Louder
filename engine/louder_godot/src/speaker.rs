//! The voices an actor can speak as or listen to.

use crate::object::HomogeneousObject;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// A voice an actor can speak as or listen to.
///
/// Defined by the conversation config, and every field optional: a speaker that
/// was never declared still resolves, and is treated as public.
///
/// A snapshot of what the engine held. Reloading the config replaces a speaker
/// rather than editing the copy a host was given.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Speaker {
  pub(crate) inner: engine::Speaker,
}

godot_api! {
  impl Speaker {
    /// The voice named `name`.
    #[func]
    pub fn create(
      name: engine::LouderName,
      flags: i32,
      listening: Array<i64>,
      additional: Gd<HomogeneousObject>,
    ) -> Gd<Speaker> {
      Gd::from_object(Speaker {
        inner: engine::Speaker::new(
          name,
          engine::SpeakerFlags::from_bits_truncate(flags),
          listening.iter_shared().map(|name| name as engine::LouderName).collect(),
          additional.bind().inner.clone(),
        ),
      })
    }

    /// The name this voice is declared under.
    #[func]
    pub fn get_name(&self) -> engine::LouderName {
      self.inner.name()
    }

    /// The flag bits, as [`SpeakerFlags::bits`](engine::SpeakerFlags).
    #[func]
    pub fn get_flags(&self) -> i32 {
      self.inner.flags().bits()
    }

    /// Whether hubs with this speaker appear to every actor.
    #[func]
    pub fn is_public(&self) -> bool {
      self.inner.is_public()
    }

    /// The speakers this one drags in with it.
    #[func]
    pub fn listening(&self) -> Array<i64> {
      self.inner.listening().iter().map(|name| *name as i64).collect()
    }

    /// Every key of this speaker's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.inner.additional_data().sorted_keys().into_iter().map(|key| key as i64).collect()
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