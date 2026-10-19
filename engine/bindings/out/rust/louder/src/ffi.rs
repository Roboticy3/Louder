//! Bindings for Rust, generated from the C header of the Louder engine.
//! 
//! Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
//! The C boundary, exactly as the header describes it.
//!
//! Nothing here interprets anything. A name is an integer, a tagged union
//! is a tag and a union, and a string is a length and a buffer. The safe
//! layer in the crate root is where those become names, variants and text.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

/// What an LouderEvent is about.
/// A tagged union cannot be read as one value by a host that has to switch on it, so this is the discriminant of an EventData as an `int`, and the payload is read through the accessors that match it. A host that does not recognise a value is looking at something a newer engine produced.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventData_Tag {
    None = 0,
    RegularMotion = 1,
    Warp = 2,
    ForceMotion = 3,
    ChoiceAppearance = 4,
    ChannelChange = 5,
    AdditionalDataChange = 6,
    SpeakerChange = 7,
    TextChange = 8,
    Statement = 9,
    Condition = 10,
    Callback = 11,
    ActorAdded = 12,
    ActorRemoved = 13,
    SpeakerAdded = 14,
    SpeakerRemoved = 15,
    ConversationAdded = 16,
    ConversationRemoved = 17,
    LoadError = 18,
}

/// An error code returned by most operations.
/// - LouderError::Ok: no error. - LouderError::Missing: the requested object does not exist.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LouderError {
    Ok = 0,
    Missing = 1,
}

/// Custom Option type exposed to the C FFI
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LouderOption_LouderAsk_Tag {
    None_LouderAsk = 0,
    Some_LouderAsk = 1,
}

/// Custom Option type exposed to the C FFI
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LouderOption_LouderEvent_Tag {
    None_LouderEvent = 0,
    Some_LouderEvent = 1,
}

/// One unit of data in an actor, hub, speaker, or story state.
/// Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is what makes them interoperable with the arithmetic and comparison rules of the engine guide.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LouderVariant_Tag {
    Int = 0,
    Float = 1,
    String = 2,
}

/// Flag bits describing an actor.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ActorFlags {
    pub bits: i32,
}

/// A key of an actor's additional data being replaced.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct AdditionalDataChange {
    pub key: i32,
}

/// A choice becoming available on a hub.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ChoiceAppearance {
    pub position: i32,
    pub choice_index: i32,
}

/// Every arm of a union, sharing one block of memory.
///
/// Only `Copy` is derived. `Debug` cannot be, because a union has no
/// single value to describe, and `Default` has nothing to pick.
#[repr(C)]
#[derive(Clone, Copy)]
pub union LouderVariant_Body {
    pub int: i32,
    pub float: f32,
    pub string: i32,
}

/// One unit of data in an actor, hub, speaker, or story state.
/// Booleans are not a separate kind: they are carried as LouderVariant::Int 1 or 0, which is what makes them interoperable with the arithmetic and comparison rules of the engine guide.
/// `Debug` and `Default` are written out below rather than derived:
/// the body is a union, which has neither, and the tag has no sensible
/// zero.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LouderVariant {
    pub tag: LouderVariant_Tag,
    pub body: LouderVariant_Body,
}

/// The arm a LouderVariant carries, read out of its shared memory.
#[derive(Clone, Copy, Debug)]
pub enum LouderVariantArm {
    int(i32),
    float(f32),
    string(i32),
}

impl std::fmt::Debug for LouderVariant {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.arm() {
            None => write!(out, concat!(stringify!(LouderVariant), " {{ tag: {:?} }}"), self.tag),
            Some(LouderVariantArm::int(value)) => write!(out, concat!("LouderVariant::int", " {{ {:?} }}"), value),
            Some(LouderVariantArm::float(value)) => write!(out, concat!("LouderVariant::float", " {{ {:?} }}"), value),
            Some(LouderVariantArm::string(value)) => write!(out, concat!("LouderVariant::string", " {{ {:?} }}"), value),
        }
    }
}

impl Default for LouderVariant {
    /// The zeroed union.
    ///
    /// Which arm that is depends on the engine, so this is only useful as
    /// something to write into rather than as a value.
    fn default() -> Self {
        LouderVariant {
            tag: unsafe { std::mem::zeroed() },
            body: unsafe { std::mem::zeroed() },
        }
    }
}

impl LouderVariant {
    /// The arm the tag names, or `None` for an arm that carries nothing.
    ///
    /// A tag this build does not know about is `None` too, rather than
    /// being read as whichever arm happens to share its bits.
    pub fn arm(&self) -> Option<LouderVariantArm> {
        match self.tag {
            LouderVariant_Tag::Int => Some(LouderVariantArm::int(unsafe { self.body.int })),
            LouderVariant_Tag::Float => Some(LouderVariantArm::float(unsafe { self.body.float })),
            LouderVariant_Tag::String => Some(LouderVariantArm::string(unsafe { self.body.string })),
        }
    }
}

/// What a condition on a hub came to, when an actor landed on it.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct ConditionOutcome {
    pub condition: i32,
    pub outcome: LouderVariant,
}

/// The dialogue rules a host can turn on and off.
/// Both are on by default, because each is what makes the engine usable without every conversation having to be written around its absence. They are gathered here rather than left as separate switches so that a host reads and writes the whole configuration in one call, and so that adding a rule does not add another pair of engine calls.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct DialogueRules {
    pub bits: i32,
}

/// One move of an actor from one hub to another.
/// Every event a move produces carries the same motion, which is how a host reads where an actor came from and where it went.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct Motion {
    pub source_position: i32,
    pub target_position: i32,
    pub choice_index: i32,
    pub background: bool,
}

