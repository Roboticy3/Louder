//! The engine's whole vocabulary, as one class.

use crate::actor::Actor;
use crate::conversation::scope_index;
use crate::conversation::Conversation;
use crate::engine;
// The name table is reached through these two, and importing them is what makes
// `intern_name`, `resolve_name` and `get_name_content` callable on the engine
// context below.
use crate::engine::{LouderInternContext, LouderNameContext};
use crate::event::{LouderAsk, LouderEvent};
use crate::hub::Hub;
use crate::speaker::Speaker;
use crate::variant;
use louder_rust::godot_api;

use godot::prelude::*;

/// One running dialogue.
///
/// This is the only class a host holds on to. Its name table, conversations,
/// actors, speaker config, story state, event queue, and ask cache all live in
/// it, and everything else is read back out of it as a snapshot.
///
/// This is the same surface the `extern "C"` entry points describe, with the
/// boundary's plumbing removed. In particular, strings are `String`s rather than
/// pointers into a caller's buffer, and an object the engine holds is handed
/// over as a copy: writing to that copy does not move the engine's own, so
/// anything that changes a conversation goes through a method on this class.
#[derive(GodotClass)]
#[class(tool)]
pub struct LouderContext {
  pub(crate) inner: engine::LouderContext,
}

godot_api! {
  impl ::godot::classes::IRefCounted for LouderContext {
    fn init(_base: ::godot::obj::Base<::godot::classes::RefCounted>) -> Self {
      // The default channel is interned eagerly by `new`, which a generated
      // `init` could not do, since it fills each field with its `Default` and
      // has no way to intern one of them.
      LouderContext { inner: engine::LouderContext::new() }
    }
  }
}

