use super::context::LouderContext;
use super::event::*;
use super::name::*;
use super::object::LouderVariant;
use super::expression::{Evaluation, Evaluator, EvaluationScope, LouderExpression};
use super::actor::{Actor, ActorFlags, LouderActor, LOUDER_ACTOR_NONE};
use super::hub::{Hub, LouderPosition};
use super::speaker::SpeakerMap;

/// Why an actor is moving. Recorded on the motion carried by every event the
/// move produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionKind {
  /// The actor selected a choice.
  Regular,
  /// The actor was pulled along by another actor's choice.
  Warp,
  /// The host forced the actor to move.
  Force,
}

impl MotionKind {
  fn data(self, motion: Motion) -> EventData {
    match self {
      MotionKind::Regular => EventData::RegularMotion(motion),
      MotionKind::Warp => EventData::Warp(motion),
      MotionKind::Force => EventData::ForceMotion(motion),
    }
  }
}

/// Whether an operation could be carried out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SimError {
  /// The conversation, actor, or position named does not exist.
  Missing,
  /// The choice index is out of range, or is not available to this actor.
  NoSuchChoice,
  /// The hub may not appear to this actor, so the move is not allowed.
  NotVisible,
  /// The hub has no choices available, which makes an auto hub an error.
  DeadEnd,
  /// An ask was raised and the move is waiting on the host.
  Suspended,
}

/// The rules that decide who sees what, and how actors move through hubs.
pub struct Simulator;

impl Simulator {
  /// The speakers an actor hears: the ones it speaks as, the ones it listens
  /// to, and the speakers those are listening to.
  pub fn speakers_heard_by(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
  ) -> Vec<LouderName> {
    let Some(entry) = context.get_actor(conversation, actor) else {
      return Vec::new();
    };

    let mut heard: Vec<LouderName> = Vec::new();
    let push = |name: LouderName, heard: &mut Vec<LouderName>| {
      if name != LOUDER_NAME_NONE && !heard.contains(&name) {
        heard.push(name);
      }
    };

    for name in entry.speak_as() {
      push(*name, &mut heard);
    }

    // The speakers an actor speaks as drag their own listening list in with
    // them, which is what makes "speaking as" imply "listening to".
    for name in entry.speak_as() {
      if let Some(speaker) = context.get_speaker(*name) {
        for listening in speaker.listening() {
          push(*listening, &mut heard);
        }
      }
    }

    for name in entry.listen_to() {
      push(*name, &mut heard);
    }

    heard
  }

  /// The speakers an actor listens to directly, without the speakers reachable
  /// through who it speaks as.
  pub fn speakers_listened_to_by(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
  ) -> Vec<LouderName> {
    match context.get_actor(conversation, actor) {
      Some(entry) => entry.listen_to().iter().copied().filter(|n| *n != LOUDER_NAME_NONE).collect(),
      None => Vec::new(),
    }
  }

  /// Whether any actor other than `actor` speaks as `speaker`.
  ///
  /// This is what keeps a speaker from being handed to a second actor: once one
  /// actor is playing a voice, nobody else gets to lead with it. `actor` is
  /// excluded, because an actor always keeps the voice it is already playing.
  pub fn speaker_is_taken_by(
    context: &LouderContext,
    conversation: LouderName,
    speaker: LouderName,
    actor: LouderActor,
  ) -> bool {
    context
      .actors(conversation)
      .iter()
      .filter(|entry| entry.id() != actor)
      .any(|entry| entry.speak_as().contains(&speaker))
  }
}

/// Who a hub belongs to, which is not the same as who is speaking it.
///
/// Ownership answers both questions the engine asks about a hub: whether it
/// appears to an actor, and whether that actor may lead out of it. Keeping one
/// definition of it is what stops the two answers from drifting apart, which is
/// how a hub ends up visible but with no way to move on from it.
///
/// A hub carrying recipients is private to those recipients, so *they* own it
/// and its speaker does not. The speaker may be another character's voice
/// entirely, and someone who owns a hub can lead it without speaking it. Without
/// recipients the speaker owns the hub. With neither, the hub is open and nobody
/// owns it, so it needs no ownership check at all.
#[derive(Clone, Copy)]
enum Owned<'a> {
  /// Neither recipients nor a speaker. Reached by everyone, owned by nobody.
  Open,
  /// Private to these speakers, whoever is speaking it.
  Private(&'a [LouderName]),
  /// Belongs to one speaker, and reaches everyone if that speaker is public.
  Spoken(LouderName),
}

