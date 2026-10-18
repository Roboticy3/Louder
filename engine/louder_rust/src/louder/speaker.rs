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
/// The speaker a config holds is handed out as a copy; reloading the config
/// replaces it rather than editing the copy in place.
#[derive(Clone, Default)]
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
