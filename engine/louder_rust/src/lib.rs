//The primary engine interface, defined in docs/engine/index.md

mod test_linkage;

pub mod louder;

use louder::actor::*;
use louder::callback::*;
use louder::context::*;
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

/// Which position of an event's data to read.
///
/// Every position an event can refer to is carried inside a [`Motion`], apart
/// from the hub a choice appeared on, which is in a [`ChoiceAppearance`]. Match
/// on the event's [`EventData`] and read the field this names.
#[repr(C)]
pub enum EventPosition {
  /// Where the actor of this event's motion starts from.
  ///
  /// [`Motion::source_position`].
  Source = 0,
  /// Where the actor of this event's motion moves to.
  ///
  /// [`Motion::target_position`].
  Target = 1,
  /// Which hub this event's choice appeared on.
  ///
  /// [`ChoiceAppearance::position`].
  ChoiceAppearance = 2,
}

/// Custom Option type exposed to the C FFI
#[repr(C)]
pub enum LouderOption<T> {
  None,
  Some(T)
}

// Constructors for the tagged unions above.
//
// cbindgen lowers each of these enums to a C++ struct of a tag and an anonymous
// union of bodies, which the FFI offers no way to fill in: a caller that wants
// to build an EventData has to know the tag order and the name of the body
// member, and gets no compiler help if either is wrong. Building the value here
// and returning it keeps that layout inside the engine, so any host can
// construct one of these without repeating it.
//
// A union is only produced here, never consumed: nothing in the engine takes one
// as an argument, so there is no constructor for a LouderOption::None. Rust
// generics cannot cross the FFI, so LouderOption has one constructor per type
// the engine hands back. These are calls rather than constants, so a union
// cannot be built in a constexpr or a static initialiser.

/// Build a [`LouderVariant::Int`].
#[unsafe(no_mangle)]
pub extern "C" fn make_variant_int(value: i32) -> LouderVariant {
  LouderVariant::from_i32(value)
}

/// Build a [`LouderVariant::Float`].
#[unsafe(no_mangle)]
pub extern "C" fn make_variant_float(value: f32) -> LouderVariant {
  LouderVariant::from_f32(value)
}

/// Build a [`LouderVariant::String`], naming the string rather than carrying it.
#[unsafe(no_mangle)]
pub extern "C" fn make_variant_string(value: LouderName) -> LouderVariant {
  LouderVariant::from_name(value)
}

/// Build a boolean as a [`LouderVariant::Int`] of 1 or 0.
#[unsafe(no_mangle)]
pub extern "C" fn make_variant_bool(value: bool) -> LouderVariant {
  LouderVariant::from_bool(value)
}

/// Build the [`LouderVariant::Int`] of 0 that an unwritten variable reads as.
#[unsafe(no_mangle)]
pub extern "C" fn make_variant_zero() -> LouderVariant {
  LouderVariant::zero()
}

/// Build a [`LouderOption::Some`] holding an event.
#[unsafe(no_mangle)]
pub extern "C" fn make_some_event(value: LouderEvent) -> LouderOption<LouderEvent> {
  LouderOption::Some(value)
}

/// Build a [`LouderOption::Some`] holding an ask.
#[unsafe(no_mangle)]
pub extern "C" fn make_some_ask(value: LouderAsk) -> LouderOption<LouderAsk> {
  LouderOption::Some(value)
}

/// Build the [`EventData::None`] variant, which carries nothing.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_none() -> EventData {
  EventData::None
}

/// Build an [`EventData::RegularMotion`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_regular_motion(motion: Motion) -> EventData {
  EventData::RegularMotion(motion)
}

/// Build an [`EventData::Warp`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_warp(motion: Motion) -> EventData {
  EventData::Warp(motion)
}

/// Build an [`EventData::ForceMotion`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_force_motion(motion: Motion) -> EventData {
  EventData::ForceMotion(motion)
}

/// Build an [`EventData::ChoiceAppearance`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_choice_appearance(appearance: ChoiceAppearance) -> EventData {
  EventData::ChoiceAppearance(appearance)
}

/// Build an [`EventData::ChannelChange`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_channel_change() -> EventData {
  EventData::ChannelChange
}

/// Build an [`EventData::AdditionalDataChange`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_additional_data_change(change: AdditionalDataChange) -> EventData {
  EventData::AdditionalDataChange(change)
}

/// Build an [`EventData::SpeakerChange`], naming the speaker.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_speaker_change(speaker: LouderName) -> EventData {
  EventData::SpeakerChange(speaker)
}

/// Build an [`EventData::TextChange`], naming the text.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_text_change(text: LouderName) -> EventData {
  EventData::TextChange(text)
}

/// Build an [`EventData::Statement`], naming the expression.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_statement(expression: LouderExpression) -> EventData {
  EventData::Statement(expression)
}

/// Build an [`EventData::Condition`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_condition(outcome: ConditionOutcome) -> EventData {
  EventData::Condition(outcome)
}

/// Build an [`EventData::Callback`], naming the callback.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_callback(callback: LouderCallback) -> EventData {
  EventData::Callback(callback)
}

/// Build an [`EventData::ActorAdded`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_actor_added() -> EventData {
  EventData::ActorAdded
}

/// Build an [`EventData::ActorRemoved`].
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_actor_removed() -> EventData {
  EventData::ActorRemoved
}

/// Build an [`EventData::SpeakerAdded`], naming the speaker.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_speaker_added(speaker: LouderName) -> EventData {
  EventData::SpeakerAdded(speaker)
}

/// Build an [`EventData::SpeakerRemoved`], naming the speaker.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_speaker_removed(speaker: LouderName) -> EventData {
  EventData::SpeakerRemoved(speaker)
}