impl<'a> Owned<'a> {
  fn of(hub: &'a Hub) -> Owned<'a> {
    if hub.has_recipients() {
      Owned::Private(hub.recipients())
    } else if hub.speaker() == LOUDER_NAME_NONE {
      Owned::Open
    } else {
      Owned::Spoken(hub.speaker())
    }
  }

  /// Whether a hub with this ownership is reached by an actor hearing `heard`.
  ///
  /// This is the visibility half. `heard` comes from
  /// [`Self::speakers_heard_by`], so it already includes the speakers the actor
  /// speaks as.
  fn reaches(self, context: &LouderContext, heard: &[LouderName]) -> bool {
    match self {
      Owned::Open => true,
      // Private means private: a recipient is reached only by those who hear it,
      // and a public recipient is no more open than a private one.
      Owned::Private(recipients) => recipients.iter().any(|r| heard.contains(r)),
      // A public speaker is heard by everyone, so its hub reaches everyone.
      Owned::Spoken(speaker) => context.is_speaker_public(speaker) || heard.contains(&speaker),
    }
  }

  /// Whether the actor speaks as one of the owners, which is an outright claim
  /// that no other actor can take away.
  fn spoken_by(self, speak_as: &[LouderName]) -> bool {
    match self {
      Owned::Open => false,
      Owned::Private(recipients) => recipients.iter().any(|r| speak_as.contains(r)),
      Owned::Spoken(speaker) => speak_as.contains(&speaker),
    }
  }

  /// Whether some actor other than `actor` already speaks as one of the owners,
  /// so the voice is being played elsewhere and is not this actor's to lead.
  fn held_by_other(self, context: &LouderContext, conversation: LouderName, actor: LouderActor) -> bool {
    let held = |speaker: &LouderName| {
      Simulator::speaker_is_taken_by(context, conversation, *speaker, actor)
    };

    match self {
      Owned::Open => false,
      Owned::Private(recipients) => recipients.iter().any(held),
      Owned::Spoken(speaker) => held(&speaker),
    }
  }
}

impl Simulator {
  /// Whether `hub` may appear to `actor` at all.
  ///
  /// This is the visibility half of the rules: auto hubs never appear, debug
  /// actors see everything, and everyone else is filtered by the hub's owners
  /// and its conditions.
  pub fn hub_appears(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    position: LouderPosition,
  ) -> bool {
    let (hub, is_debug) = {
      let Some(conversation_ref) = context.get_conversation(conversation) else {
        return false;
      };
      let Some(hub) = conversation_ref.hub(position) else {
        return false;
      };
      let Some(entry) = context.get_actor(conversation, actor) else {
        return false;
      };
      (hub.clone(), entry.is_debug())
    };

    // Auto hubs resolve on their own and are never shown.
    if hub.is_auto() {
      return is_debug;
    }

    let heard = Self::speakers_heard_by(context, conversation, actor);
    if !Owned::of(&hub).reaches(context, &heard) {
      return false;
    }

    Self::conditions_hold(context, conversation, hub.expressions())
  }

  /// Whether every condition on a hub holds.
  ///
  /// This is a pure predicate: it neither writes story state nor emits events,
  /// because the FFI answers "what can this actor see" through a `const` handle
  /// and a question must not be able to change the answer. The corresponding
  /// [`Self::report_conditions`] events are emitted when an actor actually
  /// travels to the hub.
  fn conditions_hold(
    context: &LouderContext,
    conversation: LouderName,
    expressions: &[LouderExpression],
  ) -> bool {
    expressions.iter().all(|expression| {
      let Some(source) = context.resolve_name(*expression) else {
        return true;
      };

      // Only the condition half of the expression set gates visibility;
      // statements are for the first visit.
      if super::expression::classify(source) != super::expression::ExpressionKind::Condition {
        return true;
      }

      match super::expression::parse(source) {
        Ok(expr) => super::expression::condition_holds(context, conversation, &expr),
        // A condition that cannot be parsed cannot be relied on.
        Err(_) => false,
      }
    })
  }

  /// Emit an event for each condition evaluated when an actor lands on a hub.
  fn report_conditions(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    channel: LouderName,
    motion: &Motion,
    expressions: &[LouderExpression],
  ) {
    let sources: Vec<(LouderExpression, String)> = expressions
      .iter()
      .filter_map(|expression| {
        context
          .resolve_name(*expression)
          .map(|source| (*expression, source.to_string()))
      })
      .collect();

    let mut scope = EvaluationScope::new(conversation, actor, channel);
    scope.motion = Some(motion.clone());
    let evaluator = Evaluator::new(scope);

    for (expression, source) in sources {
      if super::expression::classify(&source) != super::expression::ExpressionKind::Condition {
        continue;
      }

      let outcome = match evaluator.evaluate(context, &source) {
        Ok(Evaluation::Value(value)) => value,
        Ok(Evaluation::Suspended) => continue,
        Err(_) => LouderVariant::Int(0),
      };

      let condition = EventData::Condition(ConditionOutcome {
        condition: expression,
        outcome,
      });
      let event = LouderEvent {
        conversation,
        channel,
        actor,
        data: condition,
      };
      context.push_event(event);
    }
  }