/// A record in a document the engine could not read.
/// Carries the position of the record that was at fault, or LOUDER_POSITION_NONE when the problem was with the document as a whole.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LoadFailure {
    pub position: i32,
    pub error: i32,
}

/// Every arm of a union, sharing one block of memory.
///
/// Only `Copy` is derived. `Debug` cannot be, because a union has no
/// single value to describe, and `Default` has nothing to pick.
#[repr(C)]
#[derive(Clone, Copy)]
pub union EventData_Body {
    pub regular_motion: Motion,
    pub warp: Motion,
    pub force_motion: Motion,
    pub choice_appearance: ChoiceAppearance,
    pub additional_data_change: AdditionalDataChange,
    pub speaker_change: i32,
    pub text_change: i32,
    pub statement: i32,
    pub condition: ConditionOutcome,
    pub callback: i32,
    pub speaker_added: i32,
    pub speaker_removed: i32,
    pub conversation_added: i32,
    pub conversation_removed: i32,
    pub load_error: LoadFailure,
}

/// What an LouderEvent is about.
/// A tagged union cannot be read as one value by a host that has to switch on it, so this is the discriminant of an EventData as an `int`, and the payload is read through the accessors that match it. A host that does not recognise a value is looking at something a newer engine produced.
/// `Debug` and `Default` are written out below rather than derived:
/// the body is a union, which has neither, and the tag has no sensible
/// zero.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EventData {
    pub tag: EventData_Tag,
    pub body: EventData_Body,
}

/// The arm a EventData carries, read out of its shared memory.
#[derive(Clone, Copy, Debug)]
pub enum EventDataArm {
    regular_motion(Motion),
    warp(Motion),
    force_motion(Motion),
    choice_appearance(ChoiceAppearance),
    additional_data_change(AdditionalDataChange),
    speaker_change(i32),
    text_change(i32),
    statement(i32),
    condition(ConditionOutcome),
    callback(i32),
    speaker_added(i32),
    speaker_removed(i32),
    conversation_added(i32),
    conversation_removed(i32),
    load_error(LoadFailure),
}

impl std::fmt::Debug for EventData {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.arm() {
            None => write!(out, concat!(stringify!(EventData), " {{ tag: {:?} }}"), self.tag),
            Some(EventDataArm::regular_motion(value)) => write!(out, concat!("EventData::regular_motion", " {{ {:?} }}"), value),
            Some(EventDataArm::warp(value)) => write!(out, concat!("EventData::warp", " {{ {:?} }}"), value),
            Some(EventDataArm::force_motion(value)) => write!(out, concat!("EventData::force_motion", " {{ {:?} }}"), value),
            Some(EventDataArm::choice_appearance(value)) => write!(out, concat!("EventData::choice_appearance", " {{ {:?} }}"), value),
            Some(EventDataArm::additional_data_change(value)) => write!(out, concat!("EventData::additional_data_change", " {{ {:?} }}"), value),
            Some(EventDataArm::speaker_change(value)) => write!(out, concat!("EventData::speaker_change", " {{ {:?} }}"), value),
            Some(EventDataArm::text_change(value)) => write!(out, concat!("EventData::text_change", " {{ {:?} }}"), value),
            Some(EventDataArm::statement(value)) => write!(out, concat!("EventData::statement", " {{ {:?} }}"), value),
            Some(EventDataArm::condition(value)) => write!(out, concat!("EventData::condition", " {{ {:?} }}"), value),
            Some(EventDataArm::callback(value)) => write!(out, concat!("EventData::callback", " {{ {:?} }}"), value),
            Some(EventDataArm::speaker_added(value)) => write!(out, concat!("EventData::speaker_added", " {{ {:?} }}"), value),
            Some(EventDataArm::speaker_removed(value)) => write!(out, concat!("EventData::speaker_removed", " {{ {:?} }}"), value),
            Some(EventDataArm::conversation_added(value)) => write!(out, concat!("EventData::conversation_added", " {{ {:?} }}"), value),
            Some(EventDataArm::conversation_removed(value)) => write!(out, concat!("EventData::conversation_removed", " {{ {:?} }}"), value),
            Some(EventDataArm::load_error(value)) => write!(out, concat!("EventData::load_error", " {{ {:?} }}"), value),
        }
    }
}

impl Default for EventData {
    /// The zeroed union.
    ///
    /// Which arm that is depends on the engine, so this is only useful as
    /// something to write into rather than as a value.
    fn default() -> Self {
        EventData {
            tag: unsafe { std::mem::zeroed() },
            body: unsafe { std::mem::zeroed() },
        }
    }
}

impl EventData {
    /// The arm the tag names, or `None` for an arm that carries nothing.
    ///
    /// A tag this build does not know about is `None` too, rather than
    /// being read as whichever arm happens to share its bits.
    pub fn arm(&self) -> Option<EventDataArm> {
        match self.tag {
            EventData_Tag::None => None,
            EventData_Tag::RegularMotion => Some(EventDataArm::regular_motion(unsafe { self.body.regular_motion })),
            EventData_Tag::Warp => Some(EventDataArm::warp(unsafe { self.body.warp })),
            EventData_Tag::ForceMotion => Some(EventDataArm::force_motion(unsafe { self.body.force_motion })),
            EventData_Tag::ChoiceAppearance => Some(EventDataArm::choice_appearance(unsafe { self.body.choice_appearance })),
            EventData_Tag::ChannelChange => None,
            EventData_Tag::AdditionalDataChange => Some(EventDataArm::additional_data_change(unsafe { self.body.additional_data_change })),
            EventData_Tag::SpeakerChange => Some(EventDataArm::speaker_change(unsafe { self.body.speaker_change })),
            EventData_Tag::TextChange => Some(EventDataArm::text_change(unsafe { self.body.text_change })),
            EventData_Tag::Statement => Some(EventDataArm::statement(unsafe { self.body.statement })),
            EventData_Tag::Condition => Some(EventDataArm::condition(unsafe { self.body.condition })),
            EventData_Tag::Callback => Some(EventDataArm::callback(unsafe { self.body.callback })),
            EventData_Tag::ActorAdded => None,
            EventData_Tag::ActorRemoved => None,
            EventData_Tag::SpeakerAdded => Some(EventDataArm::speaker_added(unsafe { self.body.speaker_added })),
            EventData_Tag::SpeakerRemoved => Some(EventDataArm::speaker_removed(unsafe { self.body.speaker_removed })),
            EventData_Tag::ConversationAdded => Some(EventDataArm::conversation_added(unsafe { self.body.conversation_added })),
            EventData_Tag::ConversationRemoved => Some(EventDataArm::conversation_removed(unsafe { self.body.conversation_removed })),
            EventData_Tag::LoadError => Some(EventDataArm::load_error(unsafe { self.body.load_error })),
        }
    }