/// Build an [`EventData::ConversationAdded`], naming the conversation.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_conversation_added(conversation: LouderName) -> EventData {
  EventData::ConversationAdded(conversation)
}

/// Build an [`EventData::ConversationRemoved`], naming the conversation.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_conversation_removed(conversation: LouderName) -> EventData {
  EventData::ConversationRemoved(conversation)
}

/// The blank starting value for an iterator over positions.
///
/// Positions are chosen by the editor, and the engine requires them to be
/// non-negative so that this value is unambiguous.
const BLANK_POSITION: LouderPosition = -1;

/// Copy `text` into `buffer`, truncating if needed, and null terminate.
///
/// Returns the number of bytes written, excluding the terminator.
fn write_text(buffer: *mut u8, max_size: usize, text: &str) -> usize {
  if buffer.is_null() {
    return 0;
  }

  let bytes = text.as_bytes();
  let length = bytes.len().min(max_size);

  unsafe {
    if length > 0 {
      std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, length);
    }
    *buffer.add(length) = 0;
  }

  length
}

/// Copy a possibly truncated JSON document into `buffer`.
///
/// Returns the number of bytes written excluding the terminator, or 0 if the
/// buffer was null. A document longer than `max_size` is truncated and the
/// result will not parse; the caller is responsible for sizing the buffer.
fn write_json(buffer: *mut u8, max_size: usize, document: &str) -> usize {
  write_text(buffer, max_size, document)
}

/// Escape `text` as a JSON string, quotes included.
fn json_quote(text: &str) -> String {
  let mut out = String::with_capacity(text.len() + 2);
  out.push('"');
  for c in text.chars() {
    match c {
      '"' => out.push_str("\\\""),
      '\\' => out.push_str("\\\\"),
      '\n' => out.push_str("\\n"),
      '\r' => out.push_str("\\r"),
      '\t' => out.push_str("\\t"),
      c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
      c => out.push(c),
    }
  }
  out.push('"');
  out
}

/// Render a variant as JSON, resolving strings against the context.
fn json_variant(context: &LouderContext, value: &LouderVariant) -> String {
  match value {
    LouderVariant::Int(i) => i.to_string(),
    LouderVariant::Float(f) => {
      // Whole floats print without a trailing `.0` so that round-tripping an
      // integer stored as a float does not change how it reads.
      if f.fract() == 0.0 && f.is_finite() {
        format!("{}", *f as i64)
      } else {
        f.to_string()
      }
    }
    LouderVariant::String(name) => match context.resolve_name(*name) {
      Some(text) => json_quote(text),
      None => json_quote(""),
    },
  }
}

/// Render a set of name/variant pairs as a JSON object, in ascending key order.
fn json_object(context: &LouderContext, keys: &[LouderName], values: impl Fn(LouderName) -> LouderVariant) -> String {
  let mut out = String::from("{");
  for (index, key) in keys.iter().enumerate() {
    if index > 0 {
      out.push(',');
    }
    let name = match context.resolve_name(*key) {
      Some(text) => json_quote(text),
      None => json_quote(""),
    };
    out.push_str(&format!("{}:{}", name, json_variant(context, &values(*key))));
  }
  out.push('}');
  out
}

/// The next name after `start` in a list, or [`LOUDER_NAME_NONE`].
///
/// Name ids are content hashes and carry no ordering, so this walks the list
/// by position rather than comparing ids: `start` names the element already
/// seen, and the element after it is the next one. A blank `start` begins at
/// the front, and a `start` that is not in the list ends the iteration.
fn next_name_after(items: &[LouderName], start: LouderName) -> LouderName {
  if start == LOUDER_NAME_NONE {
    return items.first().copied().unwrap_or(LOUDER_NAME_NONE);
  }

  match items.iter().position(|item| *item == start) {
    Some(index) => items.get(index + 1).copied().unwrap_or(LOUDER_NAME_NONE),
    None => LOUDER_NAME_NONE,
  }
}

/// The next actor id after `start`, or [`LOUDER_ACTOR_NONE`].
fn next_actor_after(items: &[LouderActor], start: LouderActor) -> LouderActor {
  for item in items {
    if *item != start && (start == LOUDER_ACTOR_NONE || *item > start) {
      return *item;
    }
  }
  LOUDER_ACTOR_NONE
}

/// The next position after `start`, or [`BLANK_POSITION`].
fn next_position_after(items: &[LouderPosition], start: LouderPosition) -> LouderPosition {
  for item in items {
    if *item != start && (start == BLANK_POSITION || *item > start) {
      return *item;
    }
  }
  BLANK_POSITION
}

/// The ids of the actors matching `predicate`, in ascending order.
fn matching_actors<F>(context: &LouderContext, conversation: LouderName, predicate: F) -> Vec<LouderActor>
where
  F: Fn(&Actor) -> bool,
{
  context
    .actors(conversation)
    .iter()
    .filter(|entry| predicate(entry))
    .map(|entry| entry.id())
    .collect()
}

/// Whether `flags` satisfies a filter.
///
/// With `exact` set the flags must match exactly; otherwise every bit named in
/// `include` must be present.
fn flags_match(flags_bits: i32, include: i32, exact: i32) -> bool {
  if exact != 0 {
    return flags_bits == include;
  }
  (flags_bits & include) == include
}

#[unsafe(no_mangle)]
extern "C" fn test_linkage() -> i32 {
  test_linkage::test_linkage::test_linkage()
}

