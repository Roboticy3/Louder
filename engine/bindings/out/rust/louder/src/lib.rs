//! Bindings for Rust, generated from the C header of the Louder engine.
//! 
//! Do not edit. Run `python engine/bindings/generate.py` and this file comes back.
//! The Louder engine, from Rust.
//!
//! A [`Louder`] owns one engine context and everything that comes with it:
//! the name table, every conversation, and the event queue. Closing it
//! releases all of that at once.
//!
//! Strings cross the boundary as bytes and are decoded here as UTF-8, which
//! is the encoding the engine reads JSON in. Anything else is handed over
//! untouched.

use std::ffi::CString;
use std::ptr;

pub mod ffi;

use ffi::*;

/// Why a document was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Failure {
    /// Why a document was refused. Zero means it was not refused.
    Ok,
    /// Why a document was refused. Zero means it was not refused.
    MalformedJson,
    /// Why a document was refused. Zero means it was not refused.
    NotAList,
    /// Why a document was refused. Zero means it was not refused.
    NotAnObject,
    /// Why a document was refused. Zero means it was not refused.
    AlreadyLoaded,
    /// A code this build does not know about.
    ///
    /// What happens when a host carries a document written for a newer engine.
    Other(i32),
}

impl std::fmt::Display for Failure {
    fn fmt(&self, out: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(out, "{:?}", self)
    }
}

impl std::error::Error for Failure {}

/// One thing that happened, with the names in it already decoded.
#[derive(Debug, Clone)]
pub struct Event {
    /// The conversation it happened in.
    pub conversation: String,
    /// The channel it happened on.
    pub channel: String,
    /// The actor it happened to, or `ACTOR_NONE`.
    pub actor: i32,
    /// Which of the engine's event kinds this is.
    pub kind: &'static str,
    /// Whatever that kind carries, or `None` where it carries nothing.
    pub value: Option<EventValue>,
}

/// The arms of an event, as values rather than as memory.
#[derive(Debug, Clone)]
pub enum EventValue {
    /// One move of an actor from one hub to another.
    Motion {
        source_position: i32,
        target_position: i32,
        choice_index: i32,
        background: bool,
    },
    /// A choice becoming available on a hub.
    ChoiceAppearance { position: i32, choice_index: i32 },
    /// An arm that carries a name, already decoded.
    Name(String),
    /// An arm this build does not know how to read.
    Other,
}

const NAME_BUFFER: usize = 4096;
const JSON_BUFFER: usize = 1 << 16;

/// A buffer the engine fills, growing until the whole answer fits.
///
/// The engine truncates rather than refusing and reports how much fitted, so
/// the only way to be sure of the whole answer is to ask again with more room.
/// Two calls is the normal case; the loop is what makes a long document safe.
fn read_into(call: impl Fn(*mut u8, usize) -> isize, mut size: usize) -> Result<Vec<u8>, Failure> {
    loop {
        let mut buffer = vec![0u8; size];
        let written = call(buffer.as_mut_ptr(), size - 1);
        if written < 0 {
            return Err(Failure::Other(written as i32));
        }
        if written < (size - 1) as isize {
            buffer.truncate(written as usize);
            return Ok(buffer);
        }
        size *= 4;
    }
}

/// Text and its length, which is how every string crosses the boundary.
///
/// `CString` is used because the engine wants a NUL-terminated buffer even where
/// it is also told the length, and because a `String` may contain an interior
/// NUL that would otherwise truncate silently.
fn with_bytes(text: &str) -> Result<(CString, usize), Failure> {
    let raw = CString::new(text).map_err(|_| Failure::Other(-1))?;
    let length = raw.as_bytes().len();
    Ok((raw, length))
}

/// A borrowed buffer as the engine wants to see it.
///
/// The engine only reads through the ones handed to `get_named_id` and
/// `declare_name`, which is what the `const` in the header says. Casting here
/// rather than at each call site keeps that in one place, and the `CString` owns
/// the bytes for as long as the call needs them.
fn as_read_only(raw: &CString) -> *mut u8 {
    raw.as_ptr() as *mut u8
}