  /// Whether a single choice is available to `actor` from `hub`.
  ///
  /// The choice rules of the engine guide, in four steps:
  ///  1. the actor can only decide from a hub that actually appears to them;
  ///  2. the choice has to lead somewhere that appears to them;
  ///  3. they have to own the target: speaking as one of its owners settles the
  ///     matter outright, and otherwise listening to an owner, or the owner
  ///     being public, settles it as long as nobody else plays the voice;
  ///  4. otherwise, the spoke rule lets a lone choice through when nobody else
  ///     holds an owning voice, which is what stops linear dialogue handing the
  ///     lead back and forth between every actor.
  ///
  /// Step 3 is what lets a voice heard but not played be led from: a character
  /// listening to a speaker owns that speaker's hubs, and would otherwise be
  /// shown a hub with no way off it wherever the owning hub carries more than
  /// one choice.
  ///
  /// Ownership is [`Owned`], the same definition [`Self::hub_appears`] uses, so
  /// the two can never disagree. A hub an actor is shown but cannot decide from
  /// would dead-end a conversation that no interface could recover.
  pub fn choice_available(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    hub_position: LouderPosition,
    choice_index: usize,
  ) -> bool {
    let (hub, target, is_debug, is_spectator, speak_as) = {
      let Some(conversation_ref) = context.get_conversation(conversation) else {
        return false;
      };
      let Some(hub) = conversation_ref.hub(hub_position) else {
        return false;
      };
      let Some(choice) = hub.choice(choice_index) else {
        return false;
      };
      let Some(entry) = context.get_actor(conversation, actor) else {
        return false;
      };
      let Some(target) = conversation_ref.hub(choice.position()).cloned() else {
        return false;
      };
      (
        hub.clone(),
        target,
        entry.is_debug(),
        entry.is_spectator(),
        entry.speak_as().to_vec(),
      )
    };

    // Debug actors see every choice regardless of the rules.
    if is_debug {
      return true;
    }

    // A spectator sees every hub but is not allowed to decide, so it is only
    // given choices when it is also a debug actor.
    if is_spectator {
      return false;
    }

    // An actor may only decide from a hub that appears to them. Auto hubs are
    // the exception: they are resolved on the way through and never stood on,
    // so an actor passing through one still gets to make the decision there.
    if !hub.is_auto() && !Self::hub_appears(context, conversation, actor, hub_position) {
      return false;
    }

    // A choice can only be taken where it leads somewhere real. An auto hub is
    // the exception: it never appears to anyone, but entering one is a way of
    // moving, and selecting such a choice is how an actor gets into the auto
    // logic that decides where it actually lands.
    let target_is_auto = target.is_auto();
    if !target_is_auto && !Self::hub_appears(context, conversation, actor, target.position()) {
      return false;
    }

    // The target's owners decide whether this actor may lead it. For a private
    // hub those are the recipients, not the speaker, so owning a hub and speaking
    // it come apart: a recipient may lead another character's speech.
    let owners = Owned::of(&target);

    // Speaking as an owner is an outright claim.
    if owners.spoken_by(&speak_as) {
      return true;
    }

    // Someone else is already playing one of the owning voices.
    if owners.held_by_other(context, conversation, actor) {
      return false;
    }

    // Nobody is playing the voice, so it may be led from: by the actor that
    // speaks it, which settled the matter above, or by one that hears it. This
    // is the second and third cases of the guide's choice rules, and it is what
    // carries a choice out of a multi-choice hub rather than only out of a
    // linear one.
    let heard = Self::speakers_heard_by(context, conversation, actor);
    if owners.reaches(context, &heard) {
      return true;
    }

    // The spoke rule: a lone choice may go ahead, since there is nobody else to
    // hand it to and no branch for it to be lost on.
    hub.choice_count() == 1
  }

