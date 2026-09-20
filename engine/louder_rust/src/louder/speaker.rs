use super::name::LouderName;
use super::object::HomogeneousObject;

bitflags::bitflags! {
  /// Flag bits describing a speaker.
  #[repr(C)]
  #[derive(Clone)]
  pub struct SpeakerFlags: i32 {
    const NONE = 0b0;
    const PUBLIC = 0b00000001;
  }
}

// Object defining a speaker in the conversation. Can be possessed or heard by actors
pub struct Speaker {
  pub name: LouderName,
  pub flags: SpeakerFlags,
  pub listen_to: Vec<LouderName>,
  pub additional_data: HomogeneousObject,
}

pub trait SpeakerMap {
  fn get_speaker(&self, speaker_name:LouderName) -> Option<&Speaker>;
}