/// Get the id of a string.
///
/// Returns [`LOUDER_NAME_NONE`] if the string was never declared, or if either
/// pointer was null.
///
/// - `buffer`: the string bytes to look up.
/// - `max_size`: the number of bytes to read from `buffer`.
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
/// If the string was already declared, returns [`LOUDER_NAME_NONE`] and leaves
/// the existing id alone. Otherwise a new id is assigned and returned.
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
/// Writes 0 bytes and a terminator if no name is associated with the id, or if
/// either pointer was null. Only UTF-8 is supported; any other `encoding`
/// writes nothing. Returns the number of bytes written, excluding the
/// terminator.
#[unsafe(no_mangle)]
pub extern "C" fn read_name(ctx: *const LouderContext, name: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> usize {
  if ctx.is_null() || buffer.is_null() || encoding != LOUDER_ENCODING_UTF8 {
    return 0;
  }

  let name_context = unsafe { (*ctx).get_name_content(&name) };
  let Some(name_context) = name_context else {
    return write_text(buffer, max_size, "");
  };

  let text = name_context.as_str().unwrap_or("");
  write_text(buffer, max_size, text)
}

/// Get the event at `index`.
///
/// Returns [`LouderOption::None`] once `index` is past the end of the queue,
/// or if `ctx` is null. Poll from index 0 upwards until that happens, then
/// call [`flush_events`].
#[unsafe(no_mangle)]
pub extern "C" fn get_event(ctx: *const LouderContext, index: usize) -> LouderOption<LouderEvent> {
  if ctx.is_null() {
    return LouderOption::None
  }

  let e : Option<&LouderEvent> = unsafe {
    <LouderContext as EventCache<LouderEvent>>::get_event(&*ctx, index)
  };

  match e {
    Some(e) => LouderOption::Some(e.clone()),
    None => LouderOption::None
  }
}

/// Remove every queued event.
///
/// The queue is indexed from 0 again afterwards, so the next [`get_event`] with
/// index 0 reads the first event queued after this call.
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
/// The caller fills in the event themselves, so nothing about `code` is
/// interpreted here. Returns [`LouderError::Ok`] on success, or
/// [`LouderError::Missing`] if either pointer was null.
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

/// Get the ask at `index`.
///
/// Asks are not in a stable order, so read them until one comes back
/// [`LouderOption::None`] rather than assuming the queue only grows.
#[unsafe(no_mangle)]
pub extern "C" fn get_ask(ctx: *const LouderContext, index: usize) -> LouderOption<LouderAsk> {
  if ctx.is_null() {
    return LouderOption::None
  }

  let a = unsafe { (*ctx).ask(index).cloned() };

  match a {
    Some(a) => LouderOption::Some(a),
    None => LouderOption::None
  }
}

/// Push an ask onto the ask cache, used for testing.
///
/// Returns [`LouderError::Ok`] on success, or [`LouderError::Missing`] if
/// either pointer was null.
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

/// Answer the ask at `index` with `verdict`.
///
/// Answering marks the ask complete and moves it to the back of the cache, so
/// indexes shift: read [`get_ask`] again after each answer. Answered asks stay
/// readable but no longer accept answers, so a host can simply keep answering
/// until [`LouderError::Missing`] comes back and the cache has flushed itself.
///
/// Returns [`LouderError::Ok`] if the answer was recorded, or
/// [`LouderError::Missing`] if `index` was out of range or that ask had already
/// been answered.
#[unsafe(no_mangle)]
pub extern "C" fn answer_ask(ctx: *mut LouderContext, index: usize, verdict: i32) -> LouderError {
  if ctx.is_null() {
    return LouderError::Missing;
  }

  unsafe {
    if (*ctx).answer_ask(index, verdict) {
      LouderError::Ok
    } else {
      LouderError::Missing
    }
  }
}

/// Get the flags of a speaker.
///
/// A speaker that has not been declared reads as [`SpeakerFlags::empty`], which
/// is public by default, so a hub naming it still appears to everyone.
#[unsafe(no_mangle)]
pub extern "C" fn get_speaker_flags(ctx: *const LouderContext, speaker: LouderName) -> SpeakerFlags {
  if ctx.is_null() {
    return SpeakerFlags::empty();
  }

  unsafe {
    match (*ctx).get_speaker(speaker) {
      Some(s) => s.flags(),
      None => SpeakerFlags::empty(),
    }
  }
}

/// Iterate the speakers that `speaker` is listening to.
///
/// Pass [`LOUDER_NAME_NONE`] as `start` to begin, then pass the previous result
/// until [`LOUDER_NAME_NONE`] comes back. Returns [`LOUDER_NAME_NONE`] for an
/// unknown speaker.
#[unsafe(no_mangle)]
pub extern "C" fn get_next_speaker_listen_to(ctx: *const LouderContext, speaker: LouderName, start: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_speaker(speaker) {
      Some(s) => next_name_after(s.listening(), start),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get a speaker's additional data under `key`.
///
/// The speaker config is a singleton rather than a per-conversation object, so
/// this takes no conversation. Returns a zero variant if the speaker is
/// unknown or has nothing stored under `key`.
#[unsafe(no_mangle)]
pub extern "C" fn get_speaker_additional(ctx: *const LouderContext, speaker: LouderName, key: LouderName) -> LouderVariant {
  if ctx.is_null() {
    return LouderVariant::Int(0);
  }

  unsafe {
    match (*ctx).get_speaker(speaker).and_then(|s| s.additional_get(key)) {
      Some(value) => value.clone(),
      None => LouderVariant::Int(0),
    }
  }
}

/// Get the channel of an actor.
///
/// Actors with [`ActorFlags::SPECTATOR`] or [`ActorFlags::DEBUG`] are not part
/// of the flow and read as [`LOUDER_NAME_NONE`].
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_channel(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor) {
      Some(a) => a.channel(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the position of an actor.
///
/// Returns [`BLANK_POSITION`] if the actor does not exist.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_position(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> LouderPosition {
  if ctx.is_null() {
    return BLANK_POSITION;
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor) {
      Some(a) => a.position(),
      None => BLANK_POSITION,
    }
  }
}

/// Get the flags of an actor.
///
/// Returns [`ActorFlags::empty`] if the actor does not exist.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_flags(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> ActorFlags {
  if ctx.is_null() {
    return ActorFlags::empty();
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor) {
      Some(a) => a.flags(),
      None => ActorFlags::empty(),
    }
  }
}

/// Get the priority of an actor.
///
/// Actors outside the flow read as 0.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_priority(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor) -> i32 {
  if ctx.is_null() {
    return 0;
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor) {
      Some(a) => a.priority(),
      None => 0,
    }
  }
}

