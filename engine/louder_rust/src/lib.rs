

mod test_linkage;

pub mod louder;

use std::ops::Index;
use std::usize;

use louder::actor::*;
use louder::callback::*;
use louder::context::*;
use louder::conversation::*;
use louder::event::*;
use louder::expression::*;
use louder::hub::*;
use louder::json_loader::*;
use louder::object::*;
use louder::name::*;
use louder::simulator::*;
use louder::speaker::*;

/// An error code returned by most operations.
///
/// - [`LouderError::Ok`]: no error.
/// - [`LouderError::Missing`]: the requested object does not exist.
#[repr(usize)]
pub enum LouderError {
  Ok = 0,
  Missing,
}

/// Key passed to [`crate::get_event_position`].
///
/// Each variant names a [`LouderPosition`] field of an event's data.
#[repr(C)]
pub enum EventPosition {
  /// Get the position the actor of this event's motion starts from.
  Source = 0,
  /// Get the position the actor of this event's motion moves to.
  Target = 1,
  /// Get the position this event's choice appearance is for.
  ChoiceAppearance = 2,
}

/// Custom Option type exposed to the C FFI
#[repr(C)]
pub enum LouderOption<T> {
  None,
  Some(T)
}


#[unsafe(no_mangle)]
extern "C" fn test_linkage() -> i32 {
  test_linkage::test_linkage::test_linkage()
}

/// Get the id of a string.
///
/// Returns [`LOUDER_NAME_NONE`] if the string was never declared.
///
/// - `buffer`: the string bytes to look up.
/// - `max_size`: the maximum size of `buffer` in bytes.
#[unsafe(no_mangle)]
pub extern "C" fn get_named_id(ctx: *const LouderContext, buffer: *const u8, max_size: usize) -> LouderName {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_NAME_NONE;
  }
  
  let c = LouderNameContent {
    bytes: unsafe { std::slice::from_raw_parts(buffer, max_size).to_vec() },
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
pub extern "C" fn declare_name(ctx: *mut LouderContext, buffer: *const u8, max_size: usize) -> LouderName {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_NAME_NONE;
  }

  let c = LouderNameContent {
    bytes: unsafe { std::slice::from_raw_parts(buffer, max_size).to_vec() },
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
pub extern "C" fn read_name(ctx: *const LouderContext, name: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> usize {
  if ctx.is_null() || buffer.is_null() {
    return 0;
  }

  let name_context = unsafe { (*ctx).get_name_content(&name) };
  if name_context.is_none() {
    return 0;
  }

  let name_context = name_context.unwrap();

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
pub extern "C" fn get_event(ctx: *const LouderContext, index: usize) -> LouderOption<LouderEvent> {
  if ctx.is_null() {
    return LouderOption::None
  }

  let e : Option<&LouderEvent> = unsafe {
    (*ctx).get_event(index)
  };

  match e {
    Some(e) => LouderOption::Some(e.clone()),
    None => LouderOption::None
  }
}

/// Flush all events from the queue.
///
/// Returns the number of events flushed. Poll events with [`is_event`] until it
/// stops returning an event, then call this.
#[unsafe(no_mangle)]
pub extern "C" fn flush_events(ctx: *mut LouderContext) {
  if ctx.is_null() {
    return;
  }

  unsafe {
    <LouderContext as EventCache<LouderEvent>>::clear_events(&mut *ctx);
  }
}

/// Push an event onto the event queue, used for testing.
///
/// The event's data is built from the `code` and the argument that matches it;
/// arguments for other variants are ignored. Returns the index of the pushed
/// event, or [`LOUDER_ERROR_NOT_FOUND`] if `ctx` is null.
#[unsafe(no_mangle)]
pub extern "C" fn test_push_event(
  ctx: *mut LouderContext,
  event: *const LouderEvent,
) -> LouderError {
  if ctx.is_null() || event.is_null() {
    return LouderError::Missing;
  }

  unsafe {
    (*ctx).push_event((*event).clone());
  }
  
  LouderError::Ok
}


/// Check whether an ask exists at `index`.
#[unsafe(no_mangle)]
pub extern "C" fn get_ask(ctx: *const LouderContext, index: usize) -> LouderOption<LouderAsk> {
  if ctx.is_null() {
    return LouderOption::None
  }

  let a : Option<&LouderAsk> = unsafe {
    (*ctx).get_event(index)
  };

  match a {
    Some(a) => LouderOption::Some(a.clone()),
    None => LouderOption::None
  }
}

/// Push an ask onto the ask cache, used for testing.
///
/// Returns the index of the pushed ask, or [`LOUDER_ERROR_NOT_FOUND`] if `ctx`
/// is null.
#[unsafe(no_mangle)]
pub extern "C" fn push_ask(
  ctx: *mut LouderContext,
  ask: *const LouderAsk
) -> LouderError {
  if ctx.is_null() || ask.is_null() {
    return LouderError::Missing;
  }

  unsafe {
    (*ctx).push_ask((*ask).clone());
  }

  LouderError::Ok
}


/// Get the flags of a speaker.
#[unsafe(no_mangle)]
pub extern "C" fn get_speaker_flags(ctx: *const LouderContext, speaker: LouderName) -> SpeakerFlags {
  if ctx.is_null() {
    return SpeakerFlags::NONE;
  }

  if let Some(s) = unsafe { (*ctx).get_speaker(speaker) } {
    return s.flags.clone();
  }

  return SpeakerFlags::NONE;
}

/// Iterate the speakers that `speaker` listens to.
///
/// Pass [`LOUDER_NAME_NONE`] as `start` to begin, then keep passing the previous result until
/// the iteration ends.
#[unsafe(no_mangle)]
pub extern "C" fn get_next_speaker_listen_to(ctx: *const LouderContext, speaker: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  if let Some(s) = unsafe { (*ctx).get_speaker(speaker) } {
    let listen_to = &s.listen_to;
    if listen_to.is_empty() {
      return LOUDER_NAME_NONE;
    }

    if let Some(index) = listen_to.iter().position(|l| l.clone() == speaker) { 
      if index + 1 < listen_to.len() {
        return listen_to[index + 1];
      }
    }

    return listen_to[0];
  }

  return LOUDER_NAME_NONE;
}

/// Get a speaker's additional data under `key`.
#[unsafe(no_mangle)]
pub extern "C" fn get_speaker_additional(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, key: LouderName) -> LouderVariant {
  todo!()
}

/// Get the channel of an actor.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_channel(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> LouderName {
  todo!()
}

/// Get the position of an actor.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_position(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> LouderPosition {
  todo!()
}

/// Iterate the speakers an actor speaks as.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_speak_as(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  todo!()
}

/// Iterate the base speakers an actor listens to.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_listening_base(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  todo!()
}