godot_api! {
  impl LouderContext {
    // -- conversations ---------------------------------------------------

    /// Whether a conversation is loaded under this name.
    #[func]
    pub fn has_conversation(&self, conversation: engine::LouderName) -> bool {
      self.inner.get_conversation(conversation).is_some()
    }

    /// A loaded conversation, as a copy of what the engine holds.
    #[func]
    pub fn get_conversation(&self, conversation: engine::LouderName) -> Option<Gd<Conversation>> {
      self
        .inner
        .get_conversation(conversation)
        .map(|entry| Gd::from_object(Conversation { inner: entry.clone() }))
    }

    /// Remove a conversation and all associated actors.
    ///
    /// Shared story state is kept, since it outlives any one conversation.
    /// Returns false if the conversation was not loaded.
    #[func]
    pub fn remove_conversation(&mut self, conversation: engine::LouderName) -> bool {
      self.inner.remove_conversation(conversation)
    }

    // -- the event queue -------------------------------------------------

    /// The queued event at `index`, if there is one.
    ///
    /// Poll from index 0 upwards until this comes back empty, then call
    /// [`LouderContext::flush_events`].
    #[func]
    pub fn get_event(&self, index: i32) -> Option<Gd<LouderEvent>> {
      if index < 0 {
        return None;
      }
      self
        .inner
        .event(index as usize)
        .map(|event| Gd::from_object(LouderEvent { inner: event.clone() }))
    }

    /// How many events are queued.
    #[func]
    pub fn get_event_count(&self) -> i32 {
      self.inner.event_count() as i32
    }

    /// Remove every queued event.
    #[func]
    pub fn flush_events(&mut self) {
      self.inner.clear_events();
    }

    /// Push an event onto the queue, used for testing.
    ///
    /// Nothing about the event is interpreted here, so the caller is filling it
    /// in themselves.
    #[func]
    pub fn push_event(&mut self, event: Gd<LouderEvent>) {
      self.inner.push_event(event.bind().inner.clone());
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
      self.inner.ask(index as usize).map(|ask| Gd::from_object(LouderAsk { inner: ask.clone() }))
    }

    /// Push an ask onto the cache, used for testing.
    #[func]
    pub fn push_ask(&mut self, ask: Gd<LouderAsk>) {
      self.inner.push_ask(ask.bind().inner.clone());
    }

    /// Answer the ask at `index` with `verdict`.
    ///
    /// Answering marks the ask complete and moves it to the back of the cache,
    /// so indexes shift: read the cache again after each answer. Returns false
    /// if `index` was out of range or that ask had already been answered.
    #[func]
    pub fn answer_ask(&mut self, index: i32, verdict: i32) -> bool {
      if index < 0 {
        return false;
      }
      self.inner.answer_ask(index as usize, verdict)
    }

    // -- traces ----------------------------------------------------------

    /// The traced event at `index`, if there is one.
    ///
    /// The trace is every event the engine has created, in creation order.
    /// Unlike the event queue it is not emptied by
    /// [`LouderContext::flush_events`], so it survives being processed.
    #[func]
    pub fn get_traced_event(&self, index: i32) -> Option<Gd<LouderEvent>> {
      if index < 0 {
        return None;
      }
      self
        .inner
        .traced_event(index as usize)
        .map(|event| Gd::from_object(LouderEvent { inner: event.clone() }))
    }

    /// How many events the trace holds.
    #[func]
    pub fn get_traced_event_count(&self) -> i32 {
      self.inner.traced_event_count() as i32
    }

    /// Empty the event trace.
    #[func]
    pub fn clear_traced_events(&mut self) {
      self.inner.clear_traced_events();
    }

    /// The traced ask at `index`, if there is one.
    ///
    /// The trace is every ask the engine has created, in creation order.
    /// Answering an ask does not move or remove its traced copy.
    #[func]
    pub fn get_traced_ask(&self, index: i32) -> Option<Gd<LouderAsk>> {
      if index < 0 {
        return None;
      }
      self
        .inner
        .traced_ask(index as usize)
        .map(|ask| Gd::from_object(LouderAsk { inner: ask.clone() }))
    }

    /// How many asks the trace holds.
    #[func]
    pub fn get_traced_ask_count(&self) -> i32 {
      self.inner.traced_ask_count() as i32
    }

    /// Empty the ask trace.
    #[func]
    pub fn clear_traced_asks(&mut self) {
      self.inner.clear_traced_asks();
    }

    /// Empty both the event trace and the ask trace.
    #[func]
    pub fn clear_trace(&mut self) {
      self.inner.clear_trace();
    }

    // -- speakers --------------------------------------------------------

    /// A speaker from the config, as a copy of what the engine holds.
    ///
    /// Empty for a speaker that was never declared, which is treated as public.
    #[func]
    pub fn get_speaker(&self, speaker: engine::LouderName) -> Option<Gd<Speaker>> {
      engine::SpeakerMap::get_speaker(&self.inner, speaker)
        .map(|entry| Gd::from_object(Speaker { inner: entry.clone() }))
    }

    /// Every speaker name, in ascending order.
    #[func]
    pub fn speaker_names(&self) -> Array<i64> {
      self.inner.speaker_names().into_iter().map(|name| name as i64).collect()
    }

    /// The flag bits of a speaker, or 0 if it is not declared.
    #[func]
    pub fn get_speaker_flags(&self, speaker: engine::LouderName) -> i32 {
      match engine::SpeakerMap::get_speaker(&self.inner, speaker) {
        Some(entry) => entry.flags().bits(),
        None => engine::SpeakerFlags::empty().bits(),
      }
    }

    /// Whether hubs with this speaker are visible to actors who are not
    /// listening to it. An undefined speaker is public.
    #[func]
    pub fn is_speaker_public(&self, speaker: engine::LouderName) -> bool {
      engine::SpeakerMap::is_speaker_public(&self.inner, speaker)
    }

    /// A speaker's additional data under `key`, read as an integer, or 0.
    #[func]
    pub fn get_speaker_additional_int(&self, speaker: engine::LouderName, key: engine::LouderName) -> i32 {
      match engine::SpeakerMap::get_speaker(&self.inner, speaker).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_int(),
        None => 0,
      }
    }

    /// A speaker's additional data under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_speaker_additional_float(&self, speaker: engine::LouderName, key: engine::LouderName) -> f64 {
      match engine::SpeakerMap::get_speaker(&self.inner, speaker).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_float() as f64,
        None => 0.0,
      }
    }

    /// A speaker's additional data under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_speaker_additional_name(&self, speaker: engine::LouderName, key: engine::LouderName) -> engine::LouderName {
      match engine::SpeakerMap::get_speaker(&self.inner, speaker).and_then(|entry| entry.additional_get(key)) {
        Some(engine::LouderVariant::String(name)) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }

    /// The speakers `speaker` is listening to, after `start`.
    ///
    /// Pass [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE) as `start` to begin.
    #[func]
    pub fn get_next_speaker_listen_to(&self, speaker: engine::LouderName, start: engine::LouderName) -> engine::LouderName {
      engine::query::next_speaker_listen_to(&self.inner, speaker, start)
    }

    /// Remove all speakers in the config.
    ///
    /// Conversations and actors are untouched, so hubs naming a now-missing
    /// speaker become public again.
    #[func]
    pub fn clear_config(&mut self) {
      self.inner.clear_config();
    }

    // -- actors ----------------------------------------------------------

    /// An actor of a conversation, as a copy of what the engine holds.
    #[func]
    pub fn get_actor(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
    ) -> Option<Gd<Actor>> {
      self
        .inner
        .get_actor(conversation, actor)
        .map(|entry| Gd::from_object(Actor { inner: entry.clone() }))
    }

    /// Every actor id of a conversation, in ascending order.
    #[func]
    pub fn get_actor_ids(&self, conversation: engine::LouderName) -> Array<i64> {
      self.inner.actors(conversation).iter().map(|entry| entry.id() as i64).collect()
    }

    /// Add an actor to a conversation and return the id given to it.
    ///
    /// The actor starts at `start_position` on the default channel, with
    /// priority 0. Ids are per conversation and never reused, so ascending
    /// order is a stable iteration order. Returns
    /// [`LOUDER_ACTOR_NONE`](engine::LOUDER_ACTOR_NONE) if the conversation is
    /// not loaded.
    #[func]
    pub fn add_actor(
      &mut self,
      conversation: engine::LouderName,
      start_position: engine::LouderPosition,
      flags: i32,
      speak_as: Array<i64>,
      listen_to: Array<i64>,
    ) -> engine::LouderActor {
      if self.inner.get_conversation(conversation).is_none() {
        return engine::LOUDER_ACTOR_NONE;
      }

      let actor = engine::Actor::new(
        start_position,
        engine::ActorFlags::from_bits_truncate(flags),
        speak_as.iter_shared().map(|name| name as engine::LouderName).collect(),
        listen_to.iter_shared().map(|name| name as engine::LouderName).collect(),
        self.inner.main_channel(),
      );

      self.inner.add_actor(conversation, actor)
    }

    /// Remove an actor from a conversation, reporting whether it was there.
    #[func]
    pub fn remove_actor(&mut self, conversation: engine::LouderName, actor: engine::LouderActor) -> bool {
      self.inner.remove_actor(conversation, actor)
    }

    /// The channel of an actor, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    ///
    /// Actors with [`ActorFlags::SPECTATOR`](engine::ActorFlags) or
    /// [`ActorFlags::DEBUG`](engine::ActorFlags) are not part of the flow and
    /// read as blank.
    #[func]
    pub fn get_actor_channel(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
    ) -> engine::LouderName {
      match self.inner.get_actor(conversation, actor) {
        Some(entry) => entry.channel(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The position of an actor, or `-1` if it does not exist.
    #[func]
    pub fn get_actor_position(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
    ) -> engine::LouderPosition {
      match self.inner.get_actor(conversation, actor) {
        Some(entry) => entry.position(),
        None => -1,
      }
    }

    /// The flag bits of an actor, or 0 if it does not exist.
    #[func]
    pub fn get_actor_flags(&self, conversation: engine::LouderName, actor: engine::LouderActor) -> i32 {
      match self.inner.get_actor(conversation, actor) {
        Some(entry) => entry.flags().bits(),
        None => engine::ActorFlags::empty().bits(),
      }
    }

    /// The priority of an actor. Actors outside the flow read as 0.
    #[func]
    pub fn get_actor_priority(&self, conversation: engine::LouderName, actor: engine::LouderActor) -> i32 {
      match self.inner.get_actor(conversation, actor) {
        Some(entry) => entry.priority(),
        None => 0,
      }
    }

    /// An actor's additional data under `key`, read as an integer, or 0.
    #[func]
    pub fn get_actor_additional_int(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      key: engine::LouderName,
    ) -> i32 {
      match self.inner.get_actor(conversation, actor).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_int(),
        None => 0,
      }
    }

    /// An actor's additional data under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_actor_additional_float(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      key: engine::LouderName,
    ) -> f64 {
      match self.inner.get_actor(conversation, actor).and_then(|entry| entry.additional_get(key)) {
        Some(value) => value.as_float() as f64,
        None => 0.0,
      }
    }

    /// An actor's additional data under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_actor_additional_name(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      key: engine::LouderName,
    ) -> engine::LouderName {
      match self.inner.get_actor(conversation, actor).and_then(|entry| entry.additional_get(key)) {
        Some(engine::LouderVariant::String(name)) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }

    /// The speakers an actor speaks as, after `start`.
    #[func]
    pub fn next_actor_speak_as(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_actor_speak_as(&self.inner, conversation, actor, start)
    }

    /// The base speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening_base(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_actor_listening_base(&self.inner, conversation, actor, start)
    }

    /// The speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening(
      &self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_actor_listening(&self.inner, conversation, actor, start)
    }

    /// The actors in a channel, after `start`.
    #[func]
    pub fn next_actor_in_channel(
      &self,
      conversation: engine::LouderName,
      channel: engine::LouderName,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_in_channel(&self.inner, conversation, channel, start)
    }

    /// The actors in a channel at one position, after `start`.
    #[func]
    pub fn next_actor_in_channel_and_position(
      &self,
      conversation: engine::LouderName,
      channel: engine::LouderName,
      position: engine::LouderPosition,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_in_channel_and_position(&self.inner, conversation, channel, position, start)
    }

    /// The actors speaking as a speaker, after `start`.
    #[func]
    pub fn next_actor_speaking_as(
      &self,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_speaking_as(&self.inner, conversation, speaker, start)
    }

    /// The actors listening to a speaker, after `start`.
    #[func]
    pub fn next_actor_listening_to(
      &self,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_listening_to(&self.inner, conversation, speaker, start)
    }

    /// The actors carrying certain flags, after `start`.
    #[func]
    pub fn next_actor_from_flags(
      &self,
      conversation: engine::LouderName,
      include: i32,
      exact: bool,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_from_flags(
        &self.inner,
        conversation,
        engine::ActorFlags::from_bits_truncate(include),
        i32::from(exact),
        start,
      )
    }

    // -- hubs ------------------------------------------------------------

    /// A hub, as a copy of what the engine holds.
    #[func]
    pub fn get_hub(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> Option<Gd<Hub>> {
      self.inner.hub(conversation, position).map(|hub| Gd::from_object(Hub { inner: hub.clone() }))
    }

    /// The speaker of a hub, or [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    ///
    /// A hub with no speaker is visible to everyone, as is one that does not
    /// exist.
    #[func]
    pub fn get_hub_speaker(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> engine::LouderName {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.speaker(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The speaker of a hub, as an actor would hear it.
    ///
    /// A hub with a speaker of its own speaks as that speaker. A hub without one
    /// is anonymous: it speaks as the first voice in the speak as list of
    /// whoever is standing on it, so the answer depends on the actor, and a hub
    /// that has no speaker of its own answers differently for each of them.
    #[func]
    pub fn get_hub_speaker_for_actor(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      actor_perspective: engine::LouderActor,
    ) -> engine::LouderName {
      engine::Simulator::hub_speaker_for(&self.inner, conversation, actor_perspective, position)
    }

    /// The text of a hub as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_hub_text(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> engine::LouderName {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.text_name(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The number of choices on a hub from an actor's perspective.
    ///
    /// Only choices the actor could actually take are counted, so this applies
    /// the visibility, recipient, and spoke rules. Returns 0 for an unknown hub
    /// or actor.
    #[func]
    pub fn get_hub_choice_count(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      actor_perspective: engine::LouderActor,
    ) -> i32 {
      engine::Simulator::available_choices(&self.inner, conversation, actor_perspective, position).len() as i32
    }

    /// The indices of the choices on a hub that `actor` may take.
    #[func]
    pub fn get_hub_choices(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      actor_perspective: engine::LouderActor,
    ) -> Array<i64> {
      engine::Simulator::available_choices(&self.inner, conversation, actor_perspective, position)
        .into_iter()
        .map(|index| index as i64)
        .collect()
    }

    /// The rule behind a choice's availability, as a `ChoiceRule` value.
    ///
    /// Returns -1 for a negative choice index. Other values are the engine's
    /// `ChoiceRule` discriminants, including `Invisible` and `Held`.
    #[func]
    pub fn get_hub_choice_rule(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      actor_perspective: engine::LouderActor,
      choice_index: i32,
    ) -> i32 {
      if choice_index < 0 {
        return -1;
      }
      engine::Simulator::choice_rule(
        &self.inner,
        conversation,
        actor_perspective,
        position,
        choice_index as usize,
      ) as i32
    }

    /// The text of a choice on a hub as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    ///
    /// A choice has no text of its own, so this is the text of the hub it links
    /// to, and it is blank unless the choice is available to `actor`.
    #[func]
    pub fn get_hub_choice_text(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      actor_perspective: engine::LouderActor,
      choice_index: i32,
    ) -> engine::LouderName {
      match engine::query::available_choice(
        &self.inner,
        conversation,
        position,
        actor_perspective,
        choice_index.max(0) as usize,
      ) {
        Some(choice) => choice.text_name(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The position a choice on a hub leads to, or `-1`.
    #[func]
    pub fn get_hub_choice_link(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      actor_perspective: engine::LouderActor,
      choice_index: i32,
    ) -> engine::LouderPosition {
      match engine::query::available_choice(
        &self.inner,
        conversation,
        position,
        actor_perspective,
        choice_index.max(0) as usize,
      ) {
        Some(choice) => choice.position(),
        None => -1,
      }
    }

    /// The expressions on a hub, one name per entry.
    #[func]
    pub fn get_hub_expressions(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> Array<i64> {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.expressions().iter().map(|name| *name as i64).collect(),
        None => Array::new(),
      }
    }

    /// The whole expression block of a hub as one name, newline separated.
    #[func]
    pub fn get_hub_expressions_joined(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> engine::LouderName {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.expressions_joined(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The callbacks on a hub, one name per entry.
    #[func]
    pub fn get_hub_callbacks(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> Array<i64> {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.callbacks().iter().map(|name| *name as i64).collect(),
        None => Array::new(),
      }
    }

    /// The whole callback block of a hub as one name, newline separated.
    #[func]
    pub fn get_hub_callbacks_joined(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> engine::LouderName {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.callbacks_joined(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The flag bits of a hub, or 0 if it does not exist.
    #[func]
    pub fn get_hub_flags(&self, conversation: engine::LouderName, position: engine::LouderPosition) -> i32 {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.flags().bits(),
        None => engine::HubFlags::empty().bits(),
      }
    }

    /// The recipients of a hub, after `start`.
    ///
    /// This comes back blank straight away for a hub with no recipients, which
    /// is how a hub that is not restricted reads.
    #[func]
    pub fn next_hub_recipient(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_hub_recipient(&self.inner, conversation, position, start)
    }

    /// The tags of a hub, after `start`.
    #[func]
    pub fn next_hub_tag(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_hub_tag(&self.inner, conversation, position, start)
    }

    /// The channel a hub moves an actor into, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_hub_channel(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
    ) -> engine::LouderName {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.channel(),
        None => engine::LOUDER_NAME_NONE,
      }
    }

    /// The priority of a hub, or 0 if it does not exist.
    #[func]
    pub fn get_hub_priority(&self, conversation: engine::LouderName, position: engine::LouderPosition) -> i32 {
      match self.inner.hub(conversation, position) {
        Some(hub) => hub.priority(),
        None => 0,
      }
    }

    /// A hub's additional data under `key`, read as an integer, or 0.
    #[func]
    pub fn get_hub_additional_int(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      key: engine::LouderName,
    ) -> i32 {
      match self.inner.hub(conversation, position).and_then(|hub| hub.additional_get(key)) {
        Some(value) => value.as_int(),
        None => 0,
      }
    }

    /// A hub's additional data under `key`, read as a float, or 0.0.
    #[func]
    pub fn get_hub_additional_float(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      key: engine::LouderName,
    ) -> f64 {
      match self.inner.hub(conversation, position).and_then(|hub| hub.additional_get(key)) {
        Some(value) => value.as_float() as f64,
        None => 0.0,
      }
    }

    /// A hub's additional data under `key` as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn get_hub_additional_name(
      &self,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      key: engine::LouderName,
    ) -> engine::LouderName {
      match self.inner.hub(conversation, position).and_then(|hub| hub.additional_get(key)) {
        Some(engine::LouderVariant::String(name)) => *name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }

    /// The hubs in a channel, after `start`.
    #[func]
    pub fn next_hub_by_channel(
      &self,
      conversation: engine::LouderName,
      channel: engine::LouderName,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_by_channel(&self.inner, conversation, channel, start)
    }

    /// The hubs spoken by a speaker, after `start`.
    #[func]
    pub fn next_hub_spoken_by(
      &self,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_spoken_by(&self.inner, conversation, speaker, start)
    }

    /// The hubs heard by a speaker, after `start`.
    #[func]
    pub fn next_hub_heard_by(
      &self,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_heard_by(&self.inner, conversation, speaker, start)
    }

    /// The hubs with a priority within the inclusive bounds, after `start`.
    #[func]
    pub fn next_hub_by_priority(
      &self,
      conversation: engine::LouderName,
      priority_min: i32,
      priority_max: i32,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_by_priority(&self.inner, conversation, priority_min, priority_max, start)
    }

    /// The hubs carrying all, or exactly, the given tags, after `start`.
    #[func]
    pub fn next_hub_from_tags(
      &self,
      conversation: engine::LouderName,
      include: Array<i64>,
      exact: bool,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      let wanted: Vec<engine::LouderName> = include.iter_shared().map(|tag| tag as engine::LouderName).collect();
      engine::query::next_hub_from_tags(&self.inner, conversation, &wanted, exact, start)
    }

    /// The hubs carrying certain flags, after `start`.
    #[func]
    pub fn next_hub_from_flags(
      &self,
      conversation: engine::LouderName,
      include: i32,
      exact: bool,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_from_flags(
        &self.inner,
        conversation,
        engine::HubFlags::from_bits_truncate(include),
        i32::from(exact),
        start,
      )
    }

    /// The tags of a speaker, one after another.
    ///
    /// Pass [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE) as `start` to begin,
    /// then pass the previous result until [`LOUDER_NAME_NONE`] comes back.
    #[func]
    pub fn next_speaker_tag(
      &self,
      speaker: engine::LouderName,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_speaker_tag(&self.inner, speaker, start)
    }

    // -- story state -----------------------------------------------------

    /// A story state value.
    ///
    /// The variable's own name decides where it is stored: a leading underscore
    /// means it is not saved, and an uppercase first letter means it is shared
    /// across conversations. Variables that have never been written read as 0,
    /// as does anything read from a conversation that is not loaded.
    #[func]
    pub fn get_story_state(&self, conversation: engine::LouderName, state: engine::LouderName) -> Variant {
      variant::to_godot_from(&self.inner.get_state(conversation, state), self)
    }

    /// Set a story state value.
    ///
    /// The variable's own name decides where it is stored, as in
    /// [`LouderContext::get_story_state`]. Returns false if the variable is
    /// local and its conversation is not loaded, since there would then be
    /// nowhere to put it.
    #[func]
    pub fn set_story_state(
      &mut self,
      conversation: engine::LouderName,
      key: engine::LouderName,
      value: Variant,
    ) -> bool {
      let value = variant::from_godot_from(self, &value);
      self.inner.set_state(conversation, key, value)
    }

    /// Whether a story state variable has been written.
    #[func]
    pub fn has_story_state(&self, conversation: engine::LouderName, key: engine::LouderName) -> bool {
      self.inner.has_state(conversation, key)
    }

    /// Every key written in this conversation, in ascending id order.
    #[func]
    pub fn get_conversation_state_keys(&self, conversation: engine::LouderName) -> Array<i64> {
      self.inner.conversation_state_keys(conversation).into_iter().map(|key| key as i64).collect()
    }

    /// Every key written in either shared bucket, in ascending id order.
    #[func]
    pub fn get_shared_state_keys(&self) -> Array<i64> {
      self.inner.shared_state_keys().into_iter().map(|key| key as i64).collect()
    }

    /// Every key written anywhere, local or shared, in ascending id order.
    #[func]
    pub fn get_all_state_keys(&self, conversation: engine::LouderName) -> Array<i64> {
      self.inner.all_state_keys(conversation).into_iter().map(|key| key as i64).collect()
    }

    /// Where a story state variable of this name is stored, as a
    /// [`VariableScope`](engine::VariableScope) index.
    #[func]
    pub fn get_state_scope(&self, key: engine::LouderName) -> i32 {
      scope_index(self.inner.scope_of(key))
    }

    /// The whole story state of a conversation, as JSON.
    #[func]
    pub fn read_state_json(&self, conversation: engine::LouderName) -> GString {
      GString::from(engine::json::state(&self.inner, conversation).as_str())
    }

    /// The story state, along with the positions and details of every actor, as
    /// JSON. Empty when the conversation is not loaded.
    #[func]
    pub fn read_snapshot_json(&self, conversation: engine::LouderName) -> GString {
      match engine::json::snapshot(&self.inner, conversation) {
        Some(document) => GString::from(document.as_str()),
        None => GString::from(""),
      }
    }

    // -- documents -------------------------------------------------------

    /// Load a conversation from a JSON document.
    ///
    /// The JSON is expected in array form. Problems with individual records do
    /// not fail the load: they are emitted as [`LouderEvent`] events instead,
    /// carrying the offending hub's position. Returns a
    /// [`LouderLoader`](crate::loader::LouderLoader) code.
    #[func]
    pub fn load_conversation(&mut self, conversation: engine::LouderName, document: GString) -> i32 {
      engine::load_conversation(&mut self.inner, conversation, &document.to_string())
    }

    /// Load the speaker config from a JSON document.
    ///
    /// Uses the same error scheme as [`LouderContext::load_conversation`], with
    /// speakers as the target objects instead of hubs. The config is a singleton
    /// as opposed to a named conversation instance, so this takes no
    /// conversation name.
    #[func]
    pub fn load_conversation_config(&mut self, document: GString) -> i32 {
      engine::load_config(&mut self.inner, &document.to_string())
    }

    // -- rules -----------------------------------------------------------

    /// The dialogue rules currently in effect, as bits.
    ///
    /// A fresh context has every rule on. See
    /// [`LouderDialogueRules`](crate::rules::LouderDialogueRules) for reading
    /// and writing them by name.
    #[func]
    pub fn get_dialogue_rules(&self) -> i32 {
      self.inner.rules().bits()
    }

    /// Replace the dialogue rules, as bits.
    ///
    /// A rule takes effect on the next move rather than being applied to the
    /// conversation as it stands. Bits outside
    /// [`DialogueRules`](engine::DialogueRules) are ignored rather than refused,
    /// so a host carrying a configuration written for a newer engine still gets
    /// the rules this build knows about.
    #[func]
    pub fn set_dialogue_rules(&mut self, rules: i32) {
      self.inner.set_rules(engine::DialogueRules::from_bits_truncate(rules));
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
    pub fn select_choice(
      &mut self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      choice_index: i32,
    ) -> bool {
      engine::Simulator::select_choice(&mut self.inner, conversation, actor, choice_index).is_ok()
    }

    /// Move an actor to another position as if it had moved through the network.
    ///
    /// Other actors sharing the actor's channel may be pulled along, exactly as
    /// if the actor had chosen. `fake_priority` and `fake_channel` override the
    /// destination's own when they are not the blanks.
    #[func]
    pub fn force_warp(
      &mut self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      target_position: engine::LouderPosition,
      fake_channel: engine::LouderName,
      fake_priority: i32,
    ) -> bool {
      engine::Simulator::force_move(
        &mut self.inner,
        conversation,
        actor,
        target_position,
        engine::MotionKind::Warp,
        fake_channel,
        fake_priority,
      )
      .is_ok()
    }

    /// Move an actor to another position directly, letting followers follow it.
    #[func]
    pub fn force_position(
      &mut self,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      target_position: engine::LouderPosition,
      fake_channel: engine::LouderName,
      fake_priority: i32,
    ) -> bool {
      engine::Simulator::force_move(
        &mut self.inner,
        conversation,
        actor,
        target_position,
        engine::MotionKind::Force,
        fake_channel,
        fake_priority,
      )
      .is_ok()
    }

    // -- names -----------------------------------------------------------

    /// The id `text` stands for, assigning one if it does not have it yet.
    ///
    /// A name that was already declared keeps the id it had. Godot has already
    /// decoded the `StringName` into UTF-8 by the time it gets here, and those
    /// are the bytes the engine keeps.
    #[func]
    pub fn make_name(&mut self, text: StringName) -> engine::LouderName {
      self.inner.intern_name(&text.to_string())
    }

    /// The name an id stands for, or an empty `StringName` if there is none.
    #[func]
    pub fn make_string_name(&self, name: engine::LouderName) -> StringName {
      let buffer = match self.inner.get_name_content(&name) {
        Some(content) => content.bytes.clone(),
        None => return StringName::default(),
      };
      let gstring = PackedByteArray::from(buffer).get_string_from_utf8();
      StringName::from(&gstring)
    }

    /// Convert a Godot `StringName` to an engine name, assigning one if needed.
    #[func]
    pub fn to_name(&mut self, text: StringName) -> engine::LouderName {
      self.make_name(text)
    }

    /// Convert an engine name back to a Godot `StringName`.
    #[func]
    pub fn to_string_name(&self, name: engine::LouderName) -> StringName {
      self.make_string_name(name)
    }
  }
}