/// A list of name ids as the array the engine reads.
///
/// Empty is a one-element array rather than a null pointer, because the engine
/// accepts either and the count is what says how long it is.
fn name_array(ids: &[i32]) -> Vec<i32> {
    if ids.is_empty() { vec![0] } else { ids.to_vec() }
}

/// One engine context.
///
/// Owns the name table, every conversation loaded into it, and the event queue.
/// Closing it, or dropping it, releases all of that at once.
pub struct Louder {
    ctx: *mut LouderContext,
}

impl Louder {
    /// Open a context. The engine is linked at build time; see `build.rs`.
    pub fn new() -> Result<Louder, Failure> {
        let ctx = unsafe { ffi::init_louder() };
        if ctx.is_null() {
            return Err(Failure::Other(-1));
        }
        Ok(Louder { ctx })
    }

    /// Release the context and everything it holds.
    pub fn close(&mut self) {
        if !self.ctx.is_null() {
            unsafe { ffi::deinit_louder(self.ctx) };
            self.ctx = ptr::null_mut();
        }
    }

    /// The raw context pointer, for anything this layer does not wrap.
    pub fn handle(&self) -> *mut LouderContext {
        self.ctx
    }

    // --- names ---

    /// The id of some text, or `None` if it was never declared.
    pub fn lookup(&mut self, text: &str) -> Option<i32> {
        let (raw, length) = with_bytes(text).ok()?;
        let found = unsafe { ffi::get_named_id(self.ctx, as_read_only(&raw), length) };
        if found == LOUDER_NAME_NONE { None } else { Some(found) }
    }

    /// The id of some text, declaring it the first time.
    pub fn declare(&mut self, text: impl AsRef<str>) -> i32 {
        let text = text.as_ref();
        if let Some(found) = self.lookup(text) {
            return found;
        }
        let (raw, length) = match with_bytes(text) {
            Ok(pair) => pair,
            Err(_) => return LOUDER_NAME_NONE,
        };
        unsafe { ffi::declare_name(self.ctx, as_read_only(&raw), length) }
    }

    /// The text behind an id, decoded as UTF-8.
    pub fn text(&mut self, name: i32) -> String {
        if name == LOUDER_NAME_NONE {
            return String::new();
        }
        let mut size = NAME_BUFFER;
        loop {
            let mut buffer = vec![0u8; size];
            let got = unsafe { ffi::read_name(self.ctx, name, buffer.as_mut_ptr(), size - 1) };
            if got == 0 {
                return String::new();
            }
            if got < size - 1 {
                buffer.truncate(got);
                return String::from_utf8_lossy(&buffer).into_owned();
            }
            size *= 4;
        }
    }

    // --- documents ---

    /// Load a conversation from a JSON document.
    pub fn load_conversation(&mut self, conversation: &str, document: &str) -> Result<(), Failure> {
        let name = self.declare(conversation);
        let raw = CString::new(document).map_err(|_| Failure::Other(-1))?;
        let code = unsafe {
            ffi::load_conversation_json(self.ctx, name, raw.as_ptr() as *mut u8, raw.as_bytes().len())
        };
        Self::reason(code).map(|_| ())
    }

    /// Load the speaker config, which is a singleton.
    pub fn load_config(&mut self, document: &str) -> Result<(), Failure> {
        let raw = CString::new(document).map_err(|_| Failure::Other(-1))?;
        let code =
            unsafe { ffi::load_conversation_config(self.ctx, raw.as_ptr() as *mut u8, raw.as_bytes().len()) };
        Self::reason(code).map(|_| ())
    }

    /// Remove every speaker. Conversations are left alone.
    pub fn clear_config(&mut self) -> bool {
        unsafe { ffi::clear_config(self.ctx) == 0 }
    }