  /// The indices of the choices available to `actor` from `hub_position`.
  pub fn available_choices(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    hub_position: LouderPosition,
  ) -> Vec<usize> {
    let count = match context.get_conversation(conversation) {
      Some(conversation_ref) => match conversation_ref.hub(hub_position) {
        Some(hub) => hub.choice_count(),
        None => return Vec::new(),
      },
      None => return Vec::new(),
    };

    let mut available = Vec::new();
    for index in 0..count {
      if Self::choice_available(context, conversation, actor, hub_position, index) {
        available.push(index);
      }
    }
    available
  }

  /// Announce an auto hub to the debug actors in the conversation.
  ///
  /// Auto hubs resolve silently for the players, since by definition they never
  /// appear to anyone. A debug actor exists precisely to watch them go by, so an
  /// entered auto hub is announced to those actors and to nobody else. Announced
  /// once per move begun from outside, so a follower emulating the same auto hub
  /// does not announce it a second time.
  fn announce_auto_to_debug(
    context: &mut LouderContext,
    conversation: LouderName,
    auto_hub: LouderPosition,
  ) {
    let watchers: Vec<(LouderActor, LouderPosition)> = context
      .actors(conversation)
      .iter()
      .filter(|entry| entry.is_debug())
      .map(|entry| (entry.id(), entry.position()))
      .collect();

    for (actor, position) in watchers {
      let motion = Motion {
        source_position: position,
        target_position: auto_hub,
        choice_index: -1,
        background: false,
      };
      let event = LouderEvent {
        conversation,
        channel: LOUDER_NAME_NONE,
        actor,
        data: EventData::RegularMotion(motion),
      };
      context.push_event(event);
    }
  }

  /// Move an actor to `target`, emitting every event the move implies.
  ///
  /// This is the entry point for a single move. A move sets off a cascade of
  /// warps, and the cascade is what stops itself from looping: see
  /// [`Self::travel_within`].
  pub fn travel(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    target: LouderPosition,
    kind: MotionKind,
    fake_channel: LouderName,
    fake_priority: i32,
  ) -> Result<LouderPosition, SimError> {
    if let Some(auto_hub) = Self::auto_hub_at(context, conversation, target) {
      Self::announce_auto_to_debug(context, conversation, auto_hub);
    }

    let mut moved = Vec::new();
    Self::travel_within(
      context,
      conversation,
      actor,
      target,
      kind,
      fake_channel,
      fake_priority,
      false,
      &mut moved,
    )
  }

  /// Move an actor and warp whoever it drags along, recording both in `moved`.
  ///
  /// Two actors sharing a channel each look like a follower of the other, so
  /// left unguarded a choice would have the pair trade places forever: the
  /// leader moves, the other follows, and then the leader is a follower of the
  /// one that just followed it. `moved` holds every actor already placed in
  /// this cascade, and an actor already in it does not move again, which also
  /// bounds the recursion to at most one move per actor.
  ///
  /// `emulating` says this move is a follower being carried through a hub, and
  /// is what [`Self::resolve_auto`] consults when an auto hub offers that
  /// follower nothing it may actually take.
  fn travel_within(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    target: LouderPosition,
    kind: MotionKind,
    fake_channel: LouderName,
    fake_priority: i32,
    emulating: bool,
    moved: &mut Vec<LouderActor>,
  ) -> Result<LouderPosition, SimError> {
    // Already placed in this cascade, either as its leader or as someone it
    // pulled along, so the actor stays exactly where it is. This is checked
    // before resolving auto hubs, since a hub the actor is not going to enter
    // is no reason to fail the move.
    if moved.contains(&actor) {
      return context
        .get_actor(conversation, actor)
        .map(|entry| entry.position())
        .ok_or(SimError::Missing);
    }

    let auto_hub = Self::auto_hub_at(context, conversation, target);
    // Whoever was with this actor where it came from is who it drags along, so
    // the channel has to be read before the move changes it.
    let from_channel = context
      .get_actor(conversation, actor)
      .map(|entry| entry.channel())
      .unwrap_or(LOUDER_NAME_NONE);
    let destination = Self::resolve_auto(context, conversation, actor, target, emulating)?;
    moved.push(actor);

    let source = context
      .get_actor(conversation, actor)
      .map(|entry| entry.position())
      .ok_or(SimError::Missing)?;

    let motion = Motion {
      source_position: source,
      target_position: destination,
      choice_index: -1,
      background: false,
    };

    Self::apply_travel(
      context,
      conversation,
      actor,
      motion.clone(),
      kind,
      fake_channel,
      fake_priority,
    )?;

    // Everyone who follows this actor does so after it has settled, so that
    // the events for this move read in a sensible order.
    Self::warp_followers(context, conversation, actor, motion, kind, auto_hub, from_channel, moved)?;
    Ok(destination)
  }