/// Get an actor's additional data under `key`.
///
/// Returns a zero variant if the actor does not exist or has nothing stored
/// under `key`.
#[unsafe(no_mangle)]
pub extern "C" fn get_actor_additional(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, key: LouderName) -> LouderVariant {
  if ctx.is_null() {
    return LouderVariant::Int(0);
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor).and_then(|a| a.additional_get(key)) {
      Some(value) => value.clone(),
      None => LouderVariant::Int(0),
    }
  }
}

/// Iterate the speakers an actor speaks as.
///
/// Pass [`LOUDER_NAME_NONE`] as `start` to begin, then pass the previous result
/// until [`LOUDER_NAME_NONE`] comes back.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_speak_as(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor) {
      Some(a) => next_name_after(a.speak_as(), start),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Iterate the base speakers an actor listens to.
///
/// These are the actor's own `listen_to` speakers, without the ones reachable
/// through who it speaks as. See [`next_actor_listening`] for the wider set.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_listening_base(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_actor(conversation, actor) {
      Some(a) => next_name_after(a.listen_to(), start),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Iterate the speakers an actor listens to.
///
/// Includes the actor's `listen_to` speakers plus the listening speakers of the
/// speakers it speaks as. Pass [`LOUDER_NAME_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_listening(ctx: *const LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  let speakers = unsafe { Simulator::speakers_heard_by(&*ctx, conversation, actor) };
  next_name_after(&speakers, start)
}

/// Iterate the actors in a channel.
///
/// Pass [`LOUDER_ACTOR_NONE`] as `start` to begin. Actors outside the flow have
/// no channel and are never returned.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_in_channel(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, start: LouderActor) -> LouderActor {
  if ctx.is_null() {
    return LOUDER_ACTOR_NONE;
  }

  unsafe {
    let actors = matching_actors(&*ctx, conversation, |entry| entry.channel() == channel);
    next_actor_after(&actors, start)
  }
}

/// Iterate the actors in a channel at a specific position.
///
/// Pass [`LOUDER_ACTOR_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_in_channel_and_position(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
  if ctx.is_null() {
    return LOUDER_ACTOR_NONE;
  }

  unsafe {
    let actors = matching_actors(&*ctx, conversation, |entry| {
      entry.channel() == channel && entry.position() == position
    });
    next_actor_after(&actors, start)
  }
}

/// Iterate the actors speaking as a speaker.
///
/// Pass [`LOUDER_ACTOR_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_speaking_as(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  if ctx.is_null() {
    return LOUDER_ACTOR_NONE;
  }

  unsafe {
    let actors = matching_actors(&*ctx, conversation, |entry| entry.speak_as().contains(&speaker));
    next_actor_after(&actors, start)
  }
}

/// Iterate the actors listening to a speaker.
///
/// This covers `listen_to` plus the listening actors derived from the listening
/// speakers of the speakers an actor speaks as. Pass [`LOUDER_ACTOR_NONE`] as
/// `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_listening_to(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  if ctx.is_null() {
    return LOUDER_ACTOR_NONE;
  }

  unsafe {
    let context = &*ctx;
    let actors = matching_actors(context, conversation, |entry| {
      Simulator::speakers_heard_by(context, conversation, entry.id()).contains(&speaker)
    });
    next_actor_after(&actors, start)
  }
}

/// Iterate the actors matching certain flags.
///
/// With `exact` nonzero the flags must match exactly; otherwise every bit in
/// `include` must be present. Pass [`LOUDER_ACTOR_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_from_flags(ctx: *const LouderContext, conversation: LouderName, include: ActorFlags, exact: i32, start: LouderActor) -> LouderActor {
  if ctx.is_null() {
    return LOUDER_ACTOR_NONE;
  }

  unsafe {
    let wanted = include.bits();
    let actors = matching_actors(&*ctx, conversation, |entry| {
      flags_match(entry.flags().bits(), wanted, exact)
    });
    next_actor_after(&actors, start)
  }
}