    /// Remove a conversation and the actors in it.
    pub fn remove_conversation(&mut self, conversation: &str) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::remove_conversation(self.ctx, name) == 0 }
    }

    /// The code a load returned, as whatever it means.
    ///
    /// The constants are the ones the header declares rather than the numbers
    /// written out, so a renumbering in the engine is not also something to
    /// chase here.
    fn reason(code: i32) -> Result<(), Failure> {
        match code {
            LOUDER_LOAD_OK => Ok(()),
            LOUDER_LOAD_MALFORMED_JSON => Err(Failure::MalformedJson),
            LOUDER_LOAD_NOT_A_LIST => Err(Failure::NotAList),
            LOUDER_LOAD_NOT_AN_OBJECT => Err(Failure::NotAnObject),
            LOUDER_LOAD_ALREADY_LOADED => Err(Failure::AlreadyLoaded),
            other => Err(Failure::Other(other)),
        }
    }

    // --- actors ---

    /// Put an actor into a conversation and return its id.
    pub fn add_actor(
        &mut self,
        conversation: &str,
        position: i32,
        speak_as: &[&str],
        listen_to: &[&str],
    ) -> i32 {
        let name = self.declare(conversation);
        let speaks: Vec<i32> = speak_as.iter().map(|s| self.declare(s)).collect();
        let listens: Vec<i32> = listen_to.iter().map(|s| self.declare(s)).collect();
        // Both vectors have to outlive the call, which is why they are bound
        // before it rather than being temporaries at the end of an argument.
        let speak_ids = name_array(&speaks);
        let listen_ids = name_array(&listens);
        unsafe {
            ffi::add_actor(
                self.ctx,
                name,
                position,
                ActorFlags { bits: 0 },
                speak_ids.as_ptr(),
                speaks.len() as i32,
                listen_ids.as_ptr(),
                listens.len() as i32,
            )
        }
    }

    /// Take an actor out of a conversation.
    pub fn remove_actor(&mut self, conversation: &str, actor: i32) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::remove_actor(self.ctx, name, actor) == 0 }
    }

    /// Where an actor is standing.
    pub fn actor_position(&mut self, conversation: &str, actor: i32) -> i32 {
        let name = self.declare(conversation);
        unsafe { ffi::get_actor_position(self.ctx, name, actor) }
    }

    /// Which channel an actor is on.
    pub fn actor_channel(&mut self, conversation: &str, actor: i32) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_actor_channel(self.ctx, name, actor) })
    }

    /// The priority on an actor.
    pub fn actor_priority(&mut self, conversation: &str, actor: i32) -> i32 {
        let name = self.declare(conversation);
        unsafe { ffi::get_actor_priority(self.ctx, name, actor) }
    }

    /// Take a choice on the hub an actor is standing on.
    pub fn choose(&mut self, conversation: &str, actor: i32, index: i32) -> Result<(), Failure> {
        let name = self.declare(conversation);
        if unsafe { ffi::select_choice(self.ctx, name, actor, index) } == 0 {
            Ok(())
        } else {
            Err(Failure::Other(-1))
        }
    }

    /// Move an actor as though it had walked there.
    pub fn warp(&mut self, conversation: &str, actor: i32, target: i32) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::force_warp(self.ctx, name, actor, target, LOUDER_NAME_NONE, 0) == 0 }
    }

    /// Move an actor straight there.
    pub fn teleport(&mut self, conversation: &str, actor: i32, target: i32) -> bool {
        let name = self.declare(conversation);
        unsafe { ffi::force_position(self.ctx, name, actor, target, LOUDER_NAME_NONE, 0) == 0 }
    }

    // --- hubs ---

    /// The text of a hub.
    pub fn hub_text(&mut self, conversation: &str, position: i32) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_hub_text(self.ctx, name, position) })
    }

    /// The speaker of a hub.
    pub fn hub_speaker(&mut self, conversation: &str, position: i32) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_hub_speaker(self.ctx, name, position) })
    }

    /// How many choices are available to an actor on a hub.
    pub fn hub_choice_count(&mut self, conversation: &str, position: i32, actor: i32) -> i32 {
        let name = self.declare(conversation);
        unsafe { ffi::get_hub_choice_count(self.ctx, name, position, actor) }
    }

    /// The text of one choice, which is the text of the hub it leads to.
    pub fn hub_choice_text(
        &mut self,
        conversation: &str,
        position: i32,
        actor: i32,
        index: i32,
    ) -> String {
        let name = self.declare(conversation);
        self.text(unsafe { ffi::get_hub_choice_text(self.ctx, name, position, actor, index) })
    }

    // --- story state ---

    /// The raw variant behind a story state value.
    pub fn variant(&mut self, conversation: &str, key: &str) -> Option<LouderVariantArm> {
        let name = self.declare(conversation);
        let state = self.declare(key);
        unsafe { ffi::get_story_state(self.ctx, name, state) }.arm()
    }

    /// A story state value as an integer, or zero if there is none.
    pub fn get_int(&mut self, conversation: &str, key: &str) -> i32 {
        match self.variant(conversation, key) {
            Some(LouderVariantArm::int(value)) => value,
            _ => 0,
        }
    }

    /// A story state value as a float, or zero if there is none.
    pub fn get_float(&mut self, conversation: &str, key: &str) -> f32 {
        match self.variant(conversation, key) {
            Some(LouderVariantArm::float(value)) => value,
            _ => 0.0,
        }
    }

    /// A story state value as text, or the empty string if there is none.
    pub fn get_text(&mut self, conversation: &str, key: &str) -> String {
        match self.variant(conversation, key) {
            Some(LouderVariantArm::string(value)) => self.text(value),
            _ => String::new(),
        }
    }

    /// Write an integer story state value.
    pub fn set_int(&mut self, conversation: &str, key: &str, value: i32) {
        let name = self.declare(conversation);
        let state = self.declare(key);
        let variant = unsafe { ffi::make_variant_int(value) };
        unsafe { ffi::set_story_state(self.ctx, name, state, variant) };
    }

    /// Write a float story state value.
    pub fn set_float(&mut self, conversation: &str, key: &str, value: f32) {
        let name = self.declare(conversation);
        let state = self.declare(key);
        let variant = unsafe { ffi::make_variant_float(value) };
        unsafe { ffi::set_story_state(self.ctx, name, state, variant) };
    }

    /// Write a text story state value, stored as the interned name.
    pub fn set_name(&mut self, conversation: &str, key: &str, value: &str) {
        let name = self.declare(conversation);
        let state = self.declare(key);
        let text = self.declare(value);
        let variant = unsafe { ffi::make_variant_string(text) };
        unsafe { ffi::set_story_state(self.ctx, name, state, variant) };
    }

    /// The whole story state as a JSON document.
    pub fn state_json(&mut self, conversation: &str) -> Result<String, Failure> {
        let name = self.declare(conversation);
        let raw = read_into(
            |buffer, size| unsafe { ffi::read_state_json(self.ctx, name, buffer, size) as isize },
            JSON_BUFFER,
        )?;
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    /// The story state and every actor, as a JSON document.
    pub fn snapshot_json(&mut self, conversation: &str) -> Result<String, Failure> {
        let name = self.declare(conversation);
        let raw = read_into(
            |buffer, size| unsafe { ffi::read_snapshot_json(self.ctx, name, buffer, size) as isize },
            JSON_BUFFER,
        )?;
        Ok(String::from_utf8_lossy(&raw).into_owned())
    }

    // --- iteration ---

    /// Every queued event, oldest first, leaving the queue alone.
    ///
    /// Every one of the cursors in the header has the same shape: a `start`, the
    /// thing it returns, and a sentinel for the end of the walk. Here they are
    /// iterators, so a walk reads as a loop and the caller cannot forget to
    /// advance.
    pub fn events(&mut self) -> impl Iterator<Item = Event> + '_ {
        EventIter { engine: self, index: 0 }
    }

    /// Every queued event, oldest first, and empty the queue.
    pub fn drain_events(&mut self) -> Vec<Event> {
        let out: Vec<Event> = self.events().collect();
        unsafe { ffi::flush_events(self.ctx) };
        out
    }

    /// Every hub in a channel, in ascending position order.
    pub fn hubs(&mut self, conversation: &str, channel: &str) -> Vec<i32> {
        let name = self.declare(conversation);
        let on = self.declare(channel);
        let mut out = Vec::new();
        let mut start: i32 = -1;
        loop {
            let found = unsafe { ffi::next_hub_by_channel(self.ctx, name, on, start) };
            if found == -1 {
                return out;
            }
            out.push(found);
            start = found;
        }
    }

    /// Every actor speaking as a speaker.
    pub fn actors_speaking_as(&mut self, conversation: &str, speaker: &str) -> Vec<i32> {
        let name = self.declare(conversation);
        let as_name = self.declare(speaker);
        let mut out = Vec::new();
        let mut start: i32 = LOUDER_ACTOR_NONE;
        loop {
            let found = unsafe { ffi::next_actor_speaking_as(self.ctx, name, as_name, start) };
            if found == LOUDER_ACTOR_NONE {
                return out;
            }
            out.push(found);
            start = found;
        }
    }

    /// Every speaker an actor speaks as.
    pub fn speakers_of(&mut self, conversation: &str, actor: i32) -> Vec<String> {
        let name = self.declare(conversation);
        let mut out = Vec::new();
        let mut start = LOUDER_NAME_NONE;
        loop {
            let found = unsafe { ffi::next_actor_speak_as(self.ctx, name, actor, start) };
            if found == LOUDER_NAME_NONE {
                return out;
            }
            out.push(self.text(found));
            start = found;
        }
    }

    /// Every tag on a hub.
    pub fn tags_of(&mut self, conversation: &str, position: i32) -> Vec<String> {
        let name = self.declare(conversation);
        let mut out = Vec::new();
        let mut start = LOUDER_NAME_NONE;
        loop {
            let found = unsafe { ffi::next_hub_tag(self.ctx, name, position, start) };
            if found == LOUDER_NAME_NONE {
                return out;
            }
            out.push(self.text(found));
            start = found;
        }
    }

    /// Every speaker listening to another.
    pub fn speakers_listening_to(&mut self, speaker: &str) -> Vec<String> {
        let name = self.declare(speaker);
        let mut out = Vec::new();
        let mut start = LOUDER_NAME_NONE;
        loop {
            let found = unsafe { ffi::get_next_speaker_listen_to(self.ctx, name, start) };
            if found == LOUDER_NAME_NONE {
                return out;
            }
            out.push(self.text(found));
            start = found;
        }
    }

    /// The dialogue rules in effect.
    pub fn rules(&mut self) -> DialogueRules {
        unsafe { ffi::get_dialogue_rules(self.ctx) }
    }

    /// Replace the dialogue rules. They take effect on the next move.
    pub fn set_rules(&mut self, rules: DialogueRules) -> bool {
        unsafe { ffi::set_dialogue_rules(self.ctx, rules) == 0 }
    }

    /// Reach an entry point the safe layer did not wrap.
    ///
    /// Every function in the header is declared in `ffi`; this is a way to call
    /// one from here without the two layers having to agree on how.
    pub unsafe fn call_linkage(&mut self) -> i32 {
        ffi::test_linkage()
    }

    fn event(&mut self, raw: LouderEvent) -> Event {
        let value = match raw.data.arm() {
            Some(EventDataArm::regular_motion(m))
            | Some(EventDataArm::warp(m))
            | Some(EventDataArm::force_motion(m)) => Some(EventValue::Motion {
                source_position: m.source_position,
                target_position: m.target_position,
                choice_index: m.choice_index,
                background: m.background,
            }),
            Some(EventDataArm::choice_appearance(c)) => Some(EventValue::ChoiceAppearance {
                position: c.position,
                choice_index: c.choice_index,
            }),
            Some(EventDataArm::speaker_change(value))
            | Some(EventDataArm::text_change(value))
            | Some(EventDataArm::statement(value))
            | Some(EventDataArm::callback(value))
            | Some(EventDataArm::speaker_added(value))
            | Some(EventDataArm::speaker_removed(value))
            | Some(EventDataArm::conversation_added(value))
            | Some(EventDataArm::conversation_removed(value)) => Some(EventValue::Name(self.text(value))),
            _ => Some(EventValue::Other),
        };
        Event {
            conversation: self.text(raw.conversation),
            channel: self.text(raw.channel),
            actor: raw.actor,
            kind: raw.data.kind().unwrap_or("Unknown"),
            value,
        }
    }
}

impl Drop for Louder {
    fn drop(&mut self) {
        self.close();
    }
}

/// Walks the event queue from the outside.
struct EventIter<'a> {
    engine: &'a mut Louder,
    index: usize,
}

impl Iterator for EventIter<'_> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        let found = unsafe { ffi::get_event(self.engine.ctx, self.index) };
        match found.tag {
            LouderOption_LouderEvent_Tag::Some_LouderEvent => {
                self.index += 1;
                Some(self.engine.event(unsafe { found.body.some }))
            }
            _ => None,
        }
    }
}