    /// What kind of event this is, under the name the engine gave it.
    ///
    /// Written out from the tag enum, so an event kind added in Rust shows
    /// up here without anything having to be remembered to add it.
    pub fn kind(&self) -> Option<&'static str> {
        Some(match self.tag {
            EventData_Tag::None => "None",
            EventData_Tag::RegularMotion => "RegularMotion",
            EventData_Tag::Warp => "Warp",
            EventData_Tag::ForceMotion => "ForceMotion",
            EventData_Tag::ChoiceAppearance => "ChoiceAppearance",
            EventData_Tag::ChannelChange => "ChannelChange",
            EventData_Tag::AdditionalDataChange => "AdditionalDataChange",
            EventData_Tag::SpeakerChange => "SpeakerChange",
            EventData_Tag::TextChange => "TextChange",
            EventData_Tag::Statement => "Statement",
            EventData_Tag::Condition => "Condition",
            EventData_Tag::Callback => "Callback",
            EventData_Tag::ActorAdded => "ActorAdded",
            EventData_Tag::ActorRemoved => "ActorRemoved",
            EventData_Tag::SpeakerAdded => "SpeakerAdded",
            EventData_Tag::SpeakerRemoved => "SpeakerRemoved",
            EventData_Tag::ConversationAdded => "ConversationAdded",
            EventData_Tag::ConversationRemoved => "ConversationRemoved",
            EventData_Tag::LoadError => "LoadError",
        })
    }
}

/// Flag bits describing a hub.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct HubFlags {
    pub bits: i32,
}

/// A question the engine has asked the host and is waiting on.
/// Answering an ask moves it to the back of the cache, so indexes shift; read the cache again after each answer.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LouderAsk {
    pub conversation: i32,
    pub channel: i32,
    pub actor: i32,
    pub motion: Motion,
    pub ask: i32,
}

/// One entry in the event queue.
/// The queue is polled from index 0 upwards and emptied with LouderContext::flush_events(super::context::LouderContext).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct LouderEvent {
    pub conversation: i32,
    pub channel: i32,
    pub actor: i32,
    pub data: EventData,
}

/// Every arm of a union, sharing one block of memory.
///
/// Only `Copy` is derived. `Debug` cannot be, because a union has no
/// single value to describe, and `Default` has nothing to pick.
#[repr(C)]
#[derive(Clone, Copy)]
pub union LouderOption_LouderAsk_Body {
    pub some: LouderAsk,
}

/// Custom Option type exposed to the C FFI
/// `Debug` and `Default` are written out below rather than derived:
/// the body is a union, which has neither, and the tag has no sensible
/// zero.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LouderOption_LouderAsk {
    pub tag: LouderOption_LouderAsk_Tag,
    pub body: LouderOption_LouderAsk_Body,
}

/// The arm a LouderOption_LouderAsk carries, read out of its shared memory.
#[derive(Clone, Copy, Debug)]
pub enum LouderOption_LouderAskArm {
}

impl std::fmt::Debug for LouderOption_LouderAsk {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.arm() {
            None => write!(out, concat!(stringify!(LouderOption_LouderAsk), " {{ tag: {:?} }}"), self.tag),
        }
    }
}

impl Default for LouderOption_LouderAsk {
    /// The zeroed union.
    ///
    /// Which arm that is depends on the engine, so this is only useful as
    /// something to write into rather than as a value.
    fn default() -> Self {
        LouderOption_LouderAsk {
            tag: unsafe { std::mem::zeroed() },
            body: unsafe { std::mem::zeroed() },
        }
    }
}

impl LouderOption_LouderAsk {
    /// The arm the tag names, or `None` for an arm that carries nothing.
    ///
    /// A tag this build does not know about is `None` too, rather than
    /// being read as whichever arm happens to share its bits.
    pub fn arm(&self) -> Option<LouderOption_LouderAskArm> {
        match self.tag {
            LouderOption_LouderAsk_Tag::None_LouderAsk => None,
            LouderOption_LouderAsk_Tag::Some_LouderAsk => None,
        }
    }
}

/// Every arm of a union, sharing one block of memory.
///
/// Only `Copy` is derived. `Debug` cannot be, because a union has no
/// single value to describe, and `Default` has nothing to pick.
#[repr(C)]
#[derive(Clone, Copy)]
pub union LouderOption_LouderEvent_Body {
    pub some: LouderEvent,
}

/// Custom Option type exposed to the C FFI
/// `Debug` and `Default` are written out below rather than derived:
/// the body is a union, which has neither, and the tag has no sensible
/// zero.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct LouderOption_LouderEvent {
    pub tag: LouderOption_LouderEvent_Tag,
    pub body: LouderOption_LouderEvent_Body,
}

