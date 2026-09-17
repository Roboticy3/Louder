// Object defining a speaker in the conversation. Can be possessed or heard by actors
struct Speaker {
  name: LouderName,
  flags: SpeakerFlags,
  listen_to: Vec<LouderName>,
  additional_data: HomogeneousObject,
}