/// Iterate the tags of a position.
///
/// Pass [`LOUDER_NAME_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_tag(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => next_name_after(hub.tags(), start),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the speaker of a hub.
///
/// Returns [`LOUDER_NAME_NONE`] if the hub has no speaker, which means it is
/// visible to everyone, or if the hub does not exist.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_speaker(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.speaker(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the text of a hub as a name.
///
/// Returns [`LOUDER_NAME_NONE`] if the hub has no text or does not exist. Read
/// the string back with [`read_name`].
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_text(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.text_name(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the number of choices on a hub from an actor's perspective.
///
/// Only choices the actor could actually take are counted, so this applies the
/// visibility, recipient, and spoke rules. Returns 0 for an unknown hub or
/// actor.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_count(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor) -> i32 {
  if ctx.is_null() {
    return 0;
  }

  unsafe {
    Simulator::available_choices(&*ctx, conversation, actor_perspective, position).len() as i32
  }
}

/// Get the text of a choice on a hub as a name.
///
/// A choice has no text of its own, so this is the text of the hub it links to.
/// Returns [`LOUDER_NAME_NONE`] if the choice does not exist or is not
/// available to `actor_perspective`.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_text(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> LouderName {
  if ctx.is_null() || choice_index < 0 {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    let context = &*ctx;
    if !Simulator::choice_available(context, conversation, actor_perspective, position, choice_index as usize) {
      return LOUDER_NAME_NONE;
    }

    match context
      .get_conversation(conversation)
      .and_then(|c| c.hub(position))
      .and_then(|hub| hub.choice(choice_index as usize))
    {
      Some(choice) => choice.text_name(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the position a choice on a hub links to.
///
/// Returns [`BLANK_POSITION`] if the choice does not exist or is not available
/// to `actor_perspective`.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_link(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> i32 {
  if ctx.is_null() || choice_index < 0 {
    return BLANK_POSITION;
  }

  unsafe {
    let context = &*ctx;
    if !Simulator::choice_available(context, conversation, actor_perspective, position, choice_index as usize) {
      return BLANK_POSITION;
    }

    match context
      .get_conversation(conversation)
      .and_then(|c| c.hub(position))
      .and_then(|hub| hub.choice(choice_index as usize))
    {
      Some(choice) => choice.position(),
      None => BLANK_POSITION,
    }
  }
}

/// Get the expressions associated with a hub as a name.
///
/// The schema stores expressions newline separated; this returns the whole
/// block as one name, so the caller splits it to get individual expressions.
/// Returns [`LOUDER_NAME_NONE`] if the hub has none.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_expressions(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.expressions_joined(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the callbacks associated with a hub as a name.
///
/// The schema stores callbacks newline separated; this returns the whole block
/// as one name. Returns [`LOUDER_NAME_NONE`] if the hub has none.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_callbacks(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.callbacks_joined(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the flags of a hub.
///
/// Returns [`HubFlags::empty`] if the hub does not exist.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_flags(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> HubFlags {
  if ctx.is_null() {
    return HubFlags::empty();
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.flags(),
      None => HubFlags::empty(),
    }
  }
}

/// Iterate the recipients of a hub.
///
/// Returns [`LOUDER_NAME_NONE`] when the hub has no recipients, which means the
/// hub is not restricted. Pass [`LOUDER_NAME_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_recipient(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => next_name_after(hub.recipients(), start),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the channel of a hub.
///
/// Returns [`LOUDER_NAME_NONE`] if the hub does not change the channel, in
/// which case an actor keeps whatever channel it already had.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_channel(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderName {
  if ctx.is_null() {
    return LOUDER_NAME_NONE;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.channel(),
      None => LOUDER_NAME_NONE,
    }
  }
}

/// Get the priority of a hub.
///
/// Returns 0 if the hub does not exist, which is also the default priority.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_priority(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> i32 {
  if ctx.is_null() {
    return 0;
  }

  unsafe {
    match (*ctx).get_conversation(conversation).and_then(|c| c.hub(position)) {
      Some(hub) => hub.priority(),
      None => 0,
    }
  }
}

/// Get a hub's additional data under `key`.
///
/// Returns a zero variant if the hub does not exist or has nothing stored
/// under `key`.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_additional(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, key: LouderName) -> LouderVariant {
  if ctx.is_null() {
    return LouderVariant::Int(0);
  }

  unsafe {
    match (*ctx)
      .get_conversation(conversation)
      .and_then(|c| c.hub(position))
      .and_then(|hub| hub.additional_get(key))
    {
      Some(value) => value.clone(),
      None => LouderVariant::Int(0),
    }
  }
}

/// Iterate the hubs in a channel.
///
/// Hubs that name no channel belong to the default channel, so searching for it
/// also finds them. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_by_channel(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, start: LouderPosition) -> LouderPosition {
  if ctx.is_null() {
    return BLANK_POSITION;
  }

  unsafe {
    let context = &*ctx;
    let Some(conversation_ref) = context.get_conversation(conversation) else {
      return BLANK_POSITION;
    };

    let main = context.main_channel();
    let positions: Vec<LouderPosition> = conversation_ref
      .sorted_positions()
      .into_iter()
      .filter(|position| {
        conversation_ref
          .hub(*position)
          .map(|hub| {
            let hub_channel = hub.channel();
            if hub_channel == LOUDER_NAME_NONE {
              main == channel
            } else {
              hub_channel == channel
            }
          })
          .unwrap_or(false)
      })
      .collect();

    next_position_after(&positions, start)
  }
}

/// Iterate the hubs spoken by a speaker.
///
/// Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_spoken_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  if ctx.is_null() {
    return BLANK_POSITION;
  }

  unsafe {
    let Some(conversation_ref) = (*ctx).get_conversation(conversation) else {
      return BLANK_POSITION;
    };

    let positions: Vec<LouderPosition> = conversation_ref
      .sorted_positions()
      .into_iter()
      .filter(|position| {
        conversation_ref
          .hub(*position)
          .map(|hub| hub.speaker() == speaker)
          .unwrap_or(false)
      })
      .collect();

    next_position_after(&positions, start)
  }
}

/// Iterate the hubs heard by a speaker.
///
/// This searches the speaker's own listening speakers as well as the speaker
/// itself, which are the hubs that may appear to an actor speaking as this
/// speaker. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_heard_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  if ctx.is_null() {
    return BLANK_POSITION;
  }

  unsafe {
    let context = &*ctx;
    let Some(conversation_ref) = context.get_conversation(conversation) else {
      return BLANK_POSITION;
    };

    let mut heard = vec![speaker];
    if let Some(entry) = context.get_speaker(speaker) {
      heard.extend(entry.listening().iter().copied());
    }

    let positions: Vec<LouderPosition> = conversation_ref
      .sorted_positions()
      .into_iter()
      .filter(|position| {
        conversation_ref
          .hub(*position)
          .map(|hub| heard.contains(&hub.speaker()))
          .unwrap_or(false)
      })
      .collect();

    next_position_after(&positions, start)
  }
}

