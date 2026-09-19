mod test_linkage;

pub mod louder;

use louder::actor::*;
use louder::context::*;
use louder::conversation::*;
use louder::event::*;
use louder::hub::*;
use louder::json_loader::*;
use louder::object::*;
use louder::name::*;
use louder::simulator::*;
use louder::speaker::*;

/// An error code returned by most operations.
///
/// - [`LOUDER_ERROR_NONE`]: no error.
/// - [`LOUDER_ERROR_NOT_FOUND`]: the requested object does not exist.
pub type LouderError = i32;

/// No error.
pub const LOUDER_SUCCESS: LouderError = 0;
/// The requested object does not exist.
pub const LOUDER_ERROR_NOT_FOUND: LouderError = -1;

/// Key passed to [`crate::get_event_position`].
///
/// Each variant names a [`LouderPosition`] field of an event's data.
#[repr(i32)]
pub enum EventPosition {
  /// Get the position the actor of this event's motion starts from.
  Source = 0,
  /// Get the position the actor of this event's motion moves to.
  Target = 1,
  /// Get the position this event's choice appearance is for.
  ChoiceAppearance = 2,
}

/// Key passed to [`crate::get_event_int`].
///
/// Each variant names an `i32` field of an event's data.
#[repr(i32)]
pub enum EventInt {
  /// Get the index of the choice selected by the actor of this event's motion,
  /// or `-1` if no choice was selected.
  SourceChoiceIndex = 0,
  /// Get the index of the choice of this event's choice appearance.
  ChoiceIndex = 1,
}

/// Key passed to [`crate::get_ask_name`].
#[repr(i32)]
pub enum AskName {
  /// Get the conversation associated with this ask.
  Conversation = 0,
  /// Get the channel name of this ask if it indicates a channel change.
  Channel = 1,
  /// Get the callback which caused this ask.
  Callback = 2,
}

/// Key passed to [`crate::get_ask_int`].
#[repr(i32)]
pub enum AskInt {
  /// Get the actor associated with this ask's motion, or just the actor.
  Actor = 0,
  /// Get the index of the choice selected on the source position by the actor of
  /// this ask's motion.
  SourceChoiceIndex = 2,
}

/// Key passed to [`crate::get_ask_position`]
pub enum AskPosition {
  /// Get the position the actor of this event's motion starts from.
  Source = 0,
  /// Get the position the actor of this event's motion moves to.
  Target = 1,
}


#[unsafe(no_mangle)]
extern "system" fn test_linkage() -> i32 {
  test_linkage::test_linkage::test_linkage()
}

#[unsafe(no_mangle)]
extern "system" fn get_next_encoding(start: LouderEncoding) -> LouderEncoding {
  let e = encoding::all::encodings();
  if start + 1 >= e.len() {
    return 0;
  }
  return start + 1;
}

/// Get the name of an encoding... in utf-8
#[unsafe(no_mangle)]
extern "system" fn get_encoding_name(index: LouderEncoding, buffer: *mut u8, max_size: usize) -> usize {
  let e = encoding::all::encodings();
  if index >= e.len() {
    return 0;
  }
  let name = &e[index].name();
  let name_bytes = name.as_bytes();
  let copy_length = std::cmp::min(name_bytes.len(), max_size);
  unsafe {
    std::ptr::copy_nonoverlapping(name_bytes.as_ptr(), buffer as *mut u8, copy_length);
    *buffer.offset(copy_length as isize) = 0;
    return copy_length;
  }
}

#[unsafe(no_mangle)]
extern "system" fn change_encoding(buffer: *mut u8, max_size: usize, from_encoding: LouderEncoding, to_encoding: LouderEncoding) -> usize {
  let e = encoding::all::encodings();
  if from_encoding >= e.len() || to_encoding >= e.len() {
    return 0;
  }
  let from = &e[from_encoding];
  let to = &e[to_encoding];
  let input_bytes = unsafe { std::slice::from_raw_parts(buffer, max_size) };
  let output_bytes = match from.decode(input_bytes, encoding::DecoderTrap::Replace) {
    Ok(s) => match to.encode(&s, encoding::EncoderTrap::Replace) {
      Ok(b) => b,
      Err(_) => return 0,
    },
    Err(_) => return 0,
  };
  let copy_length = std::cmp::min(output_bytes.len(), max_size);
  unsafe {
    std::ptr::copy_nonoverlapping(output_bytes.as_ptr(), buffer as *mut u8, copy_length);
    *buffer.offset(copy_length as isize) = 0;
  }
  copy_length
}

