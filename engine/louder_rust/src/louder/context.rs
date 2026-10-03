use std::collections::VecDeque;
use std::hash::{DefaultHasher, Hash, Hasher};

use super::name::*;
use super::speaker::*;
use super::actor::{Actor, LouderActor};
use super::conversation::{Conversation, VariableMap, VariableScope, classify_variable};
use super::event::*;
use super::object::LouderVariant;

/// One queued ask, together with the verdict the host gave it.
///
/// An ask stays queued after being answered so that the host can keep walking
/// the cache, but it stops accepting answers once `verdict` is set.
struct PendingAsk {
  ask: LouderAsk,
  verdict: Option<i32>,
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
  speakers: Map<LouderName, Speaker>,
  conversations: Map<LouderName, Conversation>,
  actors: Map<LouderName, Vec<Actor>>,
  next_actor_id: Map<LouderName, LouderActor>,
  shared_saved: VariableMap,
  shared_unsaved: VariableMap,
  events: Vec<LouderEvent>,
  asks: VecDeque<PendingAsk>,
  pending_asks: usize,
}

impl LouderContext {
  pub fn new() -> LouderContext {
    let mut context = LouderContext {
      names: LouderNameMap::new(),
      // Interned eagerly so that "main" always has an id, even in a
      // conversation that never mentions it.
      main_channel: LOUDER_NAME_NONE,
      speakers: Map::new(),
      conversations: Map::new(),
      actors: Map::new(),
      next_actor_id: Map::new(),
      shared_saved: Map::new(),
      shared_unsaved: Map::new(),
      events: Vec::new(),
      asks: VecDeque::new(),
      pending_asks: 0,
    };

    context.main_channel = context.intern_name(LOUDER_MAIN_CHANNEL);
    context
  }

  /// The id of the default channel, which every conversation starts on.
  pub fn main_channel(&self) -> LouderName {
    self.main_channel
  }

  pub fn push_event(&mut self, event: LouderEvent) -> usize {
    self.events.push(event);
    self.events.len() - 1
  }

  pub fn push_ask(&mut self, ask: LouderAsk) -> usize {
    self.asks.push_back(PendingAsk { ask, verdict: None });
    self.pending_asks += 1;
    self.asks.len() - 1
  }

  /// The number of asks still waiting for a verdict.
  pub fn pending_asks(&self) -> usize {
    self.pending_asks
  }

  /// Record `verdict` for the ask at `index` and move it to the back of the
  /// queue.
  ///
  /// Returns false when `index` is out of range or that ask was already
  /// answered. Answered asks remain readable so the host can keep polling the
  /// whole cache, but they stop accepting further answers.
  pub fn answer_ask(&mut self, index: usize, verdict: i32) -> bool {
    if index >= self.asks.len() {
      return false;
    }

    let pending = &self.asks[index];
    if pending.verdict.is_some() {
      return false;
    }

    let mut entry = self.asks.remove(index).unwrap();
    entry.verdict = Some(verdict);
    self.asks.push_back(entry);
    self.pending_asks -= 1;
    true
  }

  /// The ask at `index`, or `None` if the index is out of range.
  pub fn ask(&self, index: usize) -> Option<&LouderAsk> {
    self.asks.get(index).map(|entry| &entry.ask)
  }

  /// Whether the ask at `index` has been answered.
  pub fn is_ask_answered(&self, index: usize) -> bool {
    match self.asks.get(index) {
      Some(entry) => entry.verdict.is_some(),
      None => true,
    }
  }