  /// Commit a move: update the actor's state, then announce it.
  fn apply_travel(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    motion: Motion,
    kind: MotionKind,
    fake_channel: LouderName,
    fake_priority: i32,
  ) -> Result<(), SimError> {
    let hub = context
      .get_conversation(conversation)
      .and_then(|c| c.hub(motion.target_position))
      .cloned();

    let Some(hub) = hub else {
      return Err(SimError::Missing);
    };

    let channel = if fake_channel != LOUDER_NAME_NONE {
      fake_channel
    } else if hub.channel() != LOUDER_NAME_NONE {
      hub.channel()
    } else {
      context.main_channel()
    };

    let priority = if fake_priority != 0 {
      fake_priority
    } else {
      hub.priority()
    };

    let channel_before = context
      .get_actor(conversation, actor)
      .map(|entry| entry.channel())
      .unwrap_or(LOUDER_NAME_NONE);

    let flags = context
      .get_actor(conversation, actor)
      .map(|entry| entry.flags())
      .unwrap_or(ActorFlags::empty());

    // Debug and spectator actors are not part of the flow, so they take no
    // channel or priority from the hub they land on.
    let in_flow = !flags.intersects(ActorFlags::SPECTATOR | ActorFlags::DEBUG);

    if let Some(entry) = context.get_actor_mut(conversation, actor) {
      entry.state_mut().set_position(motion.target_position);
      if in_flow {
        if channel != channel_before {
          entry.state_mut().set_channel(channel);
        }
        entry.state_mut().set_priority(priority);
      } else {
        entry.state_mut().clear_channel();
      }
    }

    let channel_now = context
      .get_actor(conversation, actor)
      .map(|entry| entry.channel())
      .unwrap_or(LOUDER_NAME_NONE);

    if in_flow && channel_now != channel_before {
      let event = LouderEvent {
        conversation,
        channel: channel_now,
        actor,
        data: EventData::ChannelChange,
      };
      context.push_event(event);
    }

    // Additional data belongs to the hub, so it replaces the actor's wholesale
    // rather than merging, which stops stale keys accumulating.
    if !hub.additional_data().is_empty() {
      if let Some(entry) = context.get_actor_mut(conversation, actor) {
        let replacement = hub.additional_data().clone();
        entry.state_mut().replace_additional_data(replacement);
      }

      for key in hub.additional_data().sorted_keys() {
        let event = LouderEvent {
          conversation,
          channel: channel_now,
          actor,
          data: EventData::AdditionalDataChange(AdditionalDataChange { key }),
        };
        context.push_event(event);
      }
    }

    let expressions = hub.expressions().to_vec();
    Self::announce(context, conversation, actor, &motion, kind);

    // Conditions are reported on arrival. The visibility decision itself was
    // made by the pure predicate, so this is where the evaluation becomes
    // observable.
    Self::report_conditions(
      context,
      conversation,
      actor,
      channel_now,
      &motion,
      &expressions,
    );

    // Statements and callbacks are one-shot: they run the first time a hub is
    // covered, not every time somebody passes through.
    let already_covered = context
      .get_conversation(conversation)
      .map(|c| c.is_covered(motion.target_position))
      .unwrap_or(true);

    if !already_covered {
      Self::run_first_visit(context, conversation, actor, channel_now, &motion, &hub);
      if let Some(c) = context.get_conversation_mut(conversation) {
        c.mark_covered(motion.target_position);
      }
    }

    Ok(())
  }

  /// Emit the hub appearance and the appearance of each visible choice.
  fn announce(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    motion: &Motion,
    kind: MotionKind,
  ) {
    let channel = context
      .get_actor(conversation, actor)
      .map(|entry| entry.channel())
      .unwrap_or(LOUDER_NAME_NONE);

    let event = LouderEvent {
      conversation,
      channel,
      actor,
      data: kind.data(motion.clone()),
    };
    context.push_event(event);

    for index in Self::available_choices(context, conversation, actor, motion.target_position) {
      let appearance = ChoiceAppearance {
        position: motion.target_position,
        choice_index: index as i32,
      };
      let event = LouderEvent {
        conversation,
        channel,
        actor,
        data: EventData::ChoiceAppearance(appearance),
      };
      context.push_event(event);
    }
  }