/// Iterate the speakers an actor listens to.
///
/// Includes the actors' `listen_to` speakers plus the listening speakers of the
/// speakers it speaks as.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_listening(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  todo!()
}

/// Get the flags of an actor.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_flags(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> ActorFlags {
  todo!()
}

/// Get the priority of an actor.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_priority(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> i32 {
  todo!()
}

/// Get an actor's additional data under `key`.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_additional(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, key: LouderName) -> LouderVariant {
  todo!()
}

/// Iterate the actors in a channel.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_in_channel(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors in a channel at a specific position.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_in_channel_and_position(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors speaking as a speaker.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_speaking_as(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors listening to a speaker.
///
/// Includes `listen_to` and listening actors derived from the listening
/// speakers of the speaker's `speak_as`.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_listening_to(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the actors matching certain flags.
///
/// `exact` nonzero requires the actor flags to match exactly.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_from_flags(ctx: *const LouderContext, conversation: LouderName, include: ActorFlags, exact: i32, start: LouderActor) -> LouderActor {
  todo!()
}

/// Iterate the tags of a position.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_tag(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  todo!()
}

/// Get the speaker of a hub position.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_speaker(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the text of a hub as a name.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_text(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the number of choices on a hub from an actor's perspective.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_count(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor) -> i32 {
  todo!()
}

/// Get the text of a choice on a hub as a name.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_text(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> LouderName {
  todo!()
}

/// Get the position a choice on a hub links to.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_link(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> i32 {
  todo!()
}

/// Get the expressions associated with a hub.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_expressions(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the callbacks associated with a hub.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_callbacks(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the flags of a hub.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_flags(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> HubFlags {
  todo!()
}

/// Iterate the recipients of a hub.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_recipient(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  todo!()
}

/// Get the channel of a hub.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_channel(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  todo!()
}

/// Get the priority of a hub.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_priority(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> i32 {
  todo!()
}

/// Get a hub's additional data under `key`.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_additional(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, key: LouderName) -> LouderVariant {
  todo!()
}