/// Iterate the hubs with a priority within `[priority_min, priority_max]`.
///
/// The bounds are inclusive. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_by_priority(ctx: *const LouderContext, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
  if ctx.is_null() {
    return BLANK_POSITION;
  }

  unsafe {
    let Some(conversation_ref) = (*ctx).get_conversation(conversation) else {
      return BLANK_POSITION;
    };

    let positions: Vec<LouderPosition> = conversation_ref
      .sorted_positions()
      .into_iter()
      .filter(|position| {
        conversation_ref
          .hub(*position)
          .map(|hub| {
            let priority = hub.priority();
            priority >= priority_min && priority <= priority_max
          })
          .unwrap_or(false)
      })
      .collect();

    next_position_after(&positions, start)
  }
}

/// Iterate the hubs containing all, or exactly, the given tags.
///
/// With `exact` nonzero the hub must carry exactly the listed tags; otherwise
/// carrying extra tags is fine. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_from_tags(ctx: *const LouderContext, conversation: LouderName, include: *const LouderName, include_count: i32, exact: i32, start: LouderPosition) -> LouderPosition {
  if ctx.is_null() || include_count < 0 || (include.is_null() && include_count > 0) {
    return BLANK_POSITION;
  }

  let wanted: Vec<LouderName> = if include_count == 0 {
    Vec::new()
  } else {
    unsafe { std::slice::from_raw_parts(include, include_count as usize) }.to_vec()
  };

  unsafe {
    let Some(conversation_ref) = (*ctx).get_conversation(conversation) else {
      return BLANK_POSITION;
    };

    let positions: Vec<LouderPosition> = conversation_ref
      .sorted_positions()
      .into_iter()
      .filter(|position| {
        conversation_ref
          .hub(*position)
          .map(|hub| {
            let tags = hub.tags();
            let has_all = wanted.iter().all(|tag| tags.contains(tag));
            if exact == 0 {
              return has_all;
            }
            has_all && tags.len() == wanted.len()
          })
          .unwrap_or(false)
      })
      .collect();

    next_position_after(&positions, start)
  }
}

/// Iterate the hubs matching certain flags.
///
/// With `exact` nonzero the flags must match exactly; otherwise every bit in
/// `include` must be present. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_from_flags(ctx: *const LouderContext, conversation: LouderName, include: HubFlags, exact: i32, start: LouderPosition) -> LouderPosition {
  if ctx.is_null() {
    return BLANK_POSITION;
  }

  unsafe {
    let Some(conversation_ref) = (*ctx).get_conversation(conversation) else {
      return BLANK_POSITION;
    };

    let wanted = include.bits();
    let positions: Vec<LouderPosition> = conversation_ref
      .sorted_positions()
      .into_iter()
      .filter(|position| {
        conversation_ref
          .hub(*position)
          .map(|hub| flags_match(hub.flags().bits(), wanted, exact))
          .unwrap_or(false)
      })
      .collect();

    next_position_after(&positions, start)
  }
}

/// Get a story state value under `state`.
///
/// The variable's own name decides where it is stored: a leading underscore
/// means it is not saved, and an uppercase first letter means it is shared
/// across conversations. Variables that have never been written read as 0.
/// Reading from a conversation that is not loaded also reads as 0.
#[unsafe(no_mangle)]
pub extern "C" fn get_story_state(ctx: *const LouderContext, conversation: LouderName, state: LouderName) -> LouderVariant {
  if ctx.is_null() {
    return LouderVariant::Int(0);
  }

  unsafe { (*ctx).get_state(conversation, state) }
}

/// Set a story state value.
///
/// The variable's own name decides where it is stored, as in
/// [`get_story_state`]. Doing nothing if `ctx` is null.
#[unsafe(no_mangle)]
pub extern "C" fn set_story_state(ctx: *mut LouderContext, conversation: LouderName, key: LouderName, value: LouderVariant) {
  if ctx.is_null() {
    return;
  }

  unsafe {
    (*ctx).set_state(conversation, key, value);
  }
}

/// Serialize the story state in JSON into `buffer`.
///
/// Every bucket is included: the conversation's own saved and unsaved
/// variables, and the shared ones. Keys are written in ascending id order so
/// the output is reproducible. Returns the number of bytes written excluding
/// the terminator; a negative value means the conversation is not loaded or the
/// encoding is unsupported.
#[unsafe(no_mangle)]
pub extern "C" fn read_state_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> i32 {
  if ctx.is_null() || encoding != LOUDER_ENCODING_UTF8 {
    return -1;
  }

  unsafe {
    if (*ctx).get_conversation(conversation).is_none() {
      return -1;
    }

    let context = &*ctx;
    let mut keys = (*ctx).conversation_state_keys(conversation);
    keys.extend((*ctx).shared_state_keys());
    keys.sort_unstable();
    keys.dedup();

    let document = json_object(context, &keys, |key| context.get_state(conversation, key));
    write_json(buffer, max_size, &document) as i32
  }
}

