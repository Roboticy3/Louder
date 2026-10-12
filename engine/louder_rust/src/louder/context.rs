use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};

use super::name::*;
use super::speaker::*;
use super::actor::{Actor, LouderActor, LOUDER_ACTOR_NONE};
use super::conversation::{Conversation, VariableMap, VariableScope, classify_variable};
use super::event::*;
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