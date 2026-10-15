#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, Gd};
use super::name::LouderName;
use super::object::{HomogeneousObject, LouderVariant};

bitflags::bitflags! {
  /// Flag bits describing a speaker.
  #[repr(C)]
  #[derive(Clone, Copy, Default)]
  pub struct SpeakerFlags: i32 {
    /// Hubs with this speaker appear to every actor in the conversation.
    ///
    /// A speaker that is not defined at all is public regardless of this flag.
    const PUBLIC = 0b00000001;
  }
}

/// A voice an actor can speak as or listen to.
///
/// Defined by the conversation config. Every field is optional: a speaker that
/// was never declared still resolves, and is treated as public.
///
/// The speaker a config holds is handed to GDScript as a copy; reloading the
/// config replaces it rather than editing the copy in place.
#[derive(Clone, Default)]
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
pub struct Speaker {
  name: LouderName,
  flags: SpeakerFlags,
  listening: Vec<LouderName>,
  additional_data: HomogeneousObject,
}

impl Speaker {
  pub fn new(
    name: LouderName,
    flags: SpeakerFlags,
    listening: Vec<LouderName>,
    additional_data: HomogeneousObject,
  ) -> Speaker {
    Speaker {
      name,
      flags,
      listening,
      additional_data,
    }
  }

  pub fn name(&self) -> LouderName {
    self.name
  }

  pub fn flags(&self) -> SpeakerFlags {
    self.flags
  }

  pub fn is_public(&self) -> bool {
    self.flags.contains(SpeakerFlags::PUBLIC)
  }

  /// Speakers an actor speaking as this speaker implicitly listens to.
  pub fn listening(&self) -> &[LouderName] {
    &self.listening
  }

  pub fn additional_data(&self) -> &HomogeneousObject {
    &self.additional_data
  }

  pub fn additional_data_mut(&mut self) -> &mut HomogeneousObject {
    &mut self.additional_data
  }

  pub fn additional_get(&self, key: LouderName) -> Option<&LouderVariant> {
    self.additional_data.get(key)
  }
}

godot_api! {
  impl Speaker {
    /// The voice named `name`.
    #[func]
    pub fn create(
      name: LouderName,
      flags: i32,
      listening: Array<i64>,
      additional: Gd<HomogeneousObject>,
    ) -> Gd<Speaker> {
      Gd::from_object(Speaker::new(
        name,
        SpeakerFlags::from_bits_truncate(flags),
        listening.iter_shared().map(|name| name as LouderName).collect(),
        additional.bind().clone(),
      ))
    }

    /// The name this voice is declared under.
    #[func]
    pub fn get_name(&self) -> LouderName {
      self.name()
    }

    /// The flag bits, as [`SpeakerFlags::bits`].
    #[func]
    pub fn get_flags(&self) -> i32 {
      self.flags().bits()
    }

    /// Whether hubs with this speaker appear to every actor.
    #[func(rename = "is_public")]
    pub fn gd_is_public(&self) -> bool {
      Speaker::is_public(self)
    }

    /// The speakers this one drags in with it.
    #[func(rename = "listening")]
    pub fn gd_listening(&self) -> Array<i64> {
      self.listening().iter().map(|name| *name as i64).collect()
    }

    /// Every key of this speaker's additional data.
    #[func]
    pub fn additional_keys(&self) -> Array<i64> {
      self.additional_data().sorted_keys().into_iter().map(|key| key as i64).collect()
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

    /// The additional value under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE).
    #[func]
    pub fn get_additional_name(&self, key: LouderName) -> LouderName {
      match self.additional_get(key) {
        Some(LouderVariant::String(name)) => *name,
        _ => super::name::LOUDER_NAME_NONE,
      }
    }
  }
}

pub trait SpeakerMap {
  fn get_speaker(&self, speaker_name: LouderName) -> Option<&Speaker>;

  /// Whether hubs with this speaker are visible to actors who are not
  /// listening to it.
  ///
  /// An undefined speaker is public, which is what lets a conversation refer
  /// to a speaker that the config never declared.
  fn is_speaker_public(&self, speaker_name: LouderName) -> bool {
    match self.get_speaker(speaker_name) {
      Some(speaker) => speaker.is_public(),
      None => true,
    }
  }
}