#[cfg(feature = "godot-bindings")]
use godot::classes::class_macros::private::virtuals::ZipReader::StringName;

#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, Gd, GString, Variant};
#[cfg(feature = "godot-bindings")]
use super::json;
#[cfg(feature = "godot-bindings")]
use super::json_loader;
#[cfg(feature = "godot-bindings")]
use super::query;
#[cfg(feature = "godot-bindings")]
use super::simulator::{MotionKind, Simulator};
use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};

use super::name::*;
use super::speaker::*;
use super::actor::{Actor, LouderActor, LOUDER_ACTOR_NONE};
#[cfg(feature = "godot-bindings")]
use super::actor::ActorFlags;
use super::conversation::{Conversation, VariableMap, VariableScope, classify_variable};
use super::event::*;
use super::hub::{Hub, LouderPosition};
#[cfg(feature = "godot-bindings")]
use super::hub::{HubFlags, BLANK_POSITION};
use super::object::LouderVariant;
use super::rules::DialogueRules;

/// One queued ask, together with the verdict the host gave it.
///
/// An ask stays queued after being answered so that the host can keep walking
/// the cache, but it stops accepting answers once `verdict` is set.
struct PendingAsk {
  ask: LouderAsk,
  verdict: Option<i32>,
}

/// Where `content` sits in the name table: the id it already has, or the id it
/// would take if it were added now.
///
/// Ids are the hash of the text, probed forward on collision. Both halves of
/// looking a name up come from here, so that the id an intern lands on and the
/// id a later lookup finds cannot disagree.
fn probe_name(names: &LouderNameMap, content: &LouderNameContent) -> LouderContextResult {
  let mut hasher = DefaultHasher::new();
  content.bytes.hash(&mut hasher);

  let mut name = hasher.finish() as LouderName;
  let mut step = 1;
  while let Some(existing) = names.get(&name) {
    if existing == content {
      return LouderContextResult::Found(name);
    }
    name = name.wrapping_add(step);
    step += 1;
  }

  LouderContextResult::NotFound(name)
}

/// Assign an id to `text` and record it in `names`.
///
/// Free-standing so that a context can be built with its ids already in place.
/// Interning only ever needs the name table, and giving it its own entry point
/// keeps that visible, instead of forcing the rest of a context to exist first
/// just so one field can be filled in after the fact.
fn intern_into(names: &mut LouderNameMap, text: &str) -> LouderName {
  let content = LouderNameContent::new(text);
  match probe_name(names, &content) {
    LouderContextResult::Found(name) => name,
    LouderContextResult::NotFound(name) => {
      names.insert(name, content);
      name
    }
  }
}

/// The opaque handle to the engine.
///
/// Obtained from [`crate::init_louder`] and released with
/// [`crate::deinit_louder`]. The contents are internal to the engine and not
/// part of the public API.
///
/// Story state is held here rather than on [`Conversation`] so that the shared
/// buckets survive loading and unloading individual conversations.
///
/// Under the Godot bindings this is a class in its own right, and the engine's
/// whole vocabulary is reached through it: names, conversations, actors, hubs,
/// speakers, story state, the event queue, and the ask cache. GDScript builds
/// one with `LouderContext.new()`.
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(tool))]
pub struct LouderContext {
  names: LouderNameMap,
  main_channel: LouderName,
  rules: DialogueRules,
  speakers: Map<LouderName, Speaker>,
  conversations: Map<LouderName, Conversation>,
  actors: Map<LouderName, Vec<Actor>>,
  next_actor_id: Map<LouderName, LouderActor>,
  shared_saved: VariableMap,
  shared_unsaved: VariableMap,
  events: Vec<LouderEvent>,
  asks: VecDeque<PendingAsk>,
}

godot_api! {
  impl ::godot::classes::IRefCounted for LouderContext {
    fn init(_base: ::godot::obj::Base<::godot::classes::RefCounted>) -> Self {
      // The default channel is interned eagerly by `new`, which a generated
      // `init` could not do, since it fills each field with its `Default` and
      // has no way to intern one of them.
      Self::new()
    }
  }
}

impl LouderContext {
  pub fn new() -> LouderContext {
    let mut names = LouderNameMap::new();
    LouderContext {
      // Interned eagerly so that "main" always has an id, even in a
      // conversation that never mentions it.
      main_channel: intern_into(&mut names, LOUDER_MAIN_CHANNEL),
      names,
      // Every rule on, since each one is a default the guide describes rather
      // than an opt-in a conversation has to ask for.
      rules: DialogueRules::all(),
      speakers: Map::new(),
      conversations: Map::new(),
      actors: Map::new(),
      next_actor_id: Map::new(),
      shared_saved: Map::new(),
      shared_unsaved: Map::new(),
      events: Vec::new(),
      asks: VecDeque::new(),
    }
  }

  /// The id of the default channel, which every conversation starts on.
  pub fn main_channel(&self) -> LouderName {
    self.main_channel
  }

  /// The dialogue rules currently in effect.
  pub fn rules(&self) -> DialogueRules {
    self.rules
  }

  /// Replace the dialogue rules.
  pub fn set_rules(&mut self, rules: DialogueRules) {
    self.rules = rules;
  }

  pub fn push_event(&mut self, event: LouderEvent) -> usize {
    self.events.push(event);
    self.events.len() - 1
  }

  /// The queued event at `index`, if there is one.
  pub fn event(&self, index: usize) -> Option<&LouderEvent> {
    self.events.get(index)
  }