  /// Run a hub's statements and callbacks, the first time it is covered.
  fn run_first_visit(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    channel: LouderName,
    motion: &Motion,
    hub: &Hub,
  ) {
    let mut scope = EvaluationScope::new(conversation, actor, channel);
    scope.motion = Some(motion.clone());
    let evaluator = Evaluator::new(scope);

    // The source text is copied out because evaluating it needs mutable access to
    // the same context that holds it.
    let sources: Vec<String> = hub
      .expressions()
      .iter()
      .filter_map(|expression| context.resolve_name(*expression).map(|s| s.to_string()))
      .collect();

    for source in sources {
      // Only statements run on arrival; conditions were already checked to
      // decide whether the hub could appear at all.
      if super::expression::classify(&source) != super::expression::ExpressionKind::Statement {
        continue;
      }

      let _ = evaluator.evaluate(context, &source);
    }

    for callback in hub.callbacks() {
      let event = LouderEvent {
        conversation,
        channel,
        actor,
        data: EventData::Callback(*callback),
      };
      context.push_event(event);
    }
  }

  /// The auto hub at `position`, if that hub is one.
  ///
  /// A move that starts here is resolved by auto logic rather than announced, so
  /// the hub itself is never where an actor comes to rest. It still matters: it
  /// is what a follower is warped through when it emulates the leader.
  fn auto_hub_at(context: &LouderContext, conversation: LouderName, position: LouderPosition) -> Option<LouderPosition> {
    context
      .get_conversation(conversation)
      .and_then(|c| c.hub(position))
      .filter(|hub| hub.is_auto())
      .map(|_| position)
  }

  /// Pull other actors along after `leader` has made a choice.
  ///
  /// An actor follows when it shares a channel with the leader, the hub the
  /// leader landed on could appear to it, and its priority does not exceed the
  /// leader's.
  ///
  /// The shared channel is the one the leader was *leaving*, not the one it
  /// arrived in. A leader that walks into a hub carrying a channel drags along
  /// whoever was with it where it came from, and the follower stays in the old
  /// channel; that divergence is the point of the guide's worked example.
  ///
  /// Where the follower lands depends on whether the leader passed through auto
  /// logic on the way, named by `auto_hub`:
  ///  - If it did, the follower is warped to that same auto hub and resolves it
  ///    as itself, taking the first branch available to *it*. This is what lets
  ///    two actors leaving one hub end up in different places, and it does not
  ///    depend on the auto hub being available to the follower, since appearing
  ///    is not the thing being emulated. Where no branch is available to the
  ///    follower, it takes the first branch that appears to it, and if none
  ///    appears either, it goes where the leader went.
  ///  - Otherwise, when the leader moved by choosing, the follower replays the
  ///    same choice from wherever it was standing.
  ///  - Otherwise the follower simply goes where the leader went.
  fn warp_followers(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    motion: Motion,
    kind: MotionKind,
    auto_hub: Option<LouderPosition>,
    from_channel: LouderName,
    moved: &mut Vec<LouderActor>,
  ) -> Result<(), SimError> {
    let leader_priority = context
      .get_actor(conversation, actor)
      .map(|entry| entry.priority())
      .unwrap_or(0);

    let leader_flags = context
      .get_actor(conversation, actor)
      .map(|entry| entry.flags())
      .unwrap_or(ActorFlags::empty());

    // Actors outside the flow neither follow nor pull others along.
    if leader_flags.intersects(ActorFlags::SPECTATOR | ActorFlags::DEBUG) {
      return Ok(());
    }

    let followers: Vec<(LouderActor, LouderPosition, i32, LouderName)> = context
      .actors(conversation)
      .iter()
      .filter(|entry| entry.id() != actor)
      .filter(|entry| !entry.flags().intersects(ActorFlags::SPECTATOR | ActorFlags::DEBUG))
      .filter(|entry| entry.channel() == from_channel)
      .map(|entry| (entry.id(), entry.position(), entry.priority(), entry.channel()))
      .collect();

    for (follower, from, priority, _) in followers {
      // Anyone this cascade has already placed is where they belong, and
      // following them again is what would otherwise ping-pong between two
      // actors sharing a channel.
      if moved.contains(&follower) {
        continue;
      }

      if priority > leader_priority {
        // Too important to be dragged along, but it still sees the hub in the
        // background, without choices.
        if Self::hub_appears(context, conversation, follower, motion.target_position) {
          let background = Motion {
            source_position: from,
            target_position: motion.target_position,
            choice_index: -1,
            background: true,
          };
          let event = LouderEvent {
            conversation,
            // The follower did not move, so it is still in the channel it shared
            // with the leader.
            channel: from_channel,
            actor: follower,
            data: EventData::RegularMotion(background),
          };
          context.push_event(event);
        }
        continue;
      }

      // Leaving an auto hub is what makes the follower emulate rather than
      // replay: the leader resolved that hub as itself, and the follower gets
      // to resolve it as itself too.
      //
      // What decides whether this follower comes along is its own landing, not
      // the leader's. The leader may have ended up somewhere the follower
      // cannot hear at all, and that is precisely the case where diverging
      // matters, so gating on the leader's landing would skip exactly the
      // followers emulation exists for.
      if let Some(hub) = auto_hub {
        // A follower that cannot be placed is left where it is rather than
        // reported: the leader's move has already been applied, so failing it
        // now would hand the host a refusal for something that happened.
        if Self::resolve_auto(context, conversation, follower, hub, true).is_ok() {
          let _ = Self::travel_within(
            context,
            conversation,
            follower,
            hub,
            MotionKind::Warp,
            LOUDER_NAME_NONE,
            0,
            true,
            moved,
          );
        } else if Self::hub_appears(context, conversation, follower, motion.target_position) {
          // Resolving an auto hub as the follower can still fail, when no branch
          // of it leads anywhere the follower can hear. There is nothing for
          // them to emulate in that case, so they go where the leader went
          // rather than being left behind.
          let _ = Self::travel_within(
            context,
            conversation,
            follower,
            motion.target_position,
            MotionKind::Warp,
            LOUDER_NAME_NONE,
            0,
            true,
            moved,
          );
        }
        continue;
      }

      if !Self::hub_appears(context, conversation, follower, motion.target_position) {
        continue;
      }

      let destination = if kind == MotionKind::Regular && motion.choice_index >= 0 {
        let replay = Self::available_choices(context, conversation, follower, from);
        let leader_choice = motion.choice_index as usize;

        match context
          .get_conversation(conversation)
          .and_then(|c| c.hub(from))
          .and_then(|hub| {
            // Prefer the same choice the leader made; fall back to whatever
            // this actor can actually see from where it stands.
            let index = if replay.contains(&leader_choice) {
              leader_choice
            } else {
              *replay.first()?
            };
            hub.choice(index)
          }) {
          Some(choice) => choice.position(),
          None => motion.target_position,
        }
      } else {
        motion.target_position
      };

      // A follower that cannot be placed is left where it is rather than
      // reported: the leader's move has already been applied, so failing it now
      // would hand the host a refusal for something that happened.
      let _ = Self::travel_within(
        context,
        conversation,
        follower,
        destination,
        MotionKind::Warp,
        LOUDER_NAME_NONE,
        0,
        true,
        moved,
      );
    }

    Ok(())
  }

