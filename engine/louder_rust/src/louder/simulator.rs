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

  /// Whether a hub's speaker lets it reach `actor`.
  ///
  /// An absent speaker and a public or undefined speaker both reach everyone;
  /// otherwise the actor has to speak as or listen to that speaker.
  pub fn speaker_reaches(context: &LouderContext, speaker: LouderName, actor: LouderActor, conversation: LouderName) -> bool {
    if speaker == LOUDER_NAME_NONE || context.is_speaker_public(speaker) {
      return true;
    }

    Self::speakers_heard_by(context, conversation, actor).contains(&speaker)
  }

  /// Whether any other actor in the conversation speaks as `speaker`.
  ///
  /// This is what keeps a speaker from being handed to a second actor: once one
  /// actor is playing a voice, nobody else gets to lead with it.
  pub fn speaker_is_taken(context: &LouderContext, conversation: LouderName, speaker: LouderName) -> bool {
    context
      .actors(conversation)
      .iter()
      .any(|entry| entry.speak_as().contains(&speaker))
  }

  /// Whether `hub` may appear to `actor` at all.
  ///
  /// This is the visibility half of the rules: auto hubs never appear, debug
  /// actors see everything, and everyone else is filtered by recipients,
  /// speaker, and the hub's conditions.
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

    // Recipients override the normal rules: the hub is for exactly them.
    if hub.has_recipients() {
      return hub.recipients().iter().any(|r| heard.contains(r));
    }

    if hub.speaker() != LOUDER_NAME_NONE && !Self::speaker_reaches(context, hub.speaker(), actor, conversation) {
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
  /// This is the spoke rule of the engine guide: the linked hub must appear to
  /// the actor, and the speaker situation has to let this actor be the one
  /// deciding. The spoke rule additionally allows a lone choice on a hub when
  /// the actor has any claim to the speaker at all, which is what stops linear
  /// dialogue from handing the lead back and forth between every actor.
  ///
  /// A claim is measured against [`Self::speakers_heard_by`], the same set
  /// [`Self::hub_appears`] uses, so that an actor is never shown a hub it has no
  /// way to move on from. An actor that hears a speaker only through the
  /// speaker it plays can therefore be led by that voice.
  pub fn choice_available(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    hub_position: LouderPosition,
    choice_index: usize,
  ) -> bool {
    let (hub, target, is_debug, is_spectator, speak_as_list) = {
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

    // A choice can only be taken where it leads somewhere real.
    if !Self::hub_appears(context, conversation, actor, target.position()) {
      return false;
    }

    let speaker = target.speaker();
    // The speak-as list still matters on its own: an actor leading as a speaker
    // is never subject to the spoke rule.
    let speak_as = speak_as_list.contains(&speaker);
    let taken = Self::speaker_is_taken(context, conversation, speaker);
    let is_public = speaker == LOUDER_NAME_NONE || context.is_speaker_public(speaker);

    // Listening reaches further than the actor's own list. Speaking as a speaker
    // drags in whatever that speaker is listening to, so an actor can be led by
    // a voice it only hears through the one it plays. This is the same heard set
    // that decides whether the hub appears at all, and the two must agree: a hub
    // an actor can see but no choice can reach would dead-end the conversation.
    let heard = Self::speakers_heard_by(context, conversation, actor);
    let listen_to = heard.contains(&speaker);

    // Recipients stand in for the speaker when deciding who may decide.
    let claim = if target.has_recipients() {
      target.recipients().iter().any(|r| heard.contains(r))
    } else {
      speak_as || listen_to || is_public
    };

    if !claim {
      return false;
    }

    if speak_as {
      return true;
    }

    if taken {
      return false;
    }

    // The spoke rule: the only choice on a hub may go ahead if the actor is
    // otherwise entitled to the speaker.
    if hub.choice_count() == 1 && (listen_to || is_public) {
      return true;
    }

    false
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
    let mut moved = Vec::new();
    Self::travel_within(
      context,
      conversation,
      actor,
      target,
      kind,
      fake_channel,
      fake_priority,
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
  fn travel_within(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    target: LouderPosition,
    kind: MotionKind,
    fake_channel: LouderName,
    fake_priority: i32,
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

    let destination = Self::resolve_auto(context, conversation, actor, target)?;
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
    Self::warp_followers(context, conversation, actor, motion, kind, moved)?;
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

  /// Pull other actors along after `leader` has made a choice.
  ///
  /// An actor follows when it shares a channel with the leader, the hub the
  /// leader landed on could appear to it, and its priority does not exceed the
  /// leader's. When the leader moved by choosing, the follower replays the same
  /// choice from wherever it was standing, which is what lets auto hubs send
  /// the two actors down different paths.
  fn warp_followers(
    context: &mut LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    motion: Motion,
    kind: MotionKind,
    moved: &mut Vec<LouderActor>,
  ) -> Result<(), SimError> {
    let leader_channel = context
      .get_actor(conversation, actor)
      .map(|entry| entry.channel())
      .unwrap_or(LOUDER_NAME_NONE);

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
      .filter(|entry| entry.channel() == leader_channel)
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
            channel: leader_channel,
            actor: follower,
            data: EventData::RegularMotion(background),
          };
          context.push_event(event);
        }
        continue;
      }

      if !Self::hub_appears(context, conversation, follower, motion.target_position) {
        continue;
      }

      // An emulation warp replays the leader's choice from the follower's own
      // position, so that auto hubs can diverge. A follower with no choice of
      // its own to replay simply goes where the leader went.
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

      // The replayed choice may itself lead through auto hubs.
      let _ = Self::travel_within(
        context,
        conversation,
        follower,
        destination,
        MotionKind::Warp,
        LOUDER_NAME_NONE,
        0,
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
    // report the move as landing on the hub the actor really ends at.
    let landing = Self::resolve_auto(context, conversation, actor, destination)?;
    let resolved = Motion {
      target_position: landing,
      ..motion
    };

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
  fn resolve_auto(
    context: &LouderContext,
    conversation: LouderName,
    actor: LouderActor,
    destination: LouderPosition,
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
      let Some(next) = available.first() else {
        return Err(SimError::DeadEnd);
      };

      current = context
        .get_conversation(conversation)
        .and_then(|c| c.hub(current))
        .and_then(|hub| hub.choice(*next))
        .map(|choice| choice.position())
        .ok_or(SimError::NoSuchChoice)?;
    }
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