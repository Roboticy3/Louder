//! The questions a host asks about what a conversation holds.
//!
//! There is no cursor and no iterator to keep: every question is asked again
//! with the previous answer as a starting point, and the answer walks further
//! along. The walking lives here, so that the boundary functions in the crate
//! root are left with nothing but the declaration of what may be asked, and so
//! that one definition of "the next one" serves every list in the engine.
//!
//! A list built from a predicate comes back in ascending order, which is what
//! makes that walk terminate: a host passes the last id it saw and gets either a
//! larger one or the blank that tells it the list is done.

#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, Gd};
use super::actor::{Actor, ActorFlags, LouderActor, LOUDER_ACTOR_NONE};
use super::context::LouderContext;
use super::hub::{Choice, Hub, HubFlags, LouderPosition, BLANK_POSITION};
use super::name::{LouderName, LOUDER_NAME_NONE};
use super::simulator::Simulator;
use super::speaker::SpeakerMap;

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

/// The smallest entry of `items` that is greater than `start`, or `blank`.
///
/// Actors and hub positions are registered as non-negative integers in
/// ascending order, so the iterators can walk a sorted list by comparison rather
/// than by remembering where they got to. The blank is the -1 that stands for
/// "not found" on both of them, and being below every real entry it doubles as
/// "start from the beginning".
fn next_after<T: Copy + Ord>(items: &[T], start: T, blank: T) -> T {
  items
    .iter()
    .copied()
    .find(|item| *item > start)
    .unwrap_or(blank)
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

/// The positions of every hub of `conversation` satisfying `predicate`, in
/// ascending order.
///
/// All six hub iterators are this with a different question, so the question is
/// the only thing each of them has to say for itself. The sort is here rather
/// than in each caller because the hub map has no order of its own, and these
/// functions are documented to walk positions upwards.
fn matching_hubs(
  context: &LouderContext,
  conversation: LouderName,
  predicate: impl Fn(&Hub) -> bool,
) -> Vec<LouderPosition> {
  let Some(conversation_ref) = context.get_conversation(conversation) else {
    return Vec::new();
  };

  let mut positions: Vec<LouderPosition> = conversation_ref
    .hubs()
    .values()
    .filter(|hub| predicate(hub))
    .map(|hub| hub.position())
    .collect();
  positions.sort_unstable();
  positions
}

/// The speakers `speaker` is listening to, one after another.
pub fn next_speaker_listen_to(context: &LouderContext, speaker: LouderName, start: LouderName) -> LouderName {
  match context.get_speaker(speaker) {
    Some(entry) => next_name_after(entry.listening(), start),
    None => LOUDER_NAME_NONE,
  }
}

/// The speakers an actor speaks as, one after another.
pub fn next_actor_speak_as(context: &LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  match context.get_actor(conversation, actor) {
    Some(entry) => next_name_after(entry.speak_as(), start),
    None => LOUDER_NAME_NONE,
  }
}

/// The base speakers an actor listens to, one after another.
///
/// These are the actor's own `listen_to` speakers, without the ones reachable
/// through who it speaks as.
pub fn next_actor_listening_base(context: &LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  match context.get_actor(conversation, actor) {
    Some(entry) => next_name_after(entry.listen_to(), start),
    None => LOUDER_NAME_NONE,
  }
}

/// The speakers an actor listens to, one after another.
///
/// The actor's `listen_to` speakers plus the listening speakers of the speakers
/// it speaks as.
pub fn next_actor_listening(context: &LouderContext, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
  let speakers = Simulator::speakers_heard_by(context, conversation, actor);
  next_name_after(&speakers, start)
}

/// The tags of a hub, one after another.
pub fn next_hub_tag(context: &LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  match context.hub(conversation, position) {
    Some(hub) => next_name_after(hub.tags(), start),
    None => LOUDER_NAME_NONE,
  }
}

/// The recipients of a hub, one after another.
///
/// The walk ends straight away for a hub with no recipients, which is how a hub
/// that is not restricted reads.
pub fn next_hub_recipient(context: &LouderContext, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
  match context.hub(conversation, position) {
    Some(hub) => next_name_after(hub.recipients(), start),
    None => LOUDER_NAME_NONE,
  }
}

/// The actors in a channel.
///
/// Actors outside the flow have no channel and are never returned.
pub fn next_actor_in_channel(context: &LouderContext, conversation: LouderName, channel: LouderName, start: LouderActor) -> LouderActor {
  let actors = matching_actors(context, conversation, |entry| entry.channel() == channel);
  next_after(&actors, start, LOUDER_ACTOR_NONE)
}

/// The actors in a channel at one position.
pub fn next_actor_in_channel_and_position(context: &LouderContext, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
  let actors = matching_actors(context, conversation, |entry| {
    entry.channel() == channel && entry.position() == position
  });
  next_after(&actors, start, LOUDER_ACTOR_NONE)
}

/// The actors speaking as a speaker.
pub fn next_actor_speaking_as(context: &LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  let actors = matching_actors(context, conversation, |entry| entry.speak_as().contains(&speaker));
  next_after(&actors, start, LOUDER_ACTOR_NONE)
}

/// The actors listening to a speaker.
///
/// This covers `listen_to` plus the listening actors derived from the listening
/// speakers of the speakers an actor speaks as.
pub fn next_actor_listening_to(context: &LouderContext, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
  let actors = matching_actors(context, conversation, |entry| {
    Simulator::speakers_heard_by(context, conversation, entry.id()).contains(&speaker)
  });
  next_after(&actors, start, LOUDER_ACTOR_NONE)
}

/// The actors carrying certain flags.
pub fn next_actor_from_flags(context: &LouderContext, conversation: LouderName, include: ActorFlags, exact: i32, start: LouderActor) -> LouderActor {
  let wanted = include.bits();
  let actors = matching_actors(context, conversation, |entry| {
    flags_match(entry.flags().bits(), wanted, exact)
  });
  next_after(&actors, start, LOUDER_ACTOR_NONE)
}

/// The hubs in a channel.
///
/// Hubs that name no channel belong to the default channel, so searching for it
/// also finds them.
pub fn next_hub_by_channel(context: &LouderContext, conversation: LouderName, channel: LouderName, start: LouderPosition) -> LouderPosition {
  let main = context.main_channel();
  let positions = matching_hubs(context, conversation, |hub| match hub.channel() {
    LOUDER_NAME_NONE => main == channel,
    hub_channel => hub_channel == channel,
  });
  next_after(&positions, start, BLANK_POSITION)
}

/// The hubs spoken by a speaker.
pub fn next_hub_spoken_by(context: &LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  let positions = matching_hubs(context, conversation, |hub| hub.speaker() == speaker);
  next_after(&positions, start, BLANK_POSITION)
}

/// The hubs heard by a speaker.
///
/// This searches the speaker's own listening speakers as well as the speaker
/// itself, which are the hubs that may appear to an actor speaking as this
/// speaker.
pub fn next_hub_heard_by(context: &LouderContext, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
  let mut heard = vec![speaker];
  if let Some(entry) = context.get_speaker(speaker) {
    heard.extend(entry.listening().iter().copied());
  }

  let positions = matching_hubs(context, conversation, |hub| heard.contains(&hub.speaker()));
  next_after(&positions, start, BLANK_POSITION)
}

/// The hubs with a priority within the given bounds.
///
/// The bounds are inclusive.
pub fn next_hub_by_priority(context: &LouderContext, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
  let positions = matching_hubs(context, conversation, |hub| {
    hub.priority() >= priority_min && hub.priority() <= priority_max
  });
  next_after(&positions, start, BLANK_POSITION)
}

/// The hubs carrying all, or exactly, the given tags.
///
/// With `exact` set the hub must carry exactly the listed tags; otherwise
/// carrying extra tags is fine.
pub fn next_hub_from_tags(context: &LouderContext, conversation: LouderName, include: &[LouderName], exact: bool, start: LouderPosition) -> LouderPosition {
  let positions = matching_hubs(context, conversation, |hub| {
    let tags = hub.tags();
    include.iter().all(|tag| tags.contains(tag)) && (!exact || tags.len() == include.len())
  });
  next_after(&positions, start, BLANK_POSITION)
}

/// The hubs carrying certain flags.
pub fn next_hub_from_flags(context: &LouderContext, conversation: LouderName, include: HubFlags, exact: i32, start: LouderPosition) -> LouderPosition {
  let wanted = include.bits();
  let positions = matching_hubs(context, conversation, |hub| {
    flags_match(hub.flags().bits(), wanted, exact)
  });
  next_after(&positions, start, BLANK_POSITION)
}

/// The choice on a hub that `actor` is allowed to take, if there is one.
///
/// The engine's rules decide whether a choice can be taken, so a lookup through
/// this answers "may this actor choose that" and not merely "is there a choice
/// there". Both the choice text and the hub it leads to are read through here,
/// which is what keeps a host from being shown a choice the engine would refuse.
pub fn available_choice(
  context: &LouderContext,
  conversation: LouderName,
  position: LouderPosition,
  actor: LouderActor,
  choice_index: usize,
) -> Option<&Choice> {
  if !Simulator::choice_available(context, conversation, actor, position, choice_index) {
    return None;
  }

  context
    .hub(conversation, position)
    .and_then(|hub| hub.choice(choice_index))
}

/// The questions a host asks about what a conversation holds.
///
/// There is no cursor: every question is asked again with the previous answer
/// as `start`, and the answer walks further along until the blank comes back.
/// The walks themselves are the free functions beside this class; what a
/// GDScript caller sees is the same walk with the blank spelled
/// [`LOUDER_NAME_NONE`] or [`LOUDER_ACTOR_NONE`].
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
pub struct LouderQuery {}

godot_api! {
  impl LouderQuery {
    /// The speakers `speaker` is listening to, after `start`.
    #[func]
    pub fn next_speaker_listen_to(context: Gd<LouderContext>, speaker: LouderName, start: LouderName) -> LouderName {
      let context = context.bind();
      next_speaker_listen_to(&context, speaker, start)
    }

    /// The speakers an actor speaks as, after `start`.
    #[func]
    pub fn next_actor_speak_as(context: Gd<LouderContext>, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
      let context = context.bind();
      next_actor_speak_as(&context, conversation, actor, start)
    }

    /// The base speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening_base(context: Gd<LouderContext>, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
      let context = context.bind();
      next_actor_listening_base(&context, conversation, actor, start)
    }

    /// The speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening(context: Gd<LouderContext>, conversation: LouderName, actor: LouderActor, start: LouderName) -> LouderName {
      let context = context.bind();
      next_actor_listening(&context, conversation, actor, start)
    }

    /// The tags of a hub, after `start`.
    #[func]
    pub fn next_hub_tag(context: Gd<LouderContext>, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
      let context = context.bind();
      next_hub_tag(&context, conversation, position, start)
    }

    /// The recipients of a hub, after `start`.
    #[func]
    pub fn next_hub_recipient(context: Gd<LouderContext>, conversation: LouderName, position: LouderPosition, start: LouderName) -> LouderName {
      let context = context.bind();
      next_hub_recipient(&context, conversation, position, start)
    }

    /// The actors in a channel, after `start`.
    #[func]
    pub fn next_actor_in_channel(context: Gd<LouderContext>, conversation: LouderName, channel: LouderName, start: LouderActor) -> LouderActor {
      let context = context.bind();
      next_actor_in_channel(&context, conversation, channel, start)
    }

    /// The actors in a channel at one position, after `start`.
    #[func]
    pub fn next_actor_in_channel_and_position(context: Gd<LouderContext>, conversation: LouderName, channel: LouderName, position: LouderPosition, start: LouderActor) -> LouderActor {
      let context = context.bind();
      next_actor_in_channel_and_position(&context, conversation, channel, position, start)
    }

    /// The actors speaking as a speaker, after `start`.
    #[func]
    pub fn next_actor_speaking_as(context: Gd<LouderContext>, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
      let context = context.bind();
      next_actor_speaking_as(&context, conversation, speaker, start)
    }

    /// The actors listening to a speaker, after `start`.
    #[func]
    pub fn next_actor_listening_to(context: Gd<LouderContext>, conversation: LouderName, speaker: LouderName, start: LouderActor) -> LouderActor {
      let context = context.bind();
      next_actor_listening_to(&context, conversation, speaker, start)
    }

    /// The actors carrying certain flags, after `start`.
    #[func]
    pub fn next_actor_from_flags(context: Gd<LouderContext>, conversation: LouderName, include: i32, exact: bool, start: LouderActor) -> LouderActor {
      let context = context.bind();
      next_actor_from_flags(&context, conversation, ActorFlags::from_bits_truncate(include), i32::from(exact), start)
    }

    /// The hubs in a channel, after `start`.
    #[func]
    pub fn next_hub_by_channel(context: Gd<LouderContext>, conversation: LouderName, channel: LouderName, start: LouderPosition) -> LouderPosition {
      let context = context.bind();
      next_hub_by_channel(&context, conversation, channel, start)
    }

    /// The hubs spoken by a speaker, after `start`.
    #[func]
    pub fn next_hub_spoken_by(context: Gd<LouderContext>, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
      let context = context.bind();
      next_hub_spoken_by(&context, conversation, speaker, start)
    }

    /// The hubs heard by a speaker, after `start`.
    #[func]
    pub fn next_hub_heard_by(context: Gd<LouderContext>, conversation: LouderName, speaker: LouderName, start: LouderPosition) -> LouderPosition {
      let context = context.bind();
      next_hub_heard_by(&context, conversation, speaker, start)
    }

    /// The hubs with a priority within the bounds, after `start`.
    ///
    /// The bounds are inclusive.
    #[func]
    pub fn next_hub_by_priority(context: Gd<LouderContext>, conversation: LouderName, priority_min: i32, priority_max: i32, start: LouderPosition) -> LouderPosition {
      let context = context.bind();
      next_hub_by_priority(&context, conversation, priority_min, priority_max, start)
    }

    /// The hubs carrying all, or exactly, the given tags, after `start`.
    #[func]
    pub fn next_hub_from_tags(context: Gd<LouderContext>, conversation: LouderName, include: Array<i64>, exact: bool, start: LouderPosition) -> LouderPosition {
      let context = context.bind();
      let wanted: Vec<LouderName> = include.iter_shared().map(|tag| tag as LouderName).collect();
      next_hub_from_tags(&context, conversation, &wanted, exact, start)
    }

    /// The hubs carrying certain flags, after `start`.
    #[func]
    pub fn next_hub_from_flags(context: Gd<LouderContext>, conversation: LouderName, include: i32, exact: bool, start: LouderPosition) -> LouderPosition {
      let context = context.bind();
      next_hub_from_flags(&context, conversation, HubFlags::from_bits_truncate(include), i32::from(exact), start)
    }

    /// The choice on a hub that `actor` is allowed to take, if there is one.
    ///
    /// This answers "may this actor choose that", not merely "is there a choice
    /// there", so a host is never shown a choice the engine would refuse.
    #[func]
    pub fn available_choice(context: Gd<LouderContext>, conversation: LouderName, position: LouderPosition, actor: LouderActor, choice_index: i32) -> Option<Gd<Choice>> {
      if choice_index < 0 {
        return None;
      }
      let context = context.bind();
      available_choice(&context, conversation, position, actor, choice_index as usize)
        .map(|choice| Gd::from_object(choice.clone()))
    }
  }
}