  /// The verdicts recorded so far, oldest first.
  pub fn ask_verdicts(&self) -> Vec<i32> {
    self.asks.iter().filter_map(|entry| entry.verdict).collect()
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

  /// Remove a conversation along with its actors.
  ///
  /// Shared story state is left alone, since it outlives any one conversation.
  pub fn remove_conversation(&mut self, name: LouderName) -> bool {
    self.actors.remove(&name);
    self.next_actor_id.remove(&name);
    self.conversations.remove(&name).is_some()
  }

  /// Every conversation name in ascending order.
  pub fn conversation_names(&self) -> Vec<LouderName> {
    let mut names: Vec<LouderName> = self.conversations.keys().copied().collect();
    names.sort_unstable();
    names
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

  /// Append an actor, giving it the next unused id for its conversation.
  ///
  /// Ids are never reused, so ascending id order is a stable ordering even
  /// after actors are removed.
  pub fn add_actor(&mut self, conversation: LouderName, mut actor: Actor) -> LouderActor {
    let next = self.next_actor_id.entry(conversation).or_insert(0);
    let id = *next;
    *next += 1;
    actor.set_id(id);

    let actors = self.actors.entry(conversation).or_default();
    actors.push(actor);
    id
  }

  pub fn remove_actor(&mut self, conversation: LouderName, actor: LouderActor) -> bool {
    match self.actors.get_mut(&conversation) {
      Some(actors) => {
        let before = actors.len();
        actors.retain(|a| a.id() != actor);
        actors.len() != before
      }
      None => false,
    }
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

  fn variables_mut(&mut self, conversation: LouderName, scope: VariableScope) -> &mut VariableMap {
    match scope {
      VariableScope::LocalSaved | VariableScope::LocalUnsaved => {
        let conversation = self
          .conversations
          .get_mut(&conversation)
          .expect("story state was requested for a conversation that is not loaded");
        conversation.variables_mut(scope)
      }
      VariableScope::SharedSaved => &mut self.shared_saved,
      VariableScope::SharedUnsaved => &mut self.shared_unsaved,
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
    let scope = self.scope_of(key);
    match self.variables(conversation, scope) {
      Some(variables) => variables.get(&key).cloned().unwrap_or_default(),
      None => LouderVariant::Int(0),
    }
  }

  /// Write a story state variable, storing it in the bucket its name implies.
  pub fn set_state(&mut self, conversation: LouderName, key: LouderName, value: LouderVariant) {
    let scope = self.scope_of(key);
    self.variables_mut(conversation, scope).insert(key, value);
  }

  /// Whether a story state variable has been written.
  pub fn has_state(&self, conversation: LouderName, key: LouderName) -> bool {
    let scope = self.scope_of(key);
    match self.variables(conversation, scope) {
      Some(variables) => variables.contains_key(&key),
      None => false,
    }
  }

  /// Every key of the bucket `key` falls into, in ascending id order.
  pub fn state_keys(&self, conversation: LouderName, key: LouderName) -> Vec<LouderName> {
    let scope = self.scope_of(key);
    match self.variables(conversation, scope) {
      Some(variables) => {
        let mut keys: Vec<LouderName> = variables.keys().copied().collect();
        keys.sort_unstable();
        keys
      }
      None => Vec::new(),
    }
  }

  /// Every key written in the conversation, in ascending id order.
  ///
  /// Covers both local buckets; shared buckets are reported separately by
  /// [`Self::shared_state_keys`].
  pub fn conversation_state_keys(&self, conversation: LouderName) -> Vec<LouderName> {
    let mut keys = match self.conversations.get(&conversation) {
      Some(conversation) => conversation.variables(VariableScope::LocalSaved).keys().copied().collect::<Vec<LouderName>>(),
      None => Vec::new(),
    };
    keys.extend(
      self.conversations
        .get(&conversation)
        .map(|c| c.variables(VariableScope::LocalUnsaved).keys().copied().collect::<Vec<LouderName>>())
        .unwrap_or_default(),
    );
    keys.sort_unstable();
    keys.dedup();
    keys
  }

  /// Every key written in either shared bucket, in ascending id order.
  pub fn shared_state_keys(&self) -> Vec<LouderName> {
    let mut keys: Vec<LouderName> = self.shared_saved.keys().copied().collect();
    keys.extend(self.shared_unsaved.keys().copied());
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
    let mut hasher = DefaultHasher::new();
    name_context.bytes.hash(&mut hasher);

    let mut name = hasher.finish() as LouderName;
    let mut iterations = 0;
    while self.names.contains_key(&name) {
      if &self.names[&name] == name_context {
        return LouderContextResult::Found(name);
      }

      name = name.wrapping_add(iterations * iterations + 1);
      iterations += 1;
    }

    LouderContextResult::NotFound(name)
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

impl EventCache<LouderEvent> for LouderContext {
  fn get_event(&self, index: usize) -> Option<&LouderEvent> {
    self.events.get(index)
  }

  fn clear_events(&mut self) {
    self.events.clear()
  }
}

impl EventCache<LouderAsk> for LouderContext {
  fn get_event(&self, index: usize) -> Option<&LouderAsk> {
    self.ask(index)
  }

  fn clear_events(&mut self) {
    self.asks.clear();
    self.pending_asks = 0;
  }
}

impl AskListener for LouderContext {
  fn answer_ask(&mut self, index: usize, verdict: i32) -> bool {
    self.answer_ask(index, verdict)
  }
}

impl SpeakerMap for LouderContext {
  fn get_speaker(&self, speaker_name: LouderName) -> Option<&Speaker> {
    self.speakers.get(&speaker_name)
  }
}