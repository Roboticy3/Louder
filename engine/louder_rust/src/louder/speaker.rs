use super::object::{HomogeneousObject, LouderName};

/// Flag bits describing a speaker.
pub type SpeakerFlags = i32;

// Object defining a speaker in the conversation. Can be possessed or heard by actors
pub struct Speaker {
  name: LouderName,
  flags: SpeakerFlags,
  listen_to: Vec<LouderName>,
  additional_data: HomogeneousObject,
}