/// Serialize the story state, along with the positions and details of every
/// actor, in JSON into `buffer`.
///
/// The document has a `state` object and an `actors` array, both ordered so
/// that the output is reproducible. Returns the number of bytes written
/// excluding the terminator; a negative value means the conversation is not
/// loaded or the encoding is unsupported.
#[unsafe(no_mangle)]
pub extern "C" fn read_snapshot_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> i32 {
  if ctx.is_null() || encoding != LOUDER_ENCODING_UTF8 {
    return -1;
  }

  unsafe {
    if (*ctx).get_conversation(conversation).is_none() {
      return -1;
    }

    let context = &*ctx;

    let mut keys = (*ctx).conversation_state_keys(conversation);
    keys.extend((*ctx).shared_state_keys());
    keys.sort_unstable();
    keys.dedup();
    let state = json_object(context, &keys, |key| context.get_state(conversation, key));

    let mut actors = String::from("[");
    let entries = (*ctx).actors(conversation);
    for (index, entry) in entries.iter().enumerate() {
      if index > 0 {
        actors.push(',');
      }

      let channel = match context.resolve_name(entry.channel()) {
        Some(text) => json_quote(text),
        None => "null".to_string(),
      };

      let additional_keys = entry.additional().sorted_keys();
      let additional = json_object(context, &additional_keys, |key| {
        entry.additional_get(key).cloned().unwrap_or_default()
      });

      actors.push_str(&format!(
        "{{\"id\":{},\"position\":{},\"channel\":{},\"priority\":{},\"flags\":{},\"speak_as\":{},\"listen_to\":{},\"additional\":{}}}",
        entry.id(),
        entry.position(),
        channel,
        entry.priority(),
        entry.flags().bits(),
        json_name_list(context, entry.speak_as()),
        json_name_list(context, entry.listen_to()),
        additional
      ));
    }
    actors.push(']');

    let document = format!("{{\"state\":{},\"actors\":{}}}", state, actors);
    write_json(buffer, max_size, &document) as i32
  }
}

/// Render a list of names as a JSON array of strings.
fn json_name_list(context: &LouderContext, names: &[LouderName]) -> String {
  let mut out = String::from("[");
  for (index, name) in names.iter().enumerate() {
    if index > 0 {
      out.push(',');
    }
    match context.resolve_name(*name) {
      Some(text) => out.push_str(&json_quote(text)),
      None => out.push_str("null"),
    }
  }
  out.push(']');
  out
}

/// Initialize the engine and return a new context.
///
/// Initializes the name table, the event queue, and the ask cache. Release the
/// returned handle with [`deinit_louder`]. Returns null only if allocation
/// failed.
#[unsafe(no_mangle)]
pub extern "C" fn init_louder() -> *mut LouderContext {
  Box::into_raw(Box::new(LouderContext::new()))
}

/// Release a context and everything it holds.
///
/// Returns [`LouderError::Ok`] on success, or [`LouderError::Missing`] if
/// `ctx` was null. Every conversation, speaker, actor, and queued event goes
/// with it.
#[unsafe(no_mangle)]
pub extern "C" fn deinit_louder(ctx: *mut LouderContext) -> LouderError {
  if ctx.is_null() {
    return LouderError::Missing;
  }
  unsafe {
    drop(Box::from_raw(ctx));
  }
  LouderError::Ok
}

/// Load a conversation from a JSON document in `buffer`.
///
/// The buffer holds `max_size` bytes of UTF-8, unless `encoding` says
/// otherwise, in which case the load fails.
///
/// - The JSON is expected in array form; [`LOUDER_LOAD_NOT_A_LIST`] is
///   returned otherwise.
/// - Standard JSON syntax errors are returned as
///   [`LOUDER_LOAD_MALFORMED_JSON`].
/// - Standard properties are left default or read from matching JSON fields.
/// - Additional JSON fields are treated as additional data on hubs.
/// - [`LOUDER_LOAD_ALREADY_LOADED`] is returned if `conversation` is already
///   occupied by another loaded conversation.
/// - Problems with individual records do not fail the load. They are emitted as
///   events instead, carrying the offending hub's position in
///   [`Motion::source_position`] and the [`LoadError`] in
///   [`Motion::target_position`].
#[unsafe(no_mangle)]
pub extern "C" fn load_conversation_json(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> LoadError {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_LOAD_MALFORMED_JSON;
  }

  let bytes = unsafe { std::slice::from_raw_parts(buffer, max_size) };
  let document = match decode(bytes, encoding) {
    Ok(document) => document,
    Err(error) => return error,
  };

  unsafe { louder::json_loader::load_conversation(&mut *ctx, conversation, document) }
}

/// Remove a conversation and all associated actors.
///
/// Returns 0 on success, or 1 if the conversation was not loaded. Shared story
/// state is kept, since it outlives any one conversation.
#[unsafe(no_mangle)]
pub extern "C" fn remove_conversation(ctx: *mut LouderContext, conversation: LouderName) -> i32 {
  if ctx.is_null() {
    return 1;
  }

  unsafe {
    let removed = (*ctx).remove_conversation(conversation);
    if removed {
      let event = LouderEvent {
        conversation,
        channel: LOUDER_NAME_NONE,
        actor: LOUDER_ACTOR_NONE,
        data: EventData::ConversationRemoved(conversation),
      };
      (*ctx).push_event(event);
    }

    if removed { 0 } else { 1 }
  }
}

/// Load the conversation config from a JSON document in `buffer`.
///
/// - Uses the same encoding and error scheme as [`load_conversation_json`],
///   but with speakers as the target objects instead of hubs.
/// - The config is a singleton as opposed to a named conversation instance,
///   so this takes no conversation name.
/// - Objects are speakers and their configurations. A speaker already in the
///   config is replaced rather than treated as an error.
#[unsafe(no_mangle)]
pub extern "C" fn load_conversation_config(ctx: *mut LouderContext, buffer: *mut u8, max_size: usize, encoding: LouderEncoding) -> LoadError {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_LOAD_MALFORMED_JSON;
  }

  let bytes = unsafe { std::slice::from_raw_parts(buffer, max_size) };
  let document = match decode(bytes, encoding) {
    Ok(document) => document,
    Err(error) => return error,
  };

  unsafe { louder::json_loader::load_config(&mut *ctx, document) }
}

/// Remove all speakers in the config.
///
/// Returns 0 on success, or 1 if `ctx` was null. Conversations and actors are
/// untouched, so hubs naming a now-missing speaker become public again.
#[unsafe(no_mangle)]
pub extern "C" fn clear_config(ctx: *mut LouderContext) -> i32 {
  if ctx.is_null() {
    return 1;
  }

  unsafe {
    (*ctx).clear_config();
  }

  0
}