/// Get the id of a string.
///
/// Returns [`LOUDER_NAME_NONE`] if the string was never declared.
///
/// - `buffer`: the string bytes to look up.
/// - `max_size`: the maximum size of `buffer` in bytes.
/// - `encoding`: the encoding of the string.
#[unsafe(no_mangle)]
pub extern "system" fn get_named_id(ctx: *const LouderContext, buffer: *const u8, max_size: usize, encoding: LouderEncoding) -> LouderName {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_NAME_NONE;
  }
  
  let c = LouderNameContent {
    bytes: unsafe { std::slice::from_raw_parts(buffer, max_size).to_vec() },
    encoding,
  };

  unsafe { 
    match (*ctx).find_name(&c) {
      LouderContextResult::Found(name) => name,
      LouderContextResult::NotFound(_) => LOUDER_NAME_NONE,
    }
  }
}

/// Assign the id of a string.
///
/// If the string was already declared, returns [`LOUDER_NAME_NONE`]. Otherwise a new
/// id is assigned.
#[unsafe(no_mangle)]
pub extern "system" fn declare_name(ctx: *mut LouderContext, buffer: *const u8, max_size: usize, encoding: LouderEncoding) -> LouderName {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_NAME_NONE;
  }

  let c = LouderNameContent {
    bytes: unsafe { std::slice::from_raw_parts(buffer, max_size).to_vec() },
    encoding,
  };

  unsafe { 
    match (*ctx).add_name(&c) {
      LouderContextResult::Found(_) => LOUDER_NAME_NONE,
      LouderContextResult::NotFound(name) => name,
    }
  }
}

/// Read the string associated with an id into `buffer`.
///
/// Writes 0 bytes if no name is associated with the id.
/// Returns number of bytes written.
#[unsafe(no_mangle)]
pub extern "system" fn read_name(ctx: *const LouderContext, name: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> usize {
  if ctx.is_null() || buffer.is_null() {
    return 0;
  }

  let name_context = unsafe { (*ctx).get_name_content(&name) };
  if name_context.is_none() {
    return 0;
  }

  let name_context = name_context.unwrap();
  if name_context.encoding != encoding {
    return 0;
  }

  let copy_length = std::cmp::min(name_context.bytes.len(), max_size);
  unsafe {
    std::ptr::copy_nonoverlapping(name_context.bytes.as_ptr(), buffer as *mut u8, copy_length);
    *buffer.offset(copy_length as isize) = 0;
  }
  
  copy_length
}

/// Check whether an event exists at `index`.
///
/// Returns [`LOUDER_ERROR_NONE`] if an event exists, or
/// [`LOUDER_ERROR_NOT_FOUND`] if the index is invalid. Poll events with
/// increasing indexes until this stops returning an event.
#[unsafe(no_mangle)]
pub extern "system" fn is_event(ctx: *const LouderContext, index: usize) -> LouderError {
  if ctx.is_null() {
    return LOUDER_NAME_NONE
  }

  let e : Option<&LouderEvent> = unsafe {
    (*ctx).get_event(index)
  };

  match e {
    Some(_) => LOUDER_SUCCESS,
    None => LOUDER_ERROR_NOT_FOUND
  }
}

/// Get the type flags of an event by index.
///
/// Returns `0` ("no flags") if the index is invalid. Events with 0 flags are
/// invalid and shouldn't appear. See [`EventCode`].
#[unsafe(no_mangle)]
pub extern "system" fn get_event_code(ctx: *const LouderContext, index: usize) -> EventCode {
  if ctx.is_null() {
    return EventCode::None
  }

  let e : Option<&LouderEvent> = unsafe {
    (*ctx).get_event(index)
  };

  match e {
    Some(event) => EventCode::from(&event.data),
    None => EventCode::None
  }
}

/// Get the additional data changed by this event.
/// Additional data is changed on the actor returned by get_event_actor.
/// 
/// Returns [`LOUDER_NAME_NONE`] if no data was changed by this event.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_additional_data_changed(ctx: *const LouderContext, index: usize) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE
  }

  let e : Option<&LouderEvent> = unsafe {
    (*ctx).get_event(index)
  };

  if let Some(event) = e {
    if let EventData::AdditionalDataChange(a) = &event.data {
      return a.key;
    }
  }

  LOUDER_NAME_NONE
}