/// The arm a LouderOption_LouderEvent carries, read out of its shared memory.
#[derive(Clone, Copy, Debug)]
pub enum LouderOption_LouderEventArm {
}

impl std::fmt::Debug for LouderOption_LouderEvent {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.arm() {
            None => write!(out, concat!(stringify!(LouderOption_LouderEvent), " {{ tag: {:?} }}"), self.tag),
        }
    }
}

impl Default for LouderOption_LouderEvent {
    /// The zeroed union.
    ///
    /// Which arm that is depends on the engine, so this is only useful as
    /// something to write into rather than as a value.
    fn default() -> Self {
        LouderOption_LouderEvent {
            tag: unsafe { std::mem::zeroed() },
            body: unsafe { std::mem::zeroed() },
        }
    }
}

impl LouderOption_LouderEvent {
    /// The arm the tag names, or `None` for an arm that carries nothing.
    ///
    /// A tag this build does not know about is `None` too, rather than
    /// being read as whichever arm happens to share its bits.
    pub fn arm(&self) -> Option<LouderOption_LouderEventArm> {
        match self.tag {
            LouderOption_LouderEvent_Tag::None_LouderEvent => None,
            LouderOption_LouderEvent_Tag::Some_LouderEvent => None,
        }
    }
}

/// Flag bits describing a speaker.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct SpeakerFlags {
    pub bits: i32,
}

/// The opaque handle to the engine.
/// Obtained from crate::init_louder and released with crate::deinit_louder. The contents are internal to the engine and not part of the public API.
/// Story state is held here rather than on Conversation so that the shared buckets survive loading and unloading individual conversations.
/// The engine's whole vocabulary is reached through this one type: names, conversations, actors, hubs, speakers, story state, the event queue, and the ask cache.
#[repr(C)]
#[derive(Debug)]
pub struct LouderContext {
    _opaque: [u8; 0],
}

/// An error code returned by load operations.
/// Standard JSON syntax errors are returned, as well as errors relating to the shape of the loaded document.
pub type LoadError = i32;

/// An actor participating in conversations.
/// Ids are assigned per conversation by crate::add_actor and are the canonical iteration order for the `next_actor_*` functions.
pub type LouderActor = i32;

/// A thing the host is asked to run when an actor first reaches a hub.
/// This is the interned copy of the callback's own text, which may be the text of its arguments as well.
pub type LouderCallback = i32;

/// A snippet of code, identified by the interned copy of its own source text.
/// Expressions are stored as their own text rather than as parsed trees, because a conversation usually contains far more of them than any one run ever reaches.
pub type LouderExpression = i32;

/// A name stored in the engine.
/// Names map bytes to integers. Obtain ids with crate::get_named_id and crate::declare_name, and read them back with crate::read_name, which hands over the bytes that went in. Conversations are id'd by names containing the file they were loaded from.
pub type LouderName = i32;

/// A position within a conversation network of hubs.
/// Positions are chosen by the editor and are sparse: a conversation may hold 65, 68, 71, and 75 with nothing in between. Iteration over positions therefore walks hubs in ascending position order rather than counting.
pub type LouderPosition = i32;


// --- constants --------------------------------------------------

pub const ACTOR_FLAGS_SPECTATOR: ActorFlags = ActorFlags { bits: 1 };
pub const ACTOR_FLAGS_DEBUG: ActorFlags = ActorFlags { bits: 2 };
pub const DIALOGUE_RULES_SPOKE: DialogueRules = DialogueRules { bits: 1 };
pub const DIALOGUE_RULES_SINGLEPLAYER: DialogueRules = DialogueRules { bits: 2 };
pub const HUB_FLAGS_AUTO: HubFlags = HubFlags { bits: 1 };
pub const HUB_FLAGS_ASK: HubFlags = HubFlags { bits: 2 };
pub const SPEAKER_FLAGS_PUBLIC: SpeakerFlags = SpeakerFlags { bits: 1 };
pub const LOUDER_ACTOR_NONE: i32 = -1;
pub const LOUDER_LOAD_ALREADY_LOADED: i32 = 4;
pub const LOUDER_LOAD_MALFORMED_JSON: i32 = 1;
pub const LOUDER_LOAD_NOT_AN_OBJECT: i32 = 3;
pub const LOUDER_LOAD_NOT_A_LIST: i32 = 2;
pub const LOUDER_LOAD_OK: i32 = 0;
pub const LOUDER_NAME_NONE: i32 = -1;

// --- entry points ---------------------------------------------------