/// Add an actor to a conversation and return the integer id'ing it.
///
/// The actor starts at `start_position` on the default channel, with priority
/// 0. Ids are per conversation and are never reused, so ascending order is a
/// stable iteration order.
///
/// - `speak_as` and `listen_to` are arrays of `count` names; either may be null
///   when its count is 0.
/// - [`ActorFlags::SPECTATOR`] and [`ActorFlags::DEBUG`] actors are given no
///   channel or priority.
///
/// Emits an [`EventData::ActorAdded`] event on success. Returns the new actor
/// id, or [`LOUDER_ACTOR_NONE`] if the conversation is not loaded or a pointer
/// was null.
#[unsafe(no_mangle)]
pub extern "C" fn add_actor(ctx: *mut LouderContext, conversation: LouderName, start_position: LouderPosition, flags: ActorFlags, speak_as: *const LouderName, speak_as_count: i32, listen_to: *const LouderName, listen_to_count: i32) -> i32 {
  if ctx.is_null() || speak_as_count < 0 || listen_to_count < 0 {
    return LOUDER_ACTOR_NONE;
  }

  if (speak_as.is_null() && speak_as_count > 0) || (listen_to.is_null() && listen_to_count > 0) {
    return LOUDER_ACTOR_NONE;
  }

  unsafe {
    if (*ctx).get_conversation(conversation).is_none() {
      return LOUDER_ACTOR_NONE;
    }

    let main = (*ctx).main_channel();

    let speak_as_list: Vec<LouderName> = if speak_as_count == 0 {
      Vec::new()
    } else {
      std::slice::from_raw_parts(speak_as, speak_as_count as usize).to_vec()
    };
    let listen_to_list: Vec<LouderName> = if listen_to_count == 0 {
      Vec::new()
    } else {
      std::slice::from_raw_parts(listen_to, listen_to_count as usize).to_vec()
    };

    let actor = Actor::new(
      LOUDER_ACTOR_NONE,
      start_position,
      flags,
      speak_as_list,
      listen_to_list,
      main,
    );

    let id = (*ctx).add_actor(conversation, actor);

    let event = LouderEvent {
      conversation,
      channel: main,
      actor: id,
      data: EventData::ActorAdded,
    };
    (*ctx).push_event(event);

    id
  }
}

/// Remove an actor from a conversation.
///
/// Returns 0 on success, or 1 if the actor or conversation did not exist.
/// Emits an [`EventData::ActorRemoved`] event on success.
#[unsafe(no_mangle)]
pub extern "C" fn remove_actor(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor) -> i32 {
  if ctx.is_null() {
    return 1;
  }

  unsafe {
    if !(*ctx).remove_actor(conversation, actor) {
      return 1;
    }

    let event = LouderEvent {
      conversation,
      channel: LOUDER_NAME_NONE,
      actor,
      data: EventData::ActorRemoved,
    };
    (*ctx).push_event(event);
  }

  0
}

/// Select a choice on an actor's current hub.
///
/// The choice must be one the actor could actually take; see
/// [`get_hub_choice_count`] for what that means. Auto hubs along the way are
/// resolved by taking the first choice available at each one.
///
/// Returns 0 on success, or 1 if the actor, hub, or choice was missing, if the
/// choice is not available to this actor, or if an auto hub ran out of choices.
#[unsafe(no_mangle)]
pub extern "C" fn select_choice(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, choice_index: i32) -> i32 {
  if ctx.is_null() {
    return 1;
  }

  unsafe {
    match Simulator::select_choice(&mut *ctx, conversation, actor, choice_index) {
      Ok(_) => 0,
      Err(_) => 1,
    }
  }
}

/// Move an actor to another position as if it had moved through the network.
///
/// - `fake_priority` is unused if `0`, otherwise overrides the priority of the
///   destination. Because 0 is the ordinary default priority, this cannot force
///   an actor's priority *to* 0 over a hub that sets one.
/// - `fake_channel` is unused if [`LOUDER_NAME_NONE`], otherwise overrides the
///   channel of the destination.
///
/// Other actors sharing the actor's channel may be pulled along, exactly as if
/// the actor had chosen. Returns 0 on success, or 1 if the conversation, actor,
/// or destination was missing, or if an auto hub ran out of choices.
#[unsafe(no_mangle)]
pub extern "C" fn force_warp(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, target_position: LouderPosition, fake_channel: LouderName, fake_priority: i32) -> i32 {
  if ctx.is_null() {
    return 1;
  }

  unsafe {
    match Simulator::force_move(&mut *ctx, conversation, actor, target_position, MotionKind::Warp, fake_channel, fake_priority) {
      Ok(_) => 0,
      Err(_) => 1,
    }
  }
}

/// Move an actor to another position directly.
///
/// - `fake_priority` is unused if `0`, otherwise overrides the priority of the
///   destination. Because 0 is the ordinary default priority, this cannot force
///   an actor's priority *to* 0 over a hub that sets one.
/// - `fake_channel` is unused if [`LOUDER_NAME_NONE`], otherwise overrides the
///   channel of the destination.
///
/// Returns 0 on success, or 1 if the conversation, actor, or destination was
/// missing, or if an auto hub ran out of choices.
#[unsafe(no_mangle)]
pub extern "C" fn force_position(ctx: *mut LouderContext, conversation: LouderName, actor: LouderActor, target_position: LouderPosition, fake_channel: LouderName, fake_priority: i32) -> i32 {
  if ctx.is_null() {
    return 1;
  }

  unsafe {
    match Simulator::force_move(&mut *ctx, conversation, actor, target_position, MotionKind::Force, fake_channel, fake_priority) {
      Ok(_) => 0,
      Err(_) => 1,
    }
  }
}