/// Get the actor associated with an event.
///
/// Returns [`LOUDER_ACTOR_NONE`] if no actor is associated with this event.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_actor(ctx: *const LouderContext, index: usize) -> LouderActor {
  if ctx.is_null() {
    return LOUDER_ACTOR_NONE
  }

  let e : Option<&LouderEvent> = unsafe {
    (*ctx).get_event(index)
  };

  if let Some(event) = e {
    return event.actor;
  }

  return LOUDER_ACTOR_NONE;
}

/// Get a position by event index for a key.
///
/// Returns `-1` if not available.
///
/// - [`EventPosition::Source`]: the origin of this event's motion.
/// - [`EventPosition::Target`]: the destination of this event's motion.
/// - [`EventPosition::Position`]: the position this event's choice appearance is for.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_position(ctx: *const LouderContext, index: usize, key: EventPosition) -> LouderPosition {
  todo!()
}

/// Get an integer by event index for a key.
///
/// Returns `-1` if not available.
///
/// - [`EventInt::SourceChoiceIndex`]: the index of the choice selected by the
///   actor of this event's motion, or `-1` if none.
/// - [`EventInt::ChoiceIndex`]: the index of the choice of this event's choice
///   appearance.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_int(ctx: *const LouderContext, index: i32, key: EventInt) -> i32 {
  todo!()
}

/// Read a string by event index for a key into `buffer`.
///
/// Reads 0 bytes if the index is invalid or no object is associated with the
/// key for this event.
///


/// Get the priority of this event if it indicates a hub appearance.
///
/// Returns `0` otherwise, or for an invalid index.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_priority(ctx: *const LouderContext, index: i32) -> i32 {
  todo!()
}

/// If [`EventFlags::CONDITION`] is set, then get the type of value this event's condition(s) evaluates to.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_condition_variant_type(ctx: *const LouderContext, index: i32, condition_index: i32) -> LouderVariantType {
  todo!()
}

/// If [`EventFlags::CONDITION`] is set, then get this event's condition as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_condition_int(ctx: *const LouderContext, index: i32, condition_index: i32) -> i32 {
  todo!()
}

/// If [`EventFlags::CONDITION`] is set, then get this event's condition as an float.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_condition_float(ctx: *const LouderContext, index: i32, condition_index: i32) -> f32 {
  todo!()
}

/// If [`EventFlags::CONDITION`] is set, then get this event's condition as a name.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_condition_string(ctx: *const LouderContext, index: i32, condition_index: i32) -> LouderName {
  todo!()
}

/// If [`EventFlags::HUB_APPEARANCE`] is set, then get the type of the additional data of the hub at motion.target_position under `key`.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_additional_variant_type(ctx: *const LouderContext, index: i32, key: LouderName) -> LouderVariantType {
  todo!()
}

/// If [`EventFlags::HUB_APPEARANCE`] is set, then get the additional data of the hub at motion.target_position under `key` as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_additional_int(ctx: *const LouderContext, index: i32, key: LouderName) -> i32 {
  todo!()
}

/// If [`EventFlags::HUB_APPEARANCE`] is set, then get the additional data of the hub at motion.target_position under `key` as a float.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_additional_float(ctx: *const LouderContext, index: i32, key: LouderName) -> f32 {
  todo!()
}

/// If [`EventFlags::HUB_APPEARANCE`] is set, then get the additional data of the hub at motion.target_position under `key` as a string.
#[unsafe(no_mangle)]
pub extern "system" fn get_event_additional_string(ctx: *const LouderContext, index: i32, key: LouderName) -> LouderName {
  todo!()
}

/// Flush all events from the queue.
///
/// Returns the number of events flushed. Poll events with [`is_event`] until it
/// stops returning an event, then call this.
#[unsafe(no_mangle)]
pub extern "system" fn flush_events(ctx: *mut LouderContext) -> i32 {
  todo!()
}