// The engine is a plain C ABI, so the whole surface is one link against
// one library. `build.rs` points the linker at wherever cargo left it.
#[link(name = "louder_rust")]
unsafe extern "C" {
    /// Add an actor to a conversation and return the integer id'ing it.
    /// The actor starts at `start_position` on the default channel, with priority 0. Ids are per conversation and are never reused, so ascending order is a stable iteration order.
    /// - `speak_as` and `listen_to` are arrays of `count` names; either may be null when its count is 0. - ActorFlags::SPECTATOR and ActorFlags::DEBUG actors are given no channel or priority.
    /// Emits an EventData::ActorAdded event on success. Returns the new actor id, or LOUDER_ACTOR_NONE if the conversation is not loaded or a pointer was null.
    pub fn add_actor(ctx: *mut LouderContext, conversation: i32, start_position: i32, flags: ActorFlags, speak_as: *const i32, speak_as_count: i32, listen_to: *const i32, listen_to_count: i32) -> i32;
    /// Answer the ask at `index` with `verdict`.
    /// Answering marks the ask complete and moves it to the back of the cache, so indexes shift: read get_ask again after each answer. Answered asks stay readable but no longer accept answers, so a host can simply keep answering until LouderError::Missing comes back and the cache has flushed itself.
    /// Returns LouderError::Ok if the answer was recorded, or LouderError::Missing if `index` was out of range or that ask had already been answered.
    pub fn answer_ask(ctx: *mut LouderContext, index: usize, verdict: i32) -> LouderError;
    /// Remove all speakers in the config.
    /// Returns 0 on success, or 1 if `ctx` was null. Conversations and actors are untouched, so hubs naming a now-missing speaker become public again.
    pub fn clear_config(ctx: *mut LouderContext) -> i32;
    /// Assign the id of a string.
    /// If the string was already declared, returns LOUDER_NAME_NONE and leaves the existing id alone. Otherwise a new id is assigned and returned.
    pub fn declare_name(ctx: *mut LouderContext, buffer: *const u8, max_size: usize) -> i32;
    /// Release a context and everything it holds.
    /// Returns LouderError::Ok on success, or LouderError::Missing if `ctx` was null. Every conversation, speaker, actor, and queued event goes with it.
    pub fn deinit_louder(ctx: *mut LouderContext) -> LouderError;
    /// Remove every queued event.
    /// The queue is indexed from 0 again afterwards, so the next get_event with index 0 reads the first event queued after this call.
    pub fn flush_events(ctx: *mut LouderContext) -> ();
    /// Move an actor to another position directly.
    /// - `fake_priority` is unused if `0`, otherwise overrides the priority of the destination. Because 0 is the ordinary default priority, this cannot force an actor's priority *to* 0 over a hub that sets one. - `fake_channel` is unused if LOUDER_NAME_NONE, otherwise overrides the channel of the destination.
    /// Returns 0 on success, or 1 if the conversation, actor, or destination was missing, or if an auto hub ran out of choices.
    pub fn force_position(ctx: *mut LouderContext, conversation: i32, actor: i32, target_position: i32, fake_channel: i32, fake_priority: i32) -> i32;
    /// Move an actor to another position as if it had moved through the network.
    /// - `fake_priority` is unused if `0`, otherwise overrides the priority of the destination. Because 0 is the ordinary default priority, this cannot force an actor's priority *to* 0 over a hub that sets one. - `fake_channel` is unused if LOUDER_NAME_NONE, otherwise overrides the channel of the destination.
    /// Other actors sharing the actor's channel may be pulled along, exactly as if the actor had chosen. Returns 0 on success, or 1 if the conversation, actor, or destination was missing, or if an auto hub ran out of choices.
    pub fn force_warp(ctx: *mut LouderContext, conversation: i32, actor: i32, target_position: i32, fake_channel: i32, fake_priority: i32) -> i32;
    /// Get an actor's additional data under `key`.
    /// Returns a zero variant if the actor does not exist or has nothing stored under `key`.
    pub fn get_actor_additional(ctx: *const LouderContext, conversation: i32, actor: i32, key: i32) -> LouderVariant;
    /// Get the channel of an actor.
    /// Actors with ActorFlags::SPECTATOR or ActorFlags::DEBUG are not part of the flow and read as LOUDER_NAME_NONE.
    pub fn get_actor_channel(ctx: *const LouderContext, conversation: i32, actor: i32) -> i32;
    /// Get the flags of an actor.
    /// Returns ActorFlags::empty if the actor does not exist.
    pub fn get_actor_flags(ctx: *const LouderContext, conversation: i32, actor: i32) -> ActorFlags;
    /// Get the position of an actor.
    /// Returns BLANK_POSITION if the actor does not exist.
    pub fn get_actor_position(ctx: *const LouderContext, conversation: i32, actor: i32) -> i32;
    /// Get the priority of an actor.
    /// Actors outside the flow read as 0.
    pub fn get_actor_priority(ctx: *const LouderContext, conversation: i32, actor: i32) -> i32;
    /// Get the ask at `index`.
    /// Asks are not in a stable order, so read them until one comes back LouderOption::None rather than assuming the queue only grows.
    pub fn get_ask(ctx: *const LouderContext, index: usize) -> LouderOption_LouderAsk;
    /// Read the dialogue rules currently in effect.
    /// A fresh context has every rule on. Returns an empty set if `ctx` was null, since a null context is not running any dialogue.
    pub fn get_dialogue_rules(ctx: *const LouderContext) -> DialogueRules;
    /// Get the event at `index`.
    /// Returns LouderOption::None once `index` is past the end of the queue, or if `ctx` is null. Poll from index 0 upwards until that happens, then call flush_events.
    pub fn get_event(ctx: *const LouderContext, index: usize) -> LouderOption_LouderEvent;
    /// Get a hub's additional data under `key`.
    /// Returns a zero variant if the hub does not exist or has nothing stored under `key`.
    pub fn get_hub_additional(ctx: *const LouderContext, conversation: i32, position: i32, key: i32) -> LouderVariant;
    /// Get the callbacks associated with a hub as a name.
    /// The schema stores callbacks newline separated; this returns the whole block as one name. Returns LOUDER_NAME_NONE if the hub has none.
    pub fn get_hub_callbacks(ctx: *const LouderContext, conversation: i32, position: i32) -> i32;
    /// Get the channel of a hub.
    /// Returns LOUDER_NAME_NONE if the hub does not change the channel, in which case an actor keeps whatever channel it already had.
    pub fn get_hub_channel(ctx: *const LouderContext, conversation: i32, position: i32) -> i32;
    /// Get the number of choices on a hub from an actor's perspective.
    /// Only choices the actor could actually take are counted, so this applies the visibility, recipient, and spoke rules. Returns 0 for an unknown hub or actor.
    pub fn get_hub_choice_count(ctx: *const LouderContext, conversation: i32, position: i32, actor_perspective: i32) -> i32;
    /// Get the position a choice on a hub links to.
    /// Returns BLANK_POSITION if the choice does not exist or is not available to `actor_perspective`.
    pub fn get_hub_choice_link(ctx: *const LouderContext, conversation: i32, position: i32, actor_perspective: i32, choice_index: i32) -> i32;
    /// Get the text of a choice on a hub as a name.
    /// A choice has no text of its own, so this is the text of the hub it links to. Returns LOUDER_NAME_NONE if the choice does not exist or is not available to `actor_perspective`.
    pub fn get_hub_choice_text(ctx: *const LouderContext, conversation: i32, position: i32, actor_perspective: i32, choice_index: i32) -> i32;
    /// Get the expressions associated with a hub as a name.
    /// The schema stores expressions newline separated; this returns the whole block as one name, so the caller splits it to get individual expressions. Returns LOUDER_NAME_NONE if the hub has none.
    pub fn get_hub_expressions(ctx: *const LouderContext, conversation: i32, position: i32) -> i32;
    /// Get the flags of a hub.
    /// Returns HubFlags::empty if the hub does not exist.
    pub fn get_hub_flags(ctx: *const LouderContext, conversation: i32, position: i32) -> HubFlags;
    /// Get the priority of a hub.
    /// Returns 0 if the hub does not exist, which is also the default priority.
    pub fn get_hub_priority(ctx: *const LouderContext, conversation: i32, position: i32) -> i32;
    /// Get the speaker of a hub.
    /// Returns LOUDER_NAME_NONE if the hub has no speaker, which means it is visible to everyone, or if the hub does not exist.
    pub fn get_hub_speaker(ctx: *const LouderContext, conversation: i32, position: i32) -> i32;
    /// Get the text of a hub as a name.
    /// Returns LOUDER_NAME_NONE if the hub has no text or does not exist. Read the string back with read_name.
    pub fn get_hub_text(ctx: *const LouderContext, conversation: i32, position: i32) -> i32;
    /// Get the id of a string.
    /// Returns LOUDER_NAME_NONE if the string was never declared, or if either pointer was null.
    /// - `buffer`: the string bytes to look up. - `max_size`: the number of bytes to read from `buffer`.
    pub fn get_named_id(ctx: *const LouderContext, buffer: *const u8, max_size: usize) -> i32;
    /// Iterate the speakers that `speaker` is listening to.
    /// Pass LOUDER_NAME_NONE as `start` to begin, then pass the previous result until LOUDER_NAME_NONE comes back. Returns LOUDER_NAME_NONE for an unknown speaker.
    pub fn get_next_speaker_listen_to(ctx: *const LouderContext, speaker: i32, start: i32) -> i32;
    /// Get a speaker's additional data under `key`.
    /// The speaker config is a singleton rather than a per-conversation object, so this takes no conversation. Returns a zero variant if the speaker is unknown or has nothing stored under `key`.
    pub fn get_speaker_additional(ctx: *const LouderContext, speaker: i32, key: i32) -> LouderVariant;
    /// Get the flags of a speaker.
    /// A speaker that has not been declared reads as SpeakerFlags::empty, which is public by default, so a hub naming it still appears to everyone.
    pub fn get_speaker_flags(ctx: *const LouderContext, speaker: i32) -> SpeakerFlags;
    /// Get a story state value under `state`.
    /// The variable's own name decides where it is stored: a leading underscore means it is not saved, and an uppercase first letter means it is shared across conversations. Variables that have never been written read as 0. Reading from a conversation that is not loaded also reads as 0.
    pub fn get_story_state(ctx: *const LouderContext, conversation: i32, state: i32) -> LouderVariant;
    /// Initialize the engine and return a new context.
    /// Initializes the name table, the event queue, and the ask cache. Release the returned handle with deinit_louder. Returns null only if allocation failed.
    pub fn init_louder() -> *mut LouderContext;
    /// Load the conversation config from a JSON document in `buffer`.
    /// - The bytes have to be UTF-8 and the error scheme is the one in load_conversation_json, with speakers as the target objects instead of hubs. - The config is a singleton as opposed to a named conversation instance, so this takes no conversation name. - Objects are speakers and their configurations. A speaker already in the config is replaced rather than treated as an error.
    pub fn load_conversation_config(ctx: *mut LouderContext, buffer: *mut u8, max_size: usize) -> i32;
    /// Load a conversation from a JSON document in `buffer`.
    /// The buffer holds `max_size` bytes. JSON is text, so those bytes have to be UTF-8 for the engine to read them; LOUDER_LOAD_MALFORMED_JSON is returned if they are not, and re-encoding them is the host's job rather than the engine's.
    /// - The JSON is expected in array form; LOUDER_LOAD_NOT_A_LIST is returned otherwise. - Standard JSON syntax errors are returned as LOUDER_LOAD_MALFORMED_JSON. - Standard properties are left default or read from matching JSON fields. - Additional JSON fields are treated as additional data on hubs. - LOUDER_LOAD_ALREADY_LOADED is returned if `conversation` is already occupied by another loaded conversation. - Problems with individual records do not fail the load. They are emitted as events instead, carrying the offending hub's position in Motion::source_position and the LoadError in Motion::target_position.
    pub fn load_conversation_json(ctx: *mut LouderContext, conversation: i32, buffer: *mut u8, max_size: usize) -> i32;
    /// Build an EventData::ActorAdded.
    pub fn make_event_data_actor_added() -> EventData;
    /// Build an EventData::ActorRemoved.
    pub fn make_event_data_actor_removed() -> EventData;
    /// Build an EventData::AdditionalDataChange.
    pub fn make_event_data_additional_data_change(change: AdditionalDataChange) -> EventData;
    /// Build an EventData::Callback, naming the callback.
    pub fn make_event_data_callback(callback: i32) -> EventData;
    /// Build an EventData::ChannelChange.
    pub fn make_event_data_channel_change() -> EventData;
    /// Build an EventData::ChoiceAppearance.
    pub fn make_event_data_choice_appearance(appearance: ChoiceAppearance) -> EventData;
    /// Build an EventData::Condition.
    pub fn make_event_data_condition(outcome: ConditionOutcome) -> EventData;
    /// Build an EventData::ConversationAdded, naming the conversation.
    pub fn make_event_data_conversation_added(conversation: i32) -> EventData;
    /// Build an EventData::ConversationRemoved, naming the conversation.
    pub fn make_event_data_conversation_removed(conversation: i32) -> EventData;
    /// Build an EventData::ForceMotion.
    pub fn make_event_data_force_motion(motion: Motion) -> EventData;
    /// Build an EventData::LoadError.
    /// The engine emits these itself when it cannot read a record of a loaded document, so this exists to round out the set rather than because a host has a reason to raise one.
    pub fn make_event_data_load_error(failure: LoadFailure) -> EventData;
    /// Build the EventData::None variant, which carries nothing.
    pub fn make_event_data_none() -> EventData;
    /// Build an EventData::RegularMotion.
    pub fn make_event_data_regular_motion(motion: Motion) -> EventData;
    /// Build an EventData::SpeakerAdded, naming the speaker.
    pub fn make_event_data_speaker_added(speaker: i32) -> EventData;
    /// Build an EventData::SpeakerChange, naming the speaker.
    pub fn make_event_data_speaker_change(speaker: i32) -> EventData;
    /// Build an EventData::SpeakerRemoved, naming the speaker.
    pub fn make_event_data_speaker_removed(speaker: i32) -> EventData;
    /// Build an EventData::Statement, naming the expression.
    pub fn make_event_data_statement(expression: i32) -> EventData;
    /// Build an EventData::TextChange, naming the text.
    pub fn make_event_data_text_change(text: i32) -> EventData;
    /// Build an EventData::Warp.
    pub fn make_event_data_warp(motion: Motion) -> EventData;
    /// Build a LouderOption::Some holding an ask.
    pub fn make_some_ask(value: LouderAsk) -> LouderOption_LouderAsk;
    /// Build a LouderOption::Some holding an event.
    pub fn make_some_event(value: LouderEvent) -> LouderOption_LouderEvent;
    /// Build a boolean as a LouderVariant::Int of 1 or 0.
    pub fn make_variant_bool(value: bool) -> LouderVariant;
    /// Build a LouderVariant::Float.
    pub fn make_variant_float(value: f32) -> LouderVariant;
    /// Build a LouderVariant::Int.
    pub fn make_variant_int(value: i32) -> LouderVariant;
    /// Build a LouderVariant::String, naming the string rather than carrying it.
    pub fn make_variant_string(value: i32) -> LouderVariant;
    /// Build the LouderVariant::Int of 0 that an unwritten variable reads as.
    pub fn make_variant_zero() -> LouderVariant;
    /// Iterate the actors matching certain flags.
    /// With `exact` nonzero the flags must match exactly; otherwise every bit in `include` must be present. Pass LOUDER_ACTOR_NONE as `start` to begin.
    pub fn next_actor_from_flags(ctx: *const LouderContext, conversation: i32, include: ActorFlags, exact: i32, start: i32) -> i32;
    /// Iterate the actors in a channel.
    /// Pass LOUDER_ACTOR_NONE as `start` to begin. Actors outside the flow have no channel and are never returned.
    pub fn next_actor_in_channel(ctx: *const LouderContext, conversation: i32, channel: i32, start: i32) -> i32;
    /// Iterate the actors in a channel at a specific position.
    /// Pass LOUDER_ACTOR_NONE as `start` to begin.
    pub fn next_actor_in_channel_and_position(ctx: *const LouderContext, conversation: i32, channel: i32, position: i32, start: i32) -> i32;
    /// Iterate the speakers an actor listens to.
    /// Includes the actor's `listen_to` speakers plus the listening speakers of the speakers it speaks as. Pass LOUDER_NAME_NONE as `start` to begin.
    pub fn next_actor_listening(ctx: *const LouderContext, conversation: i32, actor: i32, start: i32) -> i32;
    /// Iterate the base speakers an actor listens to.
    /// These are the actor's own `listen_to` speakers, without the ones reachable through who it speaks as. See next_actor_listening for the wider set.
    pub fn next_actor_listening_base(ctx: *const LouderContext, conversation: i32, actor: i32, start: i32) -> i32;
    /// Iterate the actors listening to a speaker.
    /// This covers `listen_to` plus the listening actors derived from the listening speakers of the speakers an actor speaks as. Pass LOUDER_ACTOR_NONE as `start` to begin.
    pub fn next_actor_listening_to(ctx: *const LouderContext, conversation: i32, speaker: i32, start: i32) -> i32;
    /// Iterate the speakers an actor speaks as.
    /// Pass LOUDER_NAME_NONE as `start` to begin, then pass the previous result until LOUDER_NAME_NONE comes back.
    pub fn next_actor_speak_as(ctx: *const LouderContext, conversation: i32, actor: i32, start: i32) -> i32;
    /// Iterate the actors speaking as a speaker.
    /// Pass LOUDER_ACTOR_NONE as `start` to begin.
    pub fn next_actor_speaking_as(ctx: *const LouderContext, conversation: i32, speaker: i32, start: i32) -> i32;
    /// Iterate the hubs in a channel.
    /// Hubs that name no channel belong to the default channel, so searching for it also finds them. Pass BLANK_POSITION as `start` to begin.
    pub fn next_hub_by_channel(ctx: *const LouderContext, conversation: i32, channel: i32, start: i32) -> i32;
    /// Iterate the hubs with a priority within `[priority_min, priority_max]`.
    /// The bounds are inclusive. Pass BLANK_POSITION as `start` to begin.
    pub fn next_hub_by_priority(ctx: *const LouderContext, conversation: i32, priority_min: i32, priority_max: i32, start: i32) -> i32;
    /// Iterate the hubs matching certain flags.
    /// With `exact` nonzero the flags must match exactly; otherwise every bit in `include` must be present. Pass BLANK_POSITION as `start` to begin.
    pub fn next_hub_from_flags(ctx: *const LouderContext, conversation: i32, include: HubFlags, exact: i32, start: i32) -> i32;
    /// Iterate the hubs containing all, or exactly, the given tags.
    /// With `exact` nonzero the hub must carry exactly the listed tags; otherwise carrying extra tags is fine. Pass BLANK_POSITION as `start` to begin.
    pub fn next_hub_from_tags(ctx: *const LouderContext, conversation: i32, include: *const i32, include_count: i32, exact: i32, start: i32) -> i32;
    /// Iterate the hubs heard by a speaker.
    /// This searches the speaker's own listening speakers as well as the speaker itself, which are the hubs that may appear to an actor speaking as this speaker. Pass BLANK_POSITION as `start` to begin.
    pub fn next_hub_heard_by(ctx: *const LouderContext, conversation: i32, speaker: i32, start: i32) -> i32;
    /// Iterate the recipients of a hub.
    /// Returns LOUDER_NAME_NONE when the hub has no recipients, which means the hub is not restricted. Pass LOUDER_NAME_NONE as `start` to begin.
    pub fn next_hub_recipient(ctx: *const LouderContext, conversation: i32, position: i32, start: i32) -> i32;
    /// Iterate the hubs spoken by a speaker.
    /// Pass BLANK_POSITION as `start` to begin.
    pub fn next_hub_spoken_by(ctx: *const LouderContext, conversation: i32, speaker: i32, start: i32) -> i32;
    /// Iterate the tags of a position.
    /// Pass LOUDER_NAME_NONE as `start` to begin.
    pub fn next_hub_tag(ctx: *const LouderContext, conversation: i32, position: i32, start: i32) -> i32;
    /// Push an ask onto the ask cache, used for testing.
    /// Returns LouderError::Ok on success, or LouderError::Missing if either pointer was null.
    pub fn push_ask(ctx: *mut LouderContext, ask: *const LouderAsk) -> LouderError;
    /// Read the bytes associated with an id into `buffer`.
    /// The bytes come back exactly as they went in, so a name declared in some encoding is read in that same one. Decoding them is the host's job, since the engine never displays anything itself.
    /// Writes 0 bytes and a terminator if no name is associated with the id, or if either pointer was null. Returns the number of bytes written, excluding the terminator.
    pub fn read_name(ctx: *const LouderContext, name: i32, buffer: *mut u8, max_size: usize) -> usize;
    /// Serialize the story state, along with the positions and details of every actor, in JSON into `buffer`.
    /// The document has a `state` object and an `actors` array, both ordered so that the output is reproducible.
    /// The document is written as UTF-8. Returns the number of bytes written excluding the terminator; a negative value means the conversation is not loaded.
    pub fn read_snapshot_json(ctx: *const LouderContext, conversation: i32, buffer: *mut u8, max_size: usize) -> i32;
    /// Serialize the story state in JSON into `buffer`.
    /// Every bucket is included: the conversation's own saved and unsaved variables, and the shared ones. Keys are written in ascending id order so the output is reproducible.
    /// The document is written as UTF-8. Returns the number of bytes written excluding the terminator; a negative value means the conversation is not loaded.
    pub fn read_state_json(ctx: *const LouderContext, conversation: i32, buffer: *mut u8, max_size: usize) -> i32;
    /// Remove an actor from a conversation.
    /// Returns 0 on success, or 1 if the actor or conversation did not exist. Emits an EventData::ActorRemoved event on success.
    pub fn remove_actor(ctx: *mut LouderContext, conversation: i32, actor: i32) -> i32;
    /// Remove a conversation and all associated actors.
    /// Returns 0 on success, or 1 if the conversation was not loaded. Shared story state is kept, since it outlives any one conversation.
    pub fn remove_conversation(ctx: *mut LouderContext, conversation: i32) -> i32;
    /// Select a choice on an actor's current hub.
    /// The choice must be one the actor could actually take; see get_hub_choice_count for what that means. Auto hubs along the way are resolved by taking the first choice available at each one.
    /// Returns 0 on success, or 1 if the actor, hub, or choice was missing, if the choice is not available to this actor, or if an auto hub ran out of choices.
    pub fn select_choice(ctx: *mut LouderContext, conversation: i32, actor: i32, choice_index: i32) -> i32;
    /// Replace the dialogue rules.
    /// A rule takes effect on the next move rather than being applied to the conversation as it stands, so a host can turn one off while an actor is standing on a hub it would otherwise resolve through.
    /// Bits outside DialogueRules are ignored rather than refused, so a host carrying a configuration written for a newer engine still gets the rules this build knows about.
    pub fn set_dialogue_rules(ctx: *mut LouderContext, rules: DialogueRules) -> i32;
    /// Set a story state value.
    /// The variable's own name decides where it is stored, as in get_story_state. Doing nothing if `ctx` is null.
    pub fn set_story_state(ctx: *mut LouderContext, conversation: i32, key: i32, value: LouderVariant) -> ();
    pub fn test_linkage() -> i32;
    /// Push an event onto the event queue, used for testing.
    /// The caller fills in the event themselves, so nothing about `code` is interpreted here. Returns LouderError::Ok on success, or LouderError::Missing if either pointer was null.
    pub fn test_push_event(ctx: *mut LouderContext, event: *const LouderEvent) -> LouderError;
}
