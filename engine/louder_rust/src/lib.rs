// The primary engine interface, defined in docs/engine/index.md
//
// This crate is the engine and nothing else. The `extern "C"` entry points below
// are its whole outward face, and the generated C header is built from them. No
// host language is spoken here: an adapter, such as the Godot extension in
// `louder_godot`, depends on this crate as an ordinary library and declares its
// own classes over it. The only thing this crate keeps for an adapter is the
// `godot_api!` macro in `godot_api.rs`, behind the `godot-bindings` feature.
//
// Strings cross this boundary as bytes, and nothing here decides how they are
// encoded. A host that hands over bytes in an encoding the engine does not read
// gets them back the same way, and a host that wants text decodes it itself
// after the call.

mod test_linkage;

// The `godot_api!` macro an adapter declares its classes with. The module itself
// is private; the macro is exported at the crate root, and needs the feature.
mod godot_api;

pub mod louder;

use louder::actor::*;
use louder::callback::*;
use louder::context::*;
use louder::event::*;
use louder::expression::*;
use louder::ffi::*;
use louder::hub::*;
use louder::json;
use louder::json_loader::*;
use louder::object::*;
use louder::query;
pub use louder::rule::*;
use louder::rules::DialogueRules;
use louder::name::*;
use louder::simulator::*;
use louder::speaker::*;

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