/// Check whether an ask exists at `index`.
#[unsafe(no_mangle)]
pub extern "system" fn is_ask(ctx: *const LouderContext, index: i32) -> i32 {
  todo!()
}

/// Get a name by ask index for a key.
///
/// - [`AskName::Conversation`]: the conversation associated with this ask.
/// - [`AskName::Channel`]: the channel name if this ask indicates a channel change.
#[unsafe(no_mangle)]
pub extern "system" fn get_ask_name(ctx: *const LouderContext, index: i32, key: AskName) -> LouderName {
  todo!()
}

/// Get an integer by ask index for a key.
///
/// Returns `-1` if not available or the index is invalid.
///
/// - [`AskInt::Actor`]: the actor associated with this ask's motion or just the actor.
/// - [`AskInt::SourcePosition`]: the source position of this ask's motion.
/// - [`AskInt::SourceChoiceSelected`]: the choice selected on the source position.
/// - [`AskInt::CallbackPosition`]: the position the callback originates from.
#[unsafe(no_mangle)]
pub extern "system" fn get_ask_int(ctx: *const LouderContext, index: i32, key: AskInt) -> i32 {
  todo!()
}

/// Get the callback causing this ask as a name.
#[unsafe(no_mangle)]
pub extern "system" fn get_ask_callback(ctx: *const LouderContext, index: i32) -> LouderName {
  todo!()
}

/// Answer an ask with a verdict.
#[unsafe(no_mangle)]
pub extern "system" fn answer_ask(ctx: *mut LouderContext, index: i32, verdict: i32) -> i32 {
  todo!()
}

/// Get the flags of a speaker.
#[unsafe(no_mangle)]
pub extern "system" fn get_speaker_flags(ctx: *const LouderContext, speaker: LouderName) -> SpeakerFlags {
  todo!()
}

/// Iterate the speakers that `speaker` listens to.
///
/// Pass `-1` as `start` to begin, then keep passing the previous result until
/// the iteration ends.
#[unsafe(no_mangle)]
pub extern "system" fn next_speaker_listening(ctx: *const LouderContext, speaker: LouderName, start: LouderName) -> LouderName {
  todo!()
}

/// Get the variant type of a speaker's additional data under `key`.
#[unsafe(no_mangle)]
pub extern "system" fn get_speaker_additional_type(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, key: LouderName) -> LouderVariantType {
  todo!()
}

/// Get a speaker's additional data under `key` as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn get_speaker_additional_int(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, key: LouderName) -> i32 {
  todo!()
}

/// Get a speaker's additional data under `key` as a float.
#[unsafe(no_mangle)]
pub extern "system" fn get_speaker_additional_float(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, key: LouderName) -> f32 {
  todo!()
}

/// Get a speaker's additional data under `key` as a name.
#[unsafe(no_mangle)]
pub extern "system" fn get_speaker_additional_string(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, key: LouderName) -> LouderName {
  todo!()
}

/// Get the channel of an actor.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_channel(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> LouderName {
  todo!()
}

/// Get the position of an actor.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_position(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> LouderPosition {
  todo!()
}

/// Iterate the speakers an actor speaks as.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_speak_as(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  todo!()
}

/// Iterate the base speakers an actor listens to.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_listening_base(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  todo!()
}

/// Iterate the speakers an actor listens to.
///
/// Includes the actors' `listen_to` speakers plus the listening speakers of the
/// speakers it speaks as.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_listening(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  todo!()
}

/// Get the flags of an actor.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_flags(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> ActorFlags {
  todo!()
}

/// Get the priority of an actor.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_priority(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> i32 {
  todo!()
}

/// Get the variant type of an actor's additional data under `key`.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_additional_type(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, key: LouderName) -> LouderVariantType {
  todo!()
}

/// Get an actor's additional data under `key` as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_additional_int(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, key: LouderName) -> i32 {
  todo!()
}

/// Get an actor's additional data under `key` as a float.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_additional_float(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, key: LouderName) -> f32 {
  todo!()
}

/// Get an actor's additional data under `key` as a string.
#[unsafe(no_mangle)]
pub extern "system" fn get_actor_additional_string(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, key: LouderName) -> LouderName {
  todo!()
}