/// Iterate the hubs in a channel.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_by_channel(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs spoken by a speaker.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_spoken_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs heard by a speaker.
///
/// Uses the speakers' listening speakers to search for hubs that may appear to
/// actors speaking as this speaker.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_heard_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs with a priority within `[priority_min, priority_max]`.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_by_priority(ctx: *const LouderContext, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs containing all (or exactly, if `exact` is nonzero) the given
/// tags.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_from_tags(ctx: *const LouderContext, conversation: LouderName, include: *const LouderName, include_count: i32, exact: i32, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Iterate the hubs matching certain flags.
///
/// `exact` nonzero requires the hub flags to match exactly.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_from_flags(ctx: *const LouderContext, conversation: LouderName, include: HubFlags, exact: i32, start: LouderPosition) -> LouderPosition {
  todo!()
}

/// Get a story state value under `state`.
#[unsafe(no_mangle)]
pub extern "C" fn get_story_state(ctx: *const LouderContext, conversation: LouderName, state: LouderName) -> LouderVariant {
  todo!()
}

/// Serialize the story state in JSON into `buffer`.
#[unsafe(no_mangle)]
pub extern "C" fn read_state_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> i32 {
  todo!()
}

/// Serialize the story state, as well as the positions and details of all
/// actors, in JSON into `buffer`.
#[unsafe(no_mangle)]
pub extern "C" fn read_snapshot_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> i32 {
  todo!()
}

/// Initialize the engine and return a new context.
///
/// Initializes the event queue and ask cache. Release the returned handle with
/// [`deinit_louder`].
#[unsafe(no_mangle)]
pub extern "C" fn init_louder() -> *mut LouderContext {
  Box::into_raw(Box::new(LouderContext::new()))
}

/// Remove all conversations, speaker objects, and the event queue.
#[unsafe(no_mangle)]
pub extern "C" fn deinit_louder(ctx: *mut LouderContext) -> LouderError {
  if ctx.is_null() {
    return LouderError::Missing;
  }
  unsafe {
    Box::from_raw(ctx); //allegedly this auto-destroys.
  }
  LouderError::Ok
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
pub extern "C" fn load_conversation_json(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> LoadError {
  todo!()
}

/// Remove a conversation and all associated actors and objects.
///
/// Returns an error if the conversation name was not loaded.
#[unsafe(no_mangle)]
pub extern "C" fn remove_conversation(ctx: *mut LouderContext, conversation: LouderName) -> i32 {
  todo!()
}

/// Load the conversation config from a JSON document into `buffer`.
///
/// - Uses the same error scheme and data format as [`load_conversation_json`],
///   but with speakers as the target objects instead of hubs.
/// - The config is a singleton as opposed to a named conversation instance.
/// - Objects are speakers and their configurations.
#[unsafe(no_mangle)]
pub extern "C" fn load_conversation_config(ctx: *mut LouderContext, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> LoadError {
  todo!()
}

/// Remove all speakers in the config.
#[unsafe(no_mangle)]
pub extern "C" fn clear_config(ctx: *mut LouderContext) -> i32 {
  todo!()
}

/// Add an actor to a conversation and return the integer id-ing it.
#[unsafe(no_mangle)]
pub extern "C" fn add_actor(ctx: *mut LouderContext, conversation: LouderName, start_position: LouderPosition, flags: ActorFlags, speak_as: *const LouderName, speak_as_count: i32, listen_to: *const LouderName, listen_to_count: i32) -> i32 {
  todo!()
}

/// Remove an actor from a conversation.
#[unsafe(no_mangle)]
pub extern "C" fn remove_actor(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor) -> i32 {
  todo!()
}

/// Select a choice on an actor's current hub.
#[unsafe(no_mangle)]
pub extern "C" fn select_choice(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, choice_index: i32) -> i32 {
  todo!()
}

/// Move an actor to another position as if it had moved through the network.
///
/// - `fake_priority` is unused if `0`, otherwise overrides the priority of the
///   destination.
/// - `fake_channel` is unused if `-1`, otherwise overrides the channel of the
///   destination.
#[unsafe(no_mangle)]
pub extern "C" fn force_warp(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, target_position: LouderPosition, fake_channel: LouderName, fake_priority: i32) -> i32 {
  todo!()
}

/// Move an actor to another position directly.
///
/// - `fake_priority` is unused if `0`, otherwise overrides the priority of the
///   destination.
/// - `fake_channel` is unused if `-1`, otherwise overrides the channel of the
///   destination.
#[unsafe(no_mangle)]
pub extern "C" fn force_position(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, target_position: LouderPosition, fake_channel: LouderName, fake_priority: i32) -> i32 {
  todo!()
}

/// Set a story state value as an integer.
#[unsafe(no_mangle)]
pub extern "C" fn set_story_state(ctx: *mut LouderContext, conversation: LouderName, key: LouderName, value: LouderVariant) {
  todo!()
}