/// Build an [`EventData::LoadError`].
///
/// The engine emits these itself when it cannot read a record of a loaded
/// document, so this exists to round out the set rather than because a host
/// has a reason to raise one.
#[unsafe(no_mangle)]
pub extern "C" fn make_event_data_load_error(failure: LoadFailure) -> EventData {
EventData::LoadError(failure)
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
if ctx.is_null() {
  return LOUDER_NAME_NONE;
}

let Some(content) = read_name_content(buffer, max_size) else {
  return LOUDER_NAME_NONE;
};

unsafe {
  match (*ctx).find_name(&content) {
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
if ctx.is_null() {
  return LOUDER_NAME_NONE;
}

let Some(content) = read_name_content(buffer, max_size) else {
  return LOUDER_NAME_NONE;
};

unsafe {
  match (*ctx).add_name(&content) {
    LouderContextResult::Found(_) => LOUDER_NAME_NONE,
    LouderContextResult::NotFound(name) => name,
  }
}
}

/// Read the bytes associated with an id into `buffer`.
///
/// The bytes come back exactly as they went in, so a name declared in some
/// encoding is read in that same one. Decoding them is the host's job, since the
/// engine never displays anything itself.
///
/// Writes 0 bytes and a terminator if no name is associated with the id, or if
/// either pointer was null. Returns the number of bytes written, excluding the
/// terminator.
#[unsafe(no_mangle)]
pub extern "C" fn read_name(ctx: *const LouderContext, name: LouderName, buffer: *mut u8, max_size: usize) -> usize {
if ctx.is_null() || buffer.is_null() {
  return 0;
}

let name_context = unsafe { (*ctx).get_name_content(&name) };
let Some(name_context) = name_context else {
  return write_bytes(buffer, max_size, &[]);
};

write_bytes(buffer, max_size, &name_context.bytes)
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

match unsafe { (*ctx).event(index) } {
  Some(event) => LouderOption::Some(event.clone()),
  None => LouderOption::None,
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

unsafe { (*ctx).clear_events() };
}

/// Get the traced event at `index`.
///
/// The trace is every event the engine has created, in creation order, and is
/// not emptied by [`flush_events`]. Returns [`LouderOption::None`] once `index`
/// is past the end of the trace, or if `ctx` is null.
#[unsafe(no_mangle)]
pub extern "C" fn get_traced_event(ctx: *const LouderContext, index: usize) -> LouderOption<LouderEvent> {
if ctx.is_null() {
  return LouderOption::None
}

match unsafe { (*ctx).traced_event(index) } {
  Some(event) => LouderOption::Some(event.clone()),
  None => LouderOption::None,
}
}

/// Remove every traced event.
///
/// The event queue is untouched, so this discards the history without changing
/// what the host still has to process.
#[unsafe(no_mangle)]
pub extern "C" fn flush_traced_events(ctx: *mut LouderContext) {
if ctx.is_null() {
  return;
}

unsafe { (*ctx).clear_traced_events() };
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

/// Get the traced ask at `index`.
///
/// The trace is every ask the engine has created, in creation order. Answering
/// an ask moves it around the cache but leaves its traced copy where it was.
/// Returns [`LouderOption::None`] once `index` is past the end of the trace, or
/// if `ctx` is null.
#[unsafe(no_mangle)]
pub extern "C" fn get_traced_ask(ctx: *const LouderContext, index: usize) -> LouderOption<LouderAsk> {
if ctx.is_null() {
  return LouderOption::None
}

match unsafe { (*ctx).traced_ask(index) } {
  Some(ask) => LouderOption::Some(ask.clone()),
  None => LouderOption::None,
}
}

/// Remove every traced ask.
///
/// The ask cache is untouched, so unanswered asks can still be answered after
/// this.
#[unsafe(no_mangle)]
pub extern "C" fn flush_traced_asks(ctx: *mut LouderContext) {
if ctx.is_null() {
  return;
}

unsafe { (*ctx).clear_traced_asks() };
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

unsafe { query::next_speaker_listen_to(&*ctx, speaker, start) }
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

unsafe { query::next_actor_speak_as(&*ctx, conversation, actor, start) }
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

unsafe { query::next_actor_listening_base(&*ctx, conversation, actor, start) }
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

unsafe { query::next_actor_listening(&*ctx, conversation, actor, start) }
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

unsafe { query::next_actor_in_channel(&*ctx, conversation, channel, start) }
}

/// Iterate the actors in a channel at a specific position.
///
/// Pass [`LOUDER_ACTOR_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_in_channel_and_position(ctx: *const LouderContext, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
if ctx.is_null() {
  return LOUDER_ACTOR_NONE;
}

unsafe { query::next_actor_in_channel_and_position(&*ctx, conversation, channel, position, start) }
}

/// Iterate the actors speaking as a speaker.
///
/// Pass [`LOUDER_ACTOR_NONE`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_actor_speaking_as(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
if ctx.is_null() {
  return LOUDER_ACTOR_NONE;
}

unsafe { query::next_actor_speaking_as(&*ctx, conversation, speaker, start) }
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

unsafe { query::next_actor_listening_to(&*ctx, conversation, speaker, start) }
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

unsafe { query::next_actor_from_flags(&*ctx, conversation, include, exact, start) }
}

/// Iterate the tags of a speaker.
///
/// Pass [`LOUDER_NAME_NONE`] as `start` to begin, then pass the previous result
/// until [`LOUDER_NAME_NONE`] comes back. Returns [`LOUDER_NAME_NONE`] for an
/// unknown speaker.
#[unsafe(no_mangle)]
pub extern "C" fn get_speaker_tags(ctx: *const LouderContext, speaker: LouderName) -> *const LouderName {
if ctx.is_null() {
  return std::ptr::null();
}

unsafe {
  let tags = (*ctx).get_speaker_tags(speaker);
  tags.as_ptr()
}
}

/// Iterate the tags of a speaker.
///
/// Pass [`LOUDER_NAME_NONE`] as `start` to begin, then pass the previous result
/// until [`LOUDER_NAME_NONE`] comes back. Returns [`LOUDER_NAME_NONE`] for an
/// unknown speaker.
#[unsafe(no_mangle)]
pub extern "C" fn next_speaker_tag(ctx: *const LouderContext, speaker: LouderName, start: LouderName) -> LouderName {
if ctx.is_null() {
  return LOUDER_NAME_NONE;
}

unsafe { query::next_speaker_tag(&*ctx, speaker, start) }
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
  match (*ctx).hub(conversation, position) {
    Some(hub) => hub.speaker(),
    None => LOUDER_NAME_NONE,
  }
}
}

/// Get the speaker of a hub, as an actor would hear it.
///
/// A hub with a speaker of its own speaks as that speaker. A hub without one is
/// anonymous: it speaks as the first voice in the speak as list of whoever is
/// standing on it, so the answer depends on the actor, and a hub that has no
/// speaker of its own answers differently for each of them.
///
/// Returns [`LOUDER_NAME_NONE`] if the hub does not exist, or if the actor has
/// no voice to speak it with.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_speaker_for_actor(
  ctx: *const LouderContext,
  conversation: LouderName,
  position: LouderPosition,
  actor_perspective: LouderActor,
) -> LouderName {
if ctx.is_null() {
  return LOUDER_NAME_NONE;
}

unsafe { Simulator::hub_speaker_for(&*ctx, conversation, actor_perspective, position) }
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
  match (*ctx).hub(conversation, position) {
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

/// Get the rule behind a choice's availability.
///
/// Returns 0 for an available choice's rule mapping, or -1 for an unknown
/// choice. The value is a [`ChoiceRule`].
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_choice_rule(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> i32 {
  if ctx.is_null() || choice_index < 0 {
    return -1;
  }
  unsafe {
    Simulator::choice_rule(&*ctx, conversation, actor_perspective, position, choice_index as usize) as i32
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
  match query::available_choice(&*ctx, conversation, position, actor_perspective, choice_index as usize) {
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
  match query::available_choice(&*ctx, conversation, position, actor_perspective, choice_index as usize) {
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
  match (*ctx).hub(conversation, position) {
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
  match (*ctx).hub(conversation, position) {
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
  match (*ctx).hub(conversation, position) {
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

unsafe { query::next_hub_recipient(&*ctx, conversation, position, start) }
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
  match (*ctx).hub(conversation, position) {
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
  match (*ctx).hub(conversation, position) {
    Some(hub) => hub.priority(),
    None => 0,
  }
}
}

/// Get the hub a link node points at, or -1 when it is not a link node.
#[unsafe(no_mangle)]
pub extern "C" fn get_hub_link(ctx: *const LouderContext, conversation: LouderName, position: LouderPosition) -> LouderPosition {
if ctx.is_null() {
  return -1;
}

unsafe {
  match (*ctx).hub(conversation, position) {
    Some(hub) => hub.link(),
    None => -1,
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
    .hub(conversation, position)
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

unsafe { query::next_hub_by_channel(&*ctx, conversation, channel, start) }
}

/// Iterate the hubs spoken by a speaker.
///
/// Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_spoken_by(ctx: *const LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
if ctx.is_null() {
  return BLANK_POSITION;
}

unsafe { query::next_hub_spoken_by(&*ctx, conversation, speaker, start) }
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

unsafe { query::next_hub_heard_by(&*ctx, conversation, speaker, start) }
}

/// Iterate the hubs with a priority within `[priority_min, priority_max]`.
///
/// The bounds are inclusive. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_by_priority(ctx: *const LouderContext, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
if ctx.is_null() {
  return BLANK_POSITION;
}

unsafe { query::next_hub_by_priority(&*ctx, conversation, priority_min, priority_max, start) }
}

/// Iterate the hubs containing all, or exactly, the given tags.
///
/// With `exact` nonzero the hub must carry exactly the listed tags; otherwise
/// carrying extra tags is fine. Pass [`BLANK_POSITION`] as `start` to begin.
#[unsafe(no_mangle)]
pub extern "C" fn next_hub_from_tags(ctx: *const LouderContext, conversation: LouderName, include: *const LouderName, include_count: i32, exact: i32, start: LouderPosition) -> LouderPosition {
if ctx.is_null() {
  return BLANK_POSITION;
}

let Some(wanted) = read_name_list(include, include_count) else {
  return BLANK_POSITION;
};

unsafe { query::next_hub_from_tags(&*ctx, conversation, &wanted, exact != 0, start) }
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

unsafe { query::next_hub_from_flags(&*ctx, conversation, include, exact, start) }
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
/// the output is reproducible.
///
/// The document is written as UTF-8. Returns the number of bytes written
/// excluding the terminator; a negative value means the conversation is not
/// loaded.
#[unsafe(no_mangle)]
pub extern "C" fn read_state_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize) -> i32 {
if ctx.is_null() {
  return -1;
}

unsafe {
  write_bytes(buffer, max_size, json::state(&*ctx, conversation).as_bytes()) as i32
}
}

/// Serialize the story state, along with the positions and details of every
/// actor, in JSON into `buffer`.
///
/// The document has a `state` object and an `actors` array, both ordered so
/// that the output is reproducible.
///
/// The document is written as UTF-8. Returns the number of bytes written
/// excluding the terminator; a negative value means the conversation is not
/// loaded.
#[unsafe(no_mangle)]
pub extern "C" fn read_snapshot_json(ctx: *const LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize) -> i32 {
if ctx.is_null() {
  return -1;
}

unsafe {
  match json::snapshot(&*ctx, conversation) {
    Some(document) => write_bytes(buffer, max_size, document.as_bytes()) as i32,
    None => -1,
  }
}
}

/// Serialize the event and ask traces in JSON into `buffer`.
///
/// The document has an `events` array and an `asks` array, both in creation
/// order. Unlike the event queue and ask cache, the traces are not affected by
/// [`flush_events`] or by answering an ask, so this is a complete history of
/// what the engine has done.
///
/// The document is written as UTF-8. Returns the number of bytes written
/// excluding the terminator.
#[unsafe(no_mangle)]
pub extern "C" fn read_trace_json(ctx: *const LouderContext, buffer: *mut u8, max_size: usize) -> i32 {
if ctx.is_null() {
  return -1;
}

unsafe {
  write_bytes(buffer, max_size, json::trace(&*ctx).as_bytes()) as i32
}
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
/// The buffer holds `max_size` bytes. JSON is text, so those bytes have to be
/// UTF-8 for the engine to read them; [`LOUDER_LOAD_MALFORMED_JSON`] is returned
/// if they are not, and re-encoding them is the host's job rather than the
/// engine's.
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
pub extern "C" fn load_conversation_json(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize) -> LoadError {
if ctx.is_null() || buffer.is_null() {
  return LOUDER_LOAD_MALFORMED_JSON;
}

let bytes = unsafe { std::slice::from_raw_parts(buffer, max_size) };
let document: &str = match decode(bytes) {
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
  if (*ctx).remove_conversation(conversation) { 0 } else { 1 }
}
}

/// Load the conversation config from a JSON document in `buffer`.
///
/// - The bytes have to be UTF-8 and the error scheme is the one in
///   [`load_conversation_json`], with speakers as the target objects instead of
///   hubs.
/// - The config is a singleton as opposed to a named conversation instance,
///   so this takes no conversation name.
/// - Objects are speakers and their configurations. A speaker already in the
///   config is replaced rather than treated as an error.
#[unsafe(no_mangle)]
pub extern "C" fn load_conversation_config(ctx: *mut LouderContext, buffer: *mut u8, max_size: usize) -> LoadError {
if ctx.is_null() || buffer.is_null() {
  return LOUDER_LOAD_MALFORMED_JSON;
}

let bytes = unsafe { std::slice::from_raw_parts(buffer, max_size) };
let document = match decode(bytes) {
  Ok(document) => document,
  Err(error) => return error,
};

unsafe { louder::json_loader::load_config(&mut *ctx, document) }
}

/// Load a conversation from a document in the readable text format.
///
/// The bytes must be UTF-8. The format is the line-oriented one described in
/// the issue: hubs separated by blank lines, properties as `key: value`, an
/// optional speaker and text as unlabeled blocks, choices as `>` lines naming
/// tags, and `link:` turning a hub into a link hub. Property keys and values
/// are stripped before interpretation.
#[unsafe(no_mangle)]
pub extern "C" fn load_conversation_text(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize) -> LoadError {
  load_conversation_text_impl(ctx, conversation, buffer, max_size)
}

fn load_conversation_text_impl(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize) -> LoadError {
  if ctx.is_null() || buffer.is_null() {
    return LOUDER_LOAD_MALFORMED_JSON;
  }

  let bytes = unsafe { std::slice::from_raw_parts(buffer, max_size) };
  let document = match decode(bytes) {
    Ok(document) => document,
    Err(error) => return error,
  };

  unsafe { louder::text_loader::load_conversation(&mut *ctx, conversation, document) }
}

macro_rules! text_loader_alias {
  ($name:ident) => {
    #[unsafe(no_mangle)]
    pub extern "C" fn $name(ctx: *mut LouderContext, conversation: LouderName, buffer: *mut u8, max_size: usize) -> LoadError {
      load_conversation_text_impl(ctx, conversation, buffer, max_size)
    }
  };
}

text_loader_alias!(load_text_conversation);
text_loader_alias!(load_conversation_readable);
text_loader_alias!(load_readable_conversation);
text_loader_alias!(load_conversation_from_text);
text_loader_alias!(load_conversation_string);
text_loader_alias!(parse_conversation_text);
text_loader_alias!(load_conversation);

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

/// Read the dialogue rules currently in effect.
///
/// A fresh context has every rule on. Returns an empty set if `ctx` was null,
/// since a null context is not running any dialogue.
#[unsafe(no_mangle)]
pub extern "C" fn get_dialogue_rules(ctx: *const LouderContext) -> DialogueRules {
if ctx.is_null() {
  return DialogueRules::empty();
}

unsafe { (*ctx).rules() }
}

/// Replace the dialogue rules.
///
/// A rule takes effect on the next move rather than being applied to the
/// conversation as it stands, so a host can turn one off while an actor is
/// standing on a hub it would otherwise resolve through.
///
/// Bits outside [`DialogueRules`] are ignored rather than refused, so a host
/// carrying a configuration written for a newer engine still gets the rules
/// this build knows about.
#[unsafe(no_mangle)]
pub extern "C" fn set_dialogue_rules(ctx: *mut LouderContext, rules: DialogueRules) -> i32 {
if ctx.is_null() {
  return 1;
}

unsafe {
  (*ctx).set_rules(DialogueRules::from_bits_truncate(rules.bits()));
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
if ctx.is_null() {
  return LOUDER_ACTOR_NONE;
}

let (Some(speak_as_list), Some(listen_to_list)) = (read_name_list(speak_as, speak_as_count), read_name_list(listen_to, listen_to_count)) else {
  return LOUDER_ACTOR_NONE;
};

unsafe {
  if (*ctx).get_conversation(conversation).is_none() {
    return LOUDER_ACTOR_NONE;
  }

  let main = (*ctx).main_channel();
  let actor = Actor::new(start_position, flags, speak_as_list, listen_to_list, main);
  (*ctx).add_actor(conversation, actor)
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

if unsafe { (*ctx).remove_actor(conversation, actor) } {
  0
} else {
  1
}
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
