//! The extra data an actor, hub, or speaker can carry.

use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The "homogeneous object" of the engine guide: a flat mapping between string
/// names and variants, carried by an actor, a hub, or a speaker.
///
/// A snapshot of what the engine held, so writing to one of these changes the
/// copy rather than the conversation.
#[derive(Clone)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct HomogeneousObject {
  pub(crate) inner: engine::HomogeneousObject,
}

godot_api! {
  impl HomogeneousObject {
    /// How many keys are stored.
    #[func]
    pub fn size(&self) -> i32 {
      self.inner.len() as i32
    }

    /// Whether `key` has a value.
    #[func]
    pub fn has(&self, key: engine::LouderName) -> bool {
      self.inner.get(key).is_some()
    }

    /// Every key, in ascending id order.
    #[func]
    pub fn keys(&self) -> Array<i64> {
      self.inner.sorted_keys().into_iter().map(|key| key as i64).collect()
    }

    /// The value under `key` read as an integer, or 0.
    #[func]
    pub fn get_value_int(&self, key: engine::LouderName) -> i32 {
      self.inner.get(key).map(engine::LouderVariant::as_int).unwrap_or(0)
    }

    /// The value under `key` read as a float, or 0.0.
    #[func]
    pub fn get_value_float(&self, key: engine::LouderName) -> f64 {
      self.inner.get(key).map(engine::LouderVariant::as_float).unwrap_or(0.0) as f64
    }

    /// The name under `key`, or [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE)
    /// if the value is not a string.
    #[func]
    pub fn get_value_name(&self, key: engine::LouderName) -> engine::LouderName {
      match self.inner.get(key) {
        Some(engine::LouderVariant::String(name)) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }

    /// Store an integer under `key`.
    #[func]
    pub fn set_value_int(&mut self, key: engine::LouderName, value: i32) {
      self.inner.set(key, engine::LouderVariant::Int(value));
    }

    /// Store a float under `key`.
    #[func]
    pub fn set_value_float(&mut self, key: engine::LouderName, value: f64) {
      self.inner.set(key, engine::LouderVariant::Float(value as f32));
    }

    /// Store a name under `key`.
    #[func]
    pub fn set_value_name(&mut self, key: engine::LouderName, value: engine::LouderName) {
      self.inner.set(key, engine::LouderVariant::String(value));
    }

    /// Forget `key`.
    #[func]
    pub fn forget(&mut self, key: engine::LouderName) {
      self.inner.remove(key);
    }

    /// Forget every key.
    #[func]
    pub fn forget_all(&mut self) {
      self.inner.clear();
    }
  }
}