  /// How many events are queued.
  pub fn event_count(&self) -> usize {
    self.events.len()
  }

  pub fn clear_events(&mut self) {
    self.events.clear();
  }

  pub fn push_ask(&mut self, ask: LouderAsk) -> usize {
    self.asks.push_back(PendingAsk { ask, verdict: None });
    self.asks.len() - 1
  }

  /// The ask at `index`, or `None` if the index is out of range.
  pub fn ask(&self, index: usize) -> Option<&LouderAsk> {
    self.asks.get(index).map(|entry| &entry.ask)
  }

  /// How many asks are cached, answered or not.
  pub fn ask_count(&self) -> usize {
    self.asks.len()
  }

  /// Record `verdict` for the ask at `index` and move it to the back of the
  /// queue.
  ///
  /// Returns false when `index` is out of range or that ask was already
  /// answered. Answered asks remain readable so the host can keep polling the
  /// whole cache, but they stop accepting further answers.
  pub fn answer_ask(&mut self, index: usize, verdict: i32) -> bool {
    let Some(entry) = self.asks.get_mut(index) else {
      return false;
    };
    if entry.verdict.is_some() {
      return false;
    }
    entry.verdict = Some(verdict);

    // Answering moves the ask to the back of the queue, so that a host polling
    // the whole cache sees unanswered asks first.
    let answered = self.asks.remove(index).expect("the index was just found");
    self.asks.push_back(answered);
    true
  }

  // -- conversations ------------------------------------------------------

  pub fn get_conversation(&self, name: LouderName) -> Option<&Conversation> {
    self.conversations.get(&name)
  }

  pub fn get_conversation_mut(&mut self, name: LouderName) -> Option<&mut Conversation> {
    self.conversations.get_mut(&name)
  }

  /// The hub at `position` in a conversation, if both the conversation and the
  /// hub are there.
  ///
  /// Nearly every question a host asks is about one hub of one conversation, so
  /// the two lookups are made once here rather than spelled out at each of them.
  pub fn hub(&self, conversation: LouderName, position: LouderPosition) -> Option<&Hub> {
    self.get_conversation(conversation)?.hub(position)
  }

  /// Insert a conversation, reporting whether the name was already taken.
  pub fn add_conversation(&mut self, conversation: Conversation) -> bool {
    let name = conversation.name();
    self.conversations.insert(name, conversation).is_some()
  }

  /// Remove a conversation along with its actors, and announce its removal.
  ///
  /// Shared story state is left alone, since it outlives any one conversation.
  pub fn remove_conversation(&mut self, name: LouderName) -> bool {
    self.actors.remove(&name);
    self.next_actor_id.remove(&name);

    if self.conversations.remove(&name).is_none() {
      return false;
    }

    self.push_event(LouderEvent {
      conversation: name,
      channel: LOUDER_NAME_NONE,
      actor: LOUDER_ACTOR_NONE,
      data: EventData::ConversationRemoved(name),
    });

    true
  }

  // -- actors -------------------------------------------------------------

  /// The actors of a conversation, in ascending id order.
  pub fn actors(&self, conversation: LouderName) -> &[Actor] {
    match self.actors.get(&conversation) {
      Some(actors) => actors,
      None => &[],
    }
  }

  pub fn get_actor(&self, conversation: LouderName, actor: LouderActor) -> Option<&Actor> {
    let actors = self.actors.get(&conversation)?;
    actors.iter().find(|a| a.id() == actor)
  }

  pub fn get_actor_mut(&mut self, conversation: LouderName, actor: LouderActor) -> Option<&mut Actor> {
    let actors = self.actors.get_mut(&conversation)?;
    actors.iter_mut().find(|a| a.id() == actor)
  }

  /// Append an actor, giving it the next unused id for its conversation, and
  /// announce it.
  ///
  /// Ids are never reused, so ascending id order is a stable ordering even
  /// after actors are removed. The event is emitted here rather than by the
  /// caller so that an actor cannot enter a conversation unannounced.
  pub fn add_actor(&mut self, conversation: LouderName, mut actor: Actor) -> LouderActor {
    let next = self.next_actor_id.entry(conversation).or_insert(0);
    let id = *next;
    *next += 1;
    actor.set_id(id);

    let channel = actor.channel();
    let actors = self.actors.entry(conversation).or_default();
    actors.push(actor);

    self.push_event(LouderEvent {
      conversation,
      channel,
      actor: id,
      data: EventData::ActorAdded,
    });

    id
  }

  /// Remove an actor, reporting whether it was there, and announce its removal.
  pub fn remove_actor(&mut self, conversation: LouderName, actor: LouderActor) -> bool {
    let removed = match self.actors.get_mut(&conversation) {
      Some(actors) => {
        let before = actors.len();
        actors.retain(|a| a.id() != actor);
        actors.len() != before
      }
      None => false,
    };

    if removed {
      self.push_event(LouderEvent {
        conversation,
        channel: LOUDER_NAME_NONE,
        actor,
        data: EventData::ActorRemoved,
      });
    }

    removed
  }

  // -- speakers -----------------------------------------------------------

  /// Remove every speaker in the config.
  pub fn clear_config(&mut self) {
    self.speakers.clear();
  }

  /// Insert a speaker, reporting whether the name was already taken.
  pub fn add_speaker(&mut self, speaker: Speaker) -> bool {
    let name = speaker.name();
    self.speakers.insert(name, speaker).is_some()
  }