  /// Select `choice_index` for `actor`, moving it to the linked hub.
  pub fn select_choice(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    choice_index: i32,
  ) -> Result<LouderPosition, SimError> {
    if choice_index < 0 {
      return Err(SimError::NoSuchChoice);
    }

    let position = context
      .get_actor(conversation, actor)
      .map(|entry| entry.position())
      .ok_or(SimError::Missing)?;

    let choice_index = choice_index as usize;
    if !Self::choice_available(context, conversation, actor, position, choice_index) {
      return Err(SimError::NoSuchChoice);
    }

    let destination = context
      .get_conversation(conversation)
      .and_then(|c| c.hub(position))
      .and_then(|hub| hub.choice(choice_index))
      .map(|choice| choice.position())
      .ok_or(SimError::NoSuchChoice)?;

    let motion = Motion {
      source_position: position,
      target_position: destination,
      choice_index: choice_index as i32,
      background: false,
    };

    // Auto hubs are entered but never announced, so resolve them first and
    // report the move as landing on the hub the actor really ends at. The hub
    // itself is remembered, because that is what followers emulate.
    let auto_hub = Self::auto_hub_at(context, conversation, destination);
    let landing = Self::resolve_auto(context, conversation, actor, destination, false)?;
    let resolved = Motion {
      target_position: landing,
      ..motion
    };

    // Whoever shared this actor's channel on the way out is who it leads, so that
    // channel is read before the move changes it rather than after.
    let from_channel = context
      .get_actor(conversation, actor)
      .map(|entry| entry.channel())
      .unwrap_or(LOUDER_NAME_NONE);

    if let Some(auto_hub) = auto_hub {
      Self::announce_auto_to_debug(context, conversation, auto_hub);
    }

    Self::apply_travel(
      context,
      conversation,
      actor,
      resolved.clone(),
      MotionKind::Regular,
      LOUDER_NAME_NONE,
      0,
    )?;

    // The chooser is the leader of this cascade, so it counts as moved before
    // anyone is pulled along.
    let mut moved = vec![actor];
    Self::warp_followers(
      context,
      conversation,
      actor,
      resolved,
      MotionKind::Regular,
      auto_hub,
      from_channel,
      &mut moved,
    )?;
    Ok(landing)
  }

