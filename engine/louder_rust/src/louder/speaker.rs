use super::name::LouderName;
use super::object::HomogeneousObject;

bitflags::bitflags! {
  /// Flag bits describing a speaker.
  #[repr(transparent)]
  pub struct SpeakerFlags: i32 {
    const PUBLIC = 0b00000001;
  }
}

// Object defining a speaker in the conversation. Can be possessed or heard by actors
pub struct Speaker {
  name: LouderName,
  flags: SpeakerFlags,
  listen_to: Vec<LouderName>,
  additional_data: HomogeneousObject,
}