  /// Insert a speaker, replacing any existing one with the same name.
  pub fn set_speaker(&mut self, speaker: Speaker) {
    let name = speaker.name();
    self.speakers.insert(name, speaker);
  }

  /// Every speaker name in ascending order.
  pub fn speaker_names(&self) -> Vec<LouderName> {
    let mut names: Vec<LouderName> = self.speakers.keys().copied().collect();
    names.sort_unstable();
    names
  }

  // -- story state --------------------------------------------------------

  fn variables_mut(&mut self, conversation: LouderName, scope: VariableScope) -> Option<&mut VariableMap> {
    match scope {
      VariableScope::LocalSaved | VariableScope::LocalUnsaved => {
        Some(self.conversations.get_mut(&conversation)?.variables_mut(scope))
      }
      VariableScope::SharedSaved => Some(&mut self.shared_saved),
      VariableScope::SharedUnsaved => Some(&mut self.shared_unsaved),
    }
  }

  fn variables(&self, conversation: LouderName, scope: VariableScope) -> Option<&VariableMap> {
    match scope {
      VariableScope::LocalSaved | VariableScope::LocalUnsaved => {
        self.conversations.get(&conversation).map(|c| c.variables(scope))
      }
      VariableScope::SharedSaved => Some(&self.shared_saved),
      VariableScope::SharedUnsaved => Some(&self.shared_unsaved),
    }
  }

  /// The scope a variable name falls into, read from its own text.
  pub fn scope_of(&self, key: LouderName) -> VariableScope {
    match self.resolve_name(key) {
      Some(text) => classify_variable(text),
      None => VariableScope::LocalSaved,
    }
  }

  /// Read a story state variable. Uninitialized variables read as 0.
  pub fn get_state(&self, conversation: LouderName, key: LouderName) -> LouderVariant {
    self
      .variables(conversation, self.scope_of(key))
      .and_then(|variables| variables.get(&key).cloned())
      .unwrap_or_default()
  }

  /// Write a story state variable, storing it in the bucket its name implies.
  ///
  /// Returns false when the variable is local and its conversation is not
  /// loaded, since there would be nowhere to put it. A shared variable always
  /// has somewhere to go.
  pub fn set_state(&mut self, conversation: LouderName, key: LouderName, value: LouderVariant) -> bool {
    match self.variables_mut(conversation, self.scope_of(key)) {
      Some(variables) => {
        variables.insert(key, value);
        true
      }
      None => false,
    }
  }

  /// Whether a story state variable has been written.
  pub fn has_state(&self, conversation: LouderName, key: LouderName) -> bool {
    self
      .variables(conversation, self.scope_of(key))
      .is_some_and(|variables| variables.contains_key(&key))
  }

  /// Every key written in the conversation, in ascending id order.
  ///
  /// Covers both local buckets.
  pub fn conversation_state_keys(&self, conversation: LouderName) -> Vec<LouderName> {
    let Some(conversation) = self.conversations.get(&conversation) else {
      return Vec::new();
    };

    let mut keys: Vec<LouderName> = conversation
      .variables(VariableScope::LocalSaved)
      .keys()
      .chain(conversation.variables(VariableScope::LocalUnsaved).keys())
      .copied()
      .collect();
    keys.sort_unstable();
    keys.dedup();
    keys
  }

  /// Every key written in either shared bucket, in ascending id order.
  pub fn shared_state_keys(&self) -> Vec<LouderName> {
    let mut keys: Vec<LouderName> = self.shared_saved.keys().chain(self.shared_unsaved.keys()).copied().collect();
    keys.sort_unstable();
    keys.dedup();
    keys
  }

  /// Every key written anywhere, local or shared, in ascending id order.
  ///
  /// This is what a save of the story is assembled from, so it lives here rather
  /// than being spelled out again at each place that saves.
  pub fn all_state_keys(&self, conversation: LouderName) -> Vec<LouderName> {
    let mut keys = self.conversation_state_keys(conversation);
    keys.extend(self.shared_state_keys());
    keys.sort_unstable();
    keys.dedup();
    keys
  }
}

impl Default for LouderContext {
  fn default() -> Self {
    return LouderContext::new()
  }
}

impl LouderNameContext for LouderContext {
  fn find_name(&self, name_context: &LouderNameContent) -> LouderContextResult {
    probe_name(&self.names, name_context)
  }

  fn has_name(&self, name: &LouderName) -> bool {
    self.names.contains_key(name)
  }

  fn get_name_content(&self, name: &LouderName) -> Option<&LouderNameContent> {
    self.names.get(name)
  }

  fn add_name(&mut self, name_context: &LouderNameContent) -> LouderContextResult {
    match self.find_name(name_context) {
      LouderContextResult::Found(name) => LouderContextResult::Found(name),
      LouderContextResult::NotFound(name) => {
        self.names.insert(name, name_context.clone());
        LouderContextResult::NotFound(name)
      }
    }
  }
}

impl SpeakerMap for LouderContext {
  fn get_speaker(&self, speaker_name: LouderName) -> Option<&Speaker> {
    self.speakers.get(&speaker_name)
  }
}