  /// Follow auto hubs from `destination` without emitting anything.
  ///
  /// Auto hubs resolve by taking the first choice available to the actor, so a
  /// document that links two auto hubs to each other would walk that pair
  /// forever. Each hub is therefore walked at most once, and arriving back at
  /// a hub already walked is reported as a dead end, which is what an auto hub
  /// with no way out amounts to.
  ///
  /// `emulating` marks the actor as a follower walking through a hub the leader
  /// left through, which is what the guide calls emulation. A follower is not
  /// deciding anything, so it may have no claim on any branch and still be
  /// entitled to an outcome: when no choice is available to it, the first choice
  /// whose linked hub would appear to it is taken instead. Only then is it a
  /// dead end. An actor that picked its way here gets no such fallback, since a
  /// hub it cannot leave through really is a dead end for it.
  fn resolve_auto(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    destination: LouderPosition,
    emulating: bool,
  ) -> Result<LouderPosition, SimError> {
    let mut current = destination;
    let mut walked: Vec<LouderPosition> = Vec::new();

    loop {
      let is_auto = context
        .get_conversation(conversation)
        .and_then(|c| c.hub(current))
        .map(|hub| hub.is_auto())
        .unwrap_or(false);

      if !is_auto {
        return Ok(current);
      }

      if walked.contains(&current) {
        return Err(SimError::DeadEnd);
      }
      walked.push(current);

      let available = Self::available_choices(context, conversation, actor, current);
      let next = match available.first() {
        Some(next) => *next,
        None => match Self::first_appearing_choice(context, conversation, actor, current) {
          Some(next) if emulating => next,
          _ => return Err(SimError::DeadEnd),
        },
      };

      current = context
        .get_conversation(conversation)
        .and_then(|c| c.hub(current))
        .and_then(|hub| hub.choice(next))
        .map(|choice| choice.position())
        .ok_or(SimError::NoSuchChoice)?;
    }
  }

  /// The first choice on `hub_position` leading somewhere that appears to
  /// `actor`.
  ///
  /// This is the emulation fallback, and the reason it can exist is that a
  /// follower is not deciding anything: it is being carried through the auto hub
  /// the leader passed, and where it lands only has to be somewhere it can
  /// hear. Every branch the leader could take is the leader's to take, so the
  /// follower is left with branches it is shown but not given, and taking the
  /// first of those is what lets the two actors end up apart instead of one of
  /// them being left standing.
  fn first_appearing_choice(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    hub_position: LouderPosition,
  ) -> Option<usize> {
    let count = context
      .get_conversation(conversation)
      .and_then(|c| c.hub(hub_position))
      .map(|hub| hub.choice_count())?;

    (0..count).find(|index| {
      context
        .get_conversation(conversation)
        .and_then(|c| c.hub(hub_position))
        .and_then(|hub| hub.choice(*index))
        .is_some_and(|choice| Self::hub_appears(context, conversation, actor, choice.position()))
    })
  }

  /// Move an actor to `target` without it having to choose.
  ///
  /// `fake_channel` overrides the destination's channel when it is not
  /// [`LOUDER_NAME_NONE`]; `fake_priority` overrides the destination's priority
  /// when it is not 0. Because 0 is the ordinary default priority, a host
  /// cannot force an actor's priority to 0 over a hub that sets one.
  ///
  /// `kind` separates the two forced moves the interface exposes: a warp makes
  /// followers replay a choice and land where the actor landed, while a direct
  /// move simply places the actor and lets followers follow it.
  pub fn force_move(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    target: LouderPosition,
    kind: MotionKind,
    fake_channel: LouderName,
    fake_priority: i32,
  ) -> Result<LouderPosition, SimError> {
    if context.get_conversation(conversation).is_none() {
      return Err(SimError::Missing);
    }

    if context
      .get_conversation(conversation)
      .and_then(|c| c.hub(target))
      .is_none()
    {
      return Err(SimError::Missing);
    }

    Self::travel(
      context,
      conversation,
      actor,
      target,
      kind,
      fake_channel,
      fake_priority,
    )
  }
}

/// A blank actor id, for events that belong to no actor.
pub const NO_ACTOR: LouderActor = LOUDER_ACTOR_NONE;

/// Re-exported so callers can build an [`Actor`] without importing the module.
pub type SimulatorActor = Actor;