/// Iterate the actors in a channel.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_in_channel(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors in a channel at a specific position.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_in_channel_and_position(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors speaking as a speaker.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_speaking_as(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors listening to a speaker.
///
/// Includes `listen_to` and listening actors derived from the listening
/// speakers of the speaker's `speak_as`.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_listening_to(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors matching certain flags.
///
/// `exact` nonzero requires the actor flags to match exactly.
#[unsafe(no_mangle)]
pub extern "system" fn next_actor_from_flags(ctx: *const LouderContext, conversation: LouderName, include: ActorFlags, exact: i32, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the tags of a position.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_tag(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  todo!()
}

/// Get the speaker of a hub position.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_speaker(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the text of a hub as a name.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_text(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the number of choices on a hub from an actor's perspective.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_choice_count(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor) -> i32 {
  todo!()
}

/// Get the text of a choice on a hub as a name.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_choice_text(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> LouderName {
  todo!()
}

/// Get the position a choice on a hub links to.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_choice_link(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> i32 {
  todo!()
}

/// Get the expressions associated with a hub.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_expressions(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the callbacks associated with a hub.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_callbacks(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the flags of a hub.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_flags(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> HubFlags {
  todo!()
}

/// Iterate the recipients of a hub.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_recipient(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  todo!()
}

/// Get the channel of a hub.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_channel(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the priority of a hub.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_priority(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> i32 {
  todo!()
}

/// Get the variant type of a hub's additional data under `key`.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_additional_type(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, key: LouderName) -> LouderVariantType {
  todo!()
}

/// Get a hub's additional data under `key` as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_additional_int(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, key: LouderName) -> i32 {
  todo!()
}

/// Get a hub's additional data under `key` as a float.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_additional_float(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, key: LouderName) -> f32 {
  todo!()
}

/// Get a hub's additional data string under `key`.
#[unsafe(no_mangle)]
pub extern "system" fn get_hub_additional_string(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, key: LouderName) -> LouderName {
  todo!()
}

/// Iterate the hubs in a channel.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_by_channel(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs spoken by a speaker.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_spoken_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs heard by a speaker.
///
/// Uses the speakers' listening speakers to search for hubs that may appear to
/// actors speaking as this speaker.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_heard_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs with a priority within `[priority_min, priority_max]`.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_by_priority(ctx: *const LouderContext, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs containing all (or exactly, if `exact` is nonzero) the given
/// tags.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_from_tags(ctx: *const LouderContext, conversation: LouderName, include: *const LouderName, include_count: i32, exact: i32, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs matching certain flags.
///
/// `exact` nonzero requires the hub flags to match exactly.
#[unsafe(no_mangle)]
pub extern "system" fn next_hub_from_flags(ctx: *const LouderContext, conversation: LouderName, include: HubFlags, exact: i32, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Get the variant type of a story state value.
#[unsafe(no_mangle)]
pub extern "system" fn get_story_state_type(ctx: *const LouderContext, conversation: LouderName, state: LouderName) -> LouderVariantType {
  todo!()
}

/// Get a story state value as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn get_story_state_int(ctx: *const LouderContext, conversation: LouderName, state: LouderName) -> i32 {
  todo!()
}

/// Get a story state value as a float.
#[unsafe(no_mangle)]
pub extern "system" fn get_story_state_float(ctx: *const LouderContext, conversation: LouderName, state: LouderName) -> f32 {
  todo!()
}

/// Get a story state string.
#[unsafe(no_mangle)]
pub extern "system" fn get_story_state_string(ctx: *const LouderContext, conversation: LouderName, state: LouderName) -> LouderName {
  todo!()
}

/// Serialize the story state in JSON into `buffer`.
#[unsafe(no_mangle)]
pub extern "system" fn read_state_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> i32 {
  todo!()
}

/// Serialize the story state, as well as the positions and details of all
/// actors, in JSON into `buffer`.
#[unsafe(no_mangle)]
pub extern "system" fn read_snapshot_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> i32 {
  todo!()
}

/// Initialize the engine and return a new context.
///
/// Initializes the event queue and ask cache. Release the returned handle with
/// [`deinit_louder`].
#[unsafe(no_mangle)]
pub extern "system" fn init_louder() -> *mut LouderContext {
  Box::into_raw(Box::new(LouderContext::new()))
}