godot_api! {
  /// The engine's own vocabulary, as a class.
  ///
  /// One instance is one running dialogue: its name table, its conversations,
  /// its actors, its speaker config, its story state, its event queue, and its
  /// ask cache. Everything a host can ask about or ask for goes through here.
  ///
  /// This is the same surface the `extern "C"` entry points in the crate root
  /// describe, with the boundary's plumbing removed. In particular, strings are
  /// `String`s rather than pointers into a caller's buffer, and an object the
  /// engine holds is handed over as a copy: writing to that copy does not move
  /// the engine's own, so anything that changes a conversation goes through a
  /// method on this class.
  impl LouderContext {
    // -- conversations ---------------------------------------------------

    /// Whether a conversation is loaded under this name.
    #[func]
    pub fn has_conversation(&self, conversation: LouderName) -> bool {
      self.get_conversation(conversation).is_some()
    }

    /// A loaded conversation, as a copy of what the engine holds.
    #[func(rename = "get_conversation")]
    pub fn gd_get_conversation(&self, conversation: LouderName) -> Option<Gd<Conversation>> {
      self.get_conversation(conversation).map(|entry| Gd::from_object(entry.clone()))
    }

    /// Remove a conversation and all associated actors.
    ///
    /// Shared story state is kept, since it outlives any one conversation.
    /// Returns false if the conversation was not loaded.
    #[func(rename = "remove_conversation")]
    pub fn gd_remove_conversation(&mut self, conversation: LouderName) -> bool {
      LouderContext::remove_conversation(self, conversation)
    }

    // -- the event queue -------------------------------------------------

    /// The queued event at `index`, if there is one.
    ///
    /// Poll from index 0 upwards until this comes back empty, then call
    /// [`LouderContext::gd_flush_events`].
    #[func]
    pub fn get_event(&self, index: i32) -> Option<Gd<LouderEvent>> {
      if index < 0 {
        return None;
      }
      self.event(index as usize).map(|event| Gd::from_object(event.clone()))
    }

    /// How many events are queued.
    #[func]
    pub fn get_event_count(&self) -> i32 {
      self.event_count() as i32
    }

    /// Remove every queued event.
    #[func(rename = "flush_events")]
    pub fn gd_flush_events(&mut self) {
      self.clear_events();
    }

    /// Push an event onto the queue, used for testing.
    ///
    /// Nothing about the event is interpreted here, so the caller is filling it
    /// in themselves.
    #[func(rename = "push_event")]
    pub fn gd_push_event(&mut self, event: Gd<LouderEvent>) {
      self.push_event(event.bind().clone());
    }

    // -- the ask cache ---------------------------------------------------

    /// The ask at `index`, if there is one.
    ///
    /// Asks are not in a stable order, so read them until one comes back empty
    /// rather than assuming the cache only grows.
    #[func]
    pub fn get_ask(&self, index: i32) -> Option<Gd<LouderAsk>> {
      if index < 0 {
        return None;
      }
      self.ask(index as usize).map(|ask| Gd::from_object(ask.clone()))
    }

    /// Push an ask onto the cache, used for testing.
    #[func(rename = "push_ask")]
    pub fn gd_push_ask(&mut self, ask: Gd<LouderAsk>) {
      self.push_ask(ask.bind().clone());
    }

    /// Answer the ask at `index` with `verdict`.
    ///
    /// Answering marks the ask complete and moves it to the back of the cache,
    /// so indexes shift: read the cache again after each answer. Returns false
    /// if `index` was out of range or that ask had already been answered.
    #[func(rename = "answer_ask")]
    pub fn gd_answer_ask(&mut self, index: i32, verdict: i32) -> bool {
      if index < 0 {
        return false;
      }
      LouderContext::answer_ask(self, index as usize, verdict)
    }

    // -- speakers --------------------------------------------------------

    /// A speaker from the config, as a copy of what the engine holds.
    ///
    /// Empty for a speaker that was never declared, which is treated as public.
    #[func(rename = "get_speaker")]
    pub fn gd_get_speaker(&self, speaker: LouderName) -> Option<Gd<Speaker>> {
      SpeakerMap::get_speaker(self, speaker).map(|entry| Gd::from_object(entry.clone()))
    }

    /// Every speaker name, in ascending order.
    #[func(rename = "speaker_names")]
    pub fn gd_speaker_names(&self) -> Array<i64> {
      LouderContext::speaker_names(self)
        .into_iter()
        .map(|name| name as i64)
        .collect()
    }

    /// The flag bits of a speaker, or 0 if it is not declared.
    #[func]
    pub fn get_speaker_flags(&self, speaker: LouderName) -> i32 {
      match SpeakerMap::get_speaker(self, speaker) {
        Some(entry) => entry.flags().bits(),
        None => SpeakerFlags::empty().bits(),
      }
    }

    /// Whether hubs with this speaker are visible to actors who are not
    /// listening to it. An undefined speaker is public.
    #[func]
    pub fn is_speaker_public(&self, speaker: LouderName) -> bool {
      SpeakerMap::is_speaker_public(self, speaker)
    }

    /// A speaker's additional data under `key`, read as an integer, or 0.
    #[func]
    pub fn get_speaker_additional_int(&self, speaker: LouderName, key: LouderName) -> i32 {
      match SpeakerMap::get_speaker(self, speaker).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_int(),
        None => 0,
      }
    }

    /// A speaker's additional data under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_speaker_additional_float(&self, speaker: LouderName, key: LouderName) -> f64 {
      match SpeakerMap::get_speaker(self, speaker).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_float() as f64,
        None => 0.0,
      }
    }

    /// A speaker's additional data under `key` as a name, or
    /// [`LOUDER_NAME_NONE`].
    #[func]
    pub fn get_speaker_additional_name(&self, speaker: LouderName, key: LouderName) -> LouderName {
      match SpeakerMap::get_speaker(self, speaker).and_then(|entry| entry.additional_get(key)) {
        Some(LouderVariant::String(name)) => *name,
        _ => LOUDER_NAME_NONE,
      }
    }

    /// The speakers `speaker` is listening to, after `start`.
    ///
    /// Pass [`LOUDER_NAME_NONE`] as `start` to begin.
    #[func]
    pub fn get_next_speaker_listen_to(&self, speaker: LouderName, start: LouderName) -> LouderName {
      query::next_speaker_listen_to(self, speaker, start)
    }

    /// Remove all speakers in the config.
    ///
    /// Conversations and actors are untouched, so hubs naming a now-missing
    /// speaker become public again.
    #[func(rename = "clear_config")]
    pub fn gd_clear_config(&mut self) {
      LouderContext::clear_config(self);
    }

    // -- actors ----------------------------------------------------------

    /// An actor of a conversation, as a copy of what the engine holds.
    #[func(rename = "get_actor")]
    pub fn gd_get_actor(&self, conversation: LouderName, actor: LouderActor) -> Option<Gd<Actor>> {
      self.get_actor(conversation, actor).map(|entry| Gd::from_object(entry.clone()))
    }

    /// Every actor id of a conversation, in ascending order.
    #[func]
    pub fn get_actor_ids(&self, conversation: LouderName) -> Array<i64> {
      self.actors(conversation)
        .iter()
        .map(|entry| entry.id() as i64)
        .collect()
    }

    /// Add an actor to a conversation and return the id given to it.
    ///
    /// The actor starts at `start_position` on the default channel, with
    /// priority 0. Ids are per conversation and never reused, so ascending
    /// order is a stable iteration order. Returns [`LOUDER_ACTOR_NONE`] if the
    /// conversation is not loaded.
    #[func(rename = "add_actor")]
    pub fn gd_add_actor(
      &mut self,
      conversation: LouderName,
      start_position: LouderPosition,
      flags: i32,
      speak_as: Array<i64>,
      listen_to: Array<i64>,
    ) -> LouderActor {
      if self.get_conversation(conversation).is_none() {
        return LOUDER_ACTOR_NONE;
      }

      let actor = Actor::new(
        start_position,
        ActorFlags::from_bits_truncate(flags),
        speak_as.iter_shared().map(|name| name as LouderName).collect(),
        listen_to.iter_shared().map(|name| name as LouderName).collect(),
        self.main_channel(),
      );

      self.add_actor(conversation, actor)
    }

    /// Remove an actor from a conversation, reporting whether it was there.
    #[func(rename = "remove_actor")]
    pub fn gd_remove_actor(&mut self, conversation: LouderName, actor: LouderActor) -> bool {
      LouderContext::remove_actor(self, conversation, actor)
    }

    /// The channel of an actor, or [`LOUDER_NAME_NONE`].
    ///
    /// Actors with [`ActorFlags::SPECTATOR`] or [`ActorFlags::DEBUG`] are not
    /// part of the flow and read as blank.
    #[func]
    pub fn get_actor_channel(&self, conversation: LouderName, actor: LouderActor) -> LouderName {
      match self.get_actor(conversation, actor) {
        Some(entry) => entry.channel(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The position of an actor, or `BLANK_POSITION` if it does not exist.
    #[func]
    pub fn get_actor_position(&self, conversation: LouderName, actor: LouderActor) -> LouderPosition {
      match self.get_actor(conversation, actor) {
        Some(entry) => entry.position(),
        None => BLANK_POSITION,
      }
    }

    /// The flag bits of an actor, or 0 if it does not exist.
    #[func]
    pub fn get_actor_flags(&self, conversation: LouderName, actor: LouderActor) -> i32 {
      match self.get_actor(conversation, actor) {
        Some(entry) => entry.flags().bits(),
        None => ActorFlags::empty().bits(),
      }
    }

    /// The priority of an actor. Actors outside the flow read as 0.
    #[func]
    pub fn get_actor_priority(&self, conversation: LouderName, actor: LouderActor) -> i32 {
      match self.get_actor(conversation, actor) {
        Some(entry) => entry.priority(),
        None => 0,
      }
    }

    /// An actor's additional data under `key`, read as an integer, or 0.
    #[func]
    pub fn get_actor_additional_int(&self, conversation: LouderName, actor: LouderActor, key: LouderName) -> i32 {
      match self.get_actor(conversation, actor).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_int(),
        None => 0,
      }
    }

    /// An actor's additional data under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_actor_additional_float(&self, conversation: LouderName, actor: LouderActor, key: LouderName) -> f64 {
      match self.get_actor(conversation, actor).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_float() as f64,
        None => 0.0,
      }
    }

    /// An actor's additional data under `key` as a name, or [`LOUDER_NAME_NONE`].
    #[func]
    pub fn get_actor_additional_name(&self, conversation: LouderName, actor: LouderActor, key: LouderName) -> LouderName {
      match self.get_actor(conversation, actor).and_then(|entry| entry.additional_get(key)) {
        Some(LouderVariant::String(name)) => *name,
        _ => LOUDER_NAME_NONE,
      }
    }

    /// The speakers an actor speaks as, after `start`.
    #[func]
    pub fn next_actor_speak_as(&self, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
      query::next_actor_speak_as(self, conversation, actor, start)
    }

    /// The base speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening_base(&self, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
      query::next_actor_listening_base(self, conversation, actor, start)
    }

    /// The speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening(&self, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
      query::next_actor_listening(self, conversation, actor, start)
    }

    /// The actors in a channel, after `start`.
    #[func]
    pub fn next_actor_in_channel(&self, conversation: LouderName, channel: LouderName, start: LouderActor) -> LouderActor {
      query::next_actor_in_channel(self, conversation, channel, start)
    }

    /// The actors in a channel at one position, after `start`.
    #[func]
    pub fn next_actor_in_channel_and_position(&self, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
      query::next_actor_in_channel_and_position(self, conversation, channel, position, start)
    }

    /// The actors speaking as a speaker, after `start`.
    #[func]
    pub fn next_actor_speaking_as(&self, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
      query::next_actor_speaking_as(self, conversation, speaker, start)
    }

    /// The actors listening to a speaker, after `start`.
    #[func]
    pub fn next_actor_listening_to(&self, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
      query::next_actor_listening_to(self, conversation, speaker, start)
    }

    /// The actors carrying certain flags, after `start`.
    #[func]
    pub fn next_actor_from_flags(&self, conversation: LouderName, include: i32, exact: bool, start: LouderActor) -> LouderActor {
      query::next_actor_from_flags(self, conversation, ActorFlags::from_bits_truncate(include), i32::from(exact), start)
    }

    // -- hubs ------------------------------------------------------------

    /// A hub, as a copy of what the engine holds.
    #[func(rename = "get_hub")]
    pub fn gd_get_hub(&self, conversation: LouderName, position: LouderPosition) -> Option<Gd<Hub>> {
      self.hub(conversation, position).map(|hub| Gd::from_object(hub.clone()))
    }

    /// The speaker of a hub, or [`LOUDER_NAME_NONE`].
    ///
    /// A hub with no speaker is visible to everyone, as is one that does not
    /// exist.
    #[func]
    pub fn get_hub_speaker(&self, conversation: LouderName, position: LouderPosition) -> LouderName {
      match self.hub(conversation, position) {
        Some(hub) => hub.speaker(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The text of a hub as a name, or [`LOUDER_NAME_NONE`].
    #[func]
    pub fn get_hub_text(&self, conversation: LouderName, position: LouderPosition) -> LouderName {
      match self.hub(conversation, position) {
        Some(hub) => hub.text_name(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The number of choices on a hub from an actor's perspective.
    ///
    /// Only choices the actor could actually take are counted, so this applies
    /// the visibility, recipient, and spoke rules. Returns 0 for an unknown hub
    /// or actor.
    #[func]
    pub fn get_hub_choice_count(&self, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor) -> i32 {
      Simulator::available_choices(self, conversation, actor_perspective, position).len() as i32
    }

    /// The indices of the choices on a hub that `actor` may take.
    #[func]
    pub fn get_hub_choices(&self, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor) -> Array<i64> {
      Simulator::available_choices(self, conversation, actor_perspective, position)
        .into_iter()
        .map(|index| index as i64)
        .collect()
    }

    /// The text of a choice on a hub as a name, or [`LOUDER_NAME_NONE`].
    ///
    /// A choice has no text of its own, so this is the text of the hub it links
    /// to, and it is blank unless the choice is available to `actor`.
    #[func]
    pub fn get_hub_choice_text(&self, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> LouderName {
      match query::available_choice(self, conversation, position, actor_perspective, choice_index.max(0) as usize) {
        Some(choice) => choice.text_name(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The position a choice on a hub links to, or `BLANK_POSITION`.
    #[func]
    pub fn get_hub_choice_link(&self, conversation: LouderName, position: LouderPosition, actor_perspective: LouderActor, choice_index: i32) -> LouderPosition {
      match query::available_choice(self, conversation, position, actor_perspective, choice_index.max(0) as usize) {
        Some(choice) => choice.position(),
        None => BLANK_POSITION,
      }
    }

    /// The expressions on a hub, one name per entry.
    #[func]
    pub fn get_hub_expressions(&self, conversation: LouderName, position: LouderPosition) -> Array<i64> {
      match self.hub(conversation, position) {
        Some(hub) => hub.expressions().iter().map(|name| *name as i64).collect(),
        None => Array::new(),
      }
    }

    /// The whole expression block of a hub as one name, newline separated.
    #[func]
    pub fn get_hub_expressions_joined(&self, conversation: LouderName, position: LouderPosition) -> LouderName {
      match self.hub(conversation, position) {
        Some(hub) => hub.expressions_joined(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The callbacks on a hub, one name per entry.
    #[func]
    pub fn get_hub_callbacks(&self, conversation: LouderName, position: LouderPosition) -> Array<i64> {
      match self.hub(conversation, position) {
        Some(hub) => hub.callbacks().iter().map(|name| *name as i64).collect(),
        None => Array::new(),
      }
    }

    /// The whole callback block of a hub as one name, newline separated.
    #[func]
    pub fn get_hub_callbacks_joined(&self, conversation: LouderName, position: LouderPosition) -> LouderName {
      match self.hub(conversation, position) {
        Some(hub) => hub.callbacks_joined(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The flag bits of a hub, or 0 if it does not exist.
    #[func]
    pub fn get_hub_flags(&self, conversation: LouderName, position: LouderPosition) -> i32 {
      match self.hub(conversation, position) {
        Some(hub) => hub.flags().bits(),
        None => HubFlags::empty().bits(),
      }
    }

    /// The recipients of a hub, after `start`.
    ///
    /// This comes back blank straight away for a hub with no recipients, which
    /// is how a hub that is not restricted reads.
    #[func]
    pub fn next_hub_recipient(&self, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
      query::next_hub_recipient(self, conversation, position, start)
    }

    /// The tags of a hub, after `start`.
    #[func]
    pub fn next_hub_tag(&self, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
      query::next_hub_tag(self, conversation, position, start)
    }

    /// The channel a hub moves an actor into, or [`LOUDER_NAME_NONE`].
    #[func]
    pub fn get_hub_channel(&self, conversation: LouderName, position: LouderPosition) -> LouderName {
      match self.hub(conversation, position) {
        Some(hub) => hub.channel(),
        None => LOUDER_NAME_NONE,
      }
    }

    /// The priority of a hub, or 0 if it does not exist.
    #[func]
    pub fn get_hub_priority(&self, conversation: LouderName, position: LouderPosition) -> i32 {
      match self.hub(conversation, position) {
        Some(hub) => hub.priority(),
        None => 0,
      }
    }

    /// A hub's additional data under `key`, read as an integer, or 0.
    #[func]
    pub fn get_hub_additional_int(&self, conversation: LouderName, position: LouderPosition, key: LouderName) -> i32 {
      match self.hub(conversation, position).and_then(|hub| hub.additional_get(key)) {
        Some(value) => value.as_int(),
        None => 0,
      }
    }

    /// A hub's additional data under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_hub_additional_float(&self, conversation: LouderName, position: LouderPosition, key: LouderName) -> f64 {
      match self.hub(conversation, position).and_then(|hub| hub.additional_get(key)) {
        Some(value) => value.as_float() as f64,
        None => 0.0,
      }
    }

    /// A hub's additional data under `key` as a name, or [`LOUDER_NAME_NONE`].
    #[func]
    pub fn get_hub_additional_name(&self, conversation: LouderName, position: LouderPosition, key: LouderName) -> LouderName {
      match self.hub(conversation, position).and_then(|hub| hub.additional_get(key)) {
        Some(LouderVariant::String(name)) => *name,
        _ => LOUDER_NAME_NONE,
      }
    }

    /// The hubs in a channel, after `start`.
    #[func]
    pub fn next_hub_by_channel(&self, conversation: LouderName, channel: LouderName, start: LouderPosition) -> LouderPosition {
      query::next_hub_by_channel(self, conversation, channel, start)
    }

    /// The hubs spoken by a speaker, after `start`.
    #[func]
    pub fn next_hub_spoken_by(&self, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
      query::next_hub_spoken_by(self, conversation, speaker, start)
    }

    /// The hubs heard by a speaker, after `start`.
    #[func]
    pub fn next_hub_heard_by(&self, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
      query::next_hub_heard_by(self, conversation, speaker, start)
    }

    /// The hubs with a priority within the inclusive bounds, after `start`.
    #[func]
    pub fn next_hub_by_priority(&self, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
      query::next_hub_by_priority(self, conversation, priority_min, priority_max, start)
    }

    /// The hubs carrying all, or exactly, the given tags, after `start`.
    #[func]
    pub fn next_hub_from_tags(&self, conversation: LouderName, include: Array<i64>, exact: bool, start: LouderPosition) -> LouderPosition {
      let wanted: Vec<LouderName> = include.iter_shared().map(|tag| tag as LouderName).collect();
      query::next_hub_from_tags(self, conversation, &wanted, exact, start)
    }

    /// The hubs carrying certain flags, after `start`.
    #[func]
    pub fn next_hub_from_flags(&self, conversation: LouderName, include: i32, exact: bool, start: LouderPosition) -> LouderPosition {
      query::next_hub_from_flags(self, conversation, HubFlags::from_bits_truncate(include), i32::from(exact), start)
    }

    // -- story state -----------------------------------------------------

    /// A story state value.
    ///
    /// The variable's own name decides where it is stored: a leading underscore
    /// means it is not saved, and an uppercase first letter means it is shared
    /// across conversations. Variables that have never been written read as 0,
    /// as does anything read from a conversation that is not loaded.
    #[func]
    pub fn get_story_state(&self, conversation: LouderName, state: LouderName) -> Variant {
      self.get_state(conversation, state).to_godot(self)
    }

    /// Set a story state value.
    ///
    /// The variable's own name decides where it is stored, as in
    /// [`LouderContext::get_story_state`]. Returns false if the variable is
    /// local and its conversation is not loaded, since there would then be
    /// nowhere to put it.
    #[func]
    pub fn set_story_state(&mut self, conversation: LouderName, key: LouderName, value: Variant) -> bool {
      let value = LouderVariant::from_godot(self, &value);
      self.set_state(conversation, key, value)
    }

    /// Whether a story state variable has been written.
    #[func]
    pub fn has_story_state(&self, conversation: LouderName, key: LouderName) -> bool {
      self.has_state(conversation, key)
    }

    /// Every key written in this conversation, in ascending id order.
    #[func]
    pub fn get_conversation_state_keys(&self, conversation: LouderName) -> Array<i64> {
      self.conversation_state_keys(conversation)
        .into_iter()
        .map(|key| key as i64)
        .collect()
    }

    /// Every key written in either shared bucket, in ascending id order.
    #[func]
    pub fn get_shared_state_keys(&self) -> Array<i64> {
      self.shared_state_keys().into_iter().map(|key| key as i64).collect()
    }

    /// Every key written anywhere, local or shared, in ascending id order.
    #[func]
    pub fn get_all_state_keys(&self, conversation: LouderName) -> Array<i64> {
      self.all_state_keys(conversation)
        .into_iter()
        .map(|key| key as i64)
        .collect()
    }

    /// Where a story state variable of this name is stored, as a
    /// [`super::conversation::VariableScope`] index.
    #[func]
    pub fn get_state_scope(&self, key: LouderName) -> i32 {
      match self.scope_of(key) {
        VariableScope::LocalSaved => 0,
        VariableScope::SharedSaved => 1,
        VariableScope::LocalUnsaved => 2,
        VariableScope::SharedUnsaved => 3,
      }
    }

    /// The whole story state of a conversation, as JSON.
    #[func]
    pub fn read_state_json(&self, conversation: LouderName) -> GString {
      GString::from(json::state(self, conversation).as_str())
    }

    /// The story state, along with the positions and details of every actor, as
    /// JSON. Empty when the conversation is not loaded.
    #[func]
    pub fn read_snapshot_json(&self, conversation: LouderName) -> GString {
      match json::snapshot(self, conversation) {
        Some(document) => GString::from(document.as_str()),
        None => GString::from(""),
      }
    }

    // -- documents -------------------------------------------------------

    /// Load a conversation from a JSON document.
    ///
    /// The JSON is expected in array form. Problems with individual records do
    /// not fail the load: they are emitted as
    /// [`EventData::LoadError`](super::event::EventData) events instead,
    /// carrying the offending hub's position. Returns a
    /// [`LoadError`](super::json_loader::LoadError) code.
    #[func]
    pub fn load_conversation(&mut self, conversation: LouderName, document: GString) -> i32 {
      json_loader::load_conversation(self, conversation, &document.to_string())
    }

    /// Load the speaker config from a JSON document.
    ///
    /// Uses the same error scheme as
    /// [`LouderContext::load_conversation`], with speakers as the target
    /// objects instead of hubs. The config is a singleton as opposed to a named
    /// conversation instance, so this takes no conversation name.
    #[func]
    pub fn load_conversation_config(&mut self, document: GString) -> i32 {
      json_loader::load_config(self, &document.to_string())
    }

    // -- rules -----------------------------------------------------------

    /// The dialogue rules currently in effect, as bits.
    ///
    /// A fresh context has every rule on. See
    /// [`LouderDialogueRules`](super::rules::LouderDialogueRules) for reading
    /// and writing them by name.
    #[func]
    pub fn get_dialogue_rules(&self) -> i32 {
      self.rules().bits()
    }

    /// Replace the dialogue rules, as bits.
    ///
    /// A rule takes effect on the next move rather than being applied to the
    /// conversation as it stands. Bits outside
    /// [`super::rules::DialogueRules`] are ignored rather than
    /// refused, so a host carrying a configuration written for a newer engine
    /// still gets the rules this build knows about.
    #[func]
    pub fn set_dialogue_rules(&mut self, rules: i32) {
      self.set_rules(DialogueRules::from_bits_truncate(rules));
    }

    // -- movement --------------------------------------------------------

    /// Select a choice on an actor's current hub, moving it to the linked hub.
    ///
    /// The choice must be one the actor could actually take; see
    /// [`LouderContext::get_hub_choice_count`] for what that means. Auto hubs
    /// along the way are resolved by taking the first choice available at each
    /// one. Returns false if the actor, hub, or choice was missing, if the
    /// choice is not available to this actor, or if an auto hub ran out of
    /// choices.
    #[func]
    pub fn select_choice(&mut self, conversation: LouderName, actor: LouderActor, choice_index: i32) -> bool {
      Simulator::select_choice(self, conversation, actor, choice_index).is_ok()
    }

    /// Move an actor to another position as if it had moved through the network.
    ///
    /// Other actors sharing the actor's channel may be pulled along, exactly as
    /// if the actor had chosen. `fake_priority` and `fake_channel` override the
    /// destination's own when they are not the blanks.
    #[func]
    pub fn force_warp(
      &mut self,
      conversation: LouderName,
      actor: LouderActor,
      target_position: LouderPosition,
      fake_channel: LouderName,
      fake_priority: i32,
    ) -> bool {
      Simulator::force_move(self, conversation, actor, target_position, MotionKind::Warp, fake_channel, fake_priority).is_ok()
    }

    /// Move an actor to another position directly, letting followers follow it.
    #[func]
    pub fn force_position(
      &mut self,
      conversation: LouderName,
      actor: LouderActor,
      target_position: LouderPosition,
      fake_channel: LouderName,
      fake_priority: i32,
    ) -> bool {
      Simulator::force_move(self, conversation, actor, target_position, MotionKind::Force, fake_channel, fake_priority).is_ok()
    }


    #[func]
    fn make_name(&mut self, string_name: StringName) -> LouderName {
      let buffer = string_name.to_utf8_buffer().to_vec();
      let name_content =  LouderNameContent{ bytes: buffer };
      match probe_name(&self.names, &name_content) {
        LouderContextResult::Found(name) => name,
        LouderContextResult::NotFound(noname) => {
          self.names.insert(noname, name_content);
          return noname;
        }
      }
    }
    
    #[func]
    fn make_string_name(&mut self, louder_name: LouderName) -> StringName {
      if let Some(name) = self.names.get(&louder_name) {
        let buffer_array = godot::prelude::PackedArray::<u8>::from(name.bytes.clone());
        let gstring = buffer_array.get_string_from_utf8();
        let string_name = StringName::from(&gstring);
        return string_name;
      }

      StringName::default()
    }
  }
}