/// Remove all conversations, speaker objects, and the event queue.
#[unsafe(no_mangle)]
pub extern "system" fn deinit_louder(ctx: *mut LouderContext) -> i32 {
  if ctx.is_null() {
    return LOUDER_ERROR_NOT_FOUND;
  }
  unsafe {
    Box::from_raw(ctx); //allegedly this auto-destroys.
  }
  LOUDER_SUCCESS
}

/// Load a conversation from a JSON document into `buffer`.
///
/// - The JSON is expected in array form; an error is returned otherwise.
/// - Standard JSON syntax errors are returned.
/// - Standard properties are left default or read from matching JSON fields.
/// - Additional JSON fields are treated as additional data on hubs.
/// - An error is returned if `conversation` is already occupied by another loaded
///   conversation.
/// - Errors relating to individual objects, such as the duplicate id flag, are
///   emitted as events with the invalid hub's position available via
///   `get_event_position(index, EventPosition::Source)`.
#[unsafe(no_mangle)]
pub extern "system" fn load_conversation_json(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> LoadError {
  todo!()
}

/// Remove a conversation and all associated actors and objects.
///
/// Returns an error if the conversation name was not loaded.
#[unsafe(no_mangle)]
pub extern "system" fn remove_conversation(ctx: *mut LouderContext, conversation: LouderName) -> i32 {
  todo!()
}

/// Load the conversation config from a JSON document into `buffer`.
///
/// - Uses the same error scheme and data format as [`load_conversation_json`],
///   but with speakers as the target objects instead of hubs.
/// - The config is a singleton as opposed to a named conversation instance.
/// - Objects are speakers and their configurations.
#[unsafe(no_mangle)]
pub extern "system" fn load_conversation_config(ctx: *mut LouderContext, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> LoadError {
  todo!()
}

/// Remove all speakers in the config.
#[unsafe(no_mangle)]
pub extern "system" fn clear_config(ctx: *mut LouderContext) -> i32 {
  todo!()
}

/// Add an actor to a conversation and return the integer id-ing it.
#[unsafe(no_mangle)]
pub extern "system" fn add_actor(ctx: *mut LouderContext, conversation: LouderName, start_position: LouderPosition, flags: ActorFlags, speak_as: *const LouderName, speak_as_count: i32, listen_to: *const LouderName, listen_to_count: i32) -> i32 {
  todo!()
}

/// Remove an actor from a conversation.
#[unsafe(no_mangle)]
pub extern "system" fn remove_actor(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor) -> i32 {
  todo!()
}

/// Select a choice on an actor's current hub.
#[unsafe(no_mangle)]
pub extern "system" fn select_choice(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, choice_index: i32) -> i32 {
  todo!()
}

/// Move an actor to another position as if it had moved through the network.
///
/// - `fake_priority` is unused if `0`, otherwise overrides the priority of the
///   destination.
/// - `fake_channel` is unused if `-1`, otherwise overrides the channel of the
///   destination.
#[unsafe(no_mangle)]
pub extern "system" fn force_warp(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, target_position: LouderPosition, fake_channel: LouderName, fake_priority: i32) -> i32 {
  todo!()
}

/// Move an actor to another position directly.
///
/// - `fake_priority` is unused if `0`, otherwise overrides the priority of the
///   destination.
/// - `fake_channel` is unused if `-1`, otherwise overrides the channel of the
///   destination.
#[unsafe(no_mangle)]
pub extern "system" fn force_position(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, target_position: LouderPosition, fake_channel: LouderName, fake_priority: i32) -> i32 {
  todo!()
}

/// Set a story state value as an integer.
#[unsafe(no_mangle)]
pub extern "system" fn set_story_state_int(ctx: *mut LouderContext, conversation: LouderName, key: LouderName, value: i32) -> i32 {
  todo!()
}

/// Set a story state value as a float.
#[unsafe(no_mangle)]
pub extern "system" fn set_story_state_float(ctx: *mut LouderContext, conversation: LouderName, key: LouderName, value: f32) -> i32 {
  todo!()
}

/// Set a story state value as a string.
#[unsafe(no_mangle)]
pub extern "system" fn set_story_state_string(ctx: *mut LouderContext, conversation: LouderName, key: LouderName, buffer: *const u8, size: usize, encoding: LouderEncoding) -> i32 {
  todo!()
}