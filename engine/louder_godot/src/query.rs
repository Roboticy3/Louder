//! The questions a host asks about what a conversation holds, as static calls.
//!
//! There is no cursor: every question is asked again with the previous answer as
//! `start`, and the answer walks further along until the blank comes back. Pass
//! [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE) or
//! [`LOUDER_ACTOR_NONE`](engine::LOUDER_ACTOR_NONE) to begin, or
//! `-1` for positions.

use crate::context::LouderContext;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The walks, each taking a context and the position to start from.
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderQuery {}

godot_api! {
  impl LouderQuery {
    /// The speakers `speaker` is listening to, after `start`.
    #[func]
    pub fn next_speaker_listen_to(
      context: Gd<LouderContext>,
      speaker: engine::LouderName,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_speaker_listen_to(&context.bind().inner, speaker, start)
    }

    /// The speakers an actor speaks as, after `start`.
    #[func]
    pub fn next_actor_speak_as(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_actor_speak_as(&context.bind().inner, conversation, actor, start)
    }

    /// The base speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening_base(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_actor_listening_base(&context.bind().inner, conversation, actor, start)
    }

    /// The speakers an actor listens to, after `start`.
    #[func]
    pub fn next_actor_listening(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_actor_listening(&context.bind().inner, conversation, actor, start)
    }

    /// The tags of a hub, after `start`.
    #[func]
    pub fn next_hub_tag(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_hub_tag(&context.bind().inner, conversation, position, start)
    }

    /// The recipients of a hub, after `start`.
    #[func]
    pub fn next_hub_recipient(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      position: engine::LouderPosition,
      start: engine::LouderName,
    ) -> engine::LouderName {
      engine::query::next_hub_recipient(&context.bind().inner, conversation, position, start)
    }

    /// The actors in a channel, after `start`.
    #[func]
    pub fn next_actor_in_channel(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      channel: engine::LouderName,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_in_channel(&context.bind().inner, conversation, channel, start)
    }

    /// The actors in a channel at one position, after `start`.
    #[func]
    pub fn next_actor_in_channel_and_position(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      channel: engine::LouderName,
      position: engine::LouderPosition,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_in_channel_and_position(
        &context.bind().inner,
        conversation,
        channel,
        position,
        start,
      )
    }

    /// The actors speaking as a speaker, after `start`.
    #[func]
    pub fn next_actor_speaking_as(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_speaking_as(&context.bind().inner, conversation, speaker, start)
    }

    /// The actors listening to a speaker, after `start`.
    #[func]
    pub fn next_actor_listening_to(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_listening_to(&context.bind().inner, conversation, speaker, start)
    }

    /// The actors carrying certain flags, after `start`.
    #[func]
    pub fn next_actor_from_flags(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      include: i32,
      exact: bool,
      start: engine::LouderActor,
    ) -> engine::LouderActor {
      engine::query::next_actor_from_flags(
        &context.bind().inner,
        conversation,
        engine::ActorFlags::from_bits_truncate(include),
        i32::from(exact),
        start,
      )
    }

    /// The hubs in a channel, after `start`.
    #[func]
    pub fn next_hub_by_channel(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      channel: engine::LouderName,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_by_channel(&context.bind().inner, conversation, channel, start)
    }

    /// The hubs spoken by a speaker, after `start`.
    #[func]
    pub fn next_hub_spoken_by(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_spoken_by(&context.bind().inner, conversation, speaker, start)
    }

    /// The hubs heard by a speaker, after `start`.
    #[func]
    pub fn next_hub_heard_by(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_heard_by(&context.bind().inner, conversation, speaker, start)
    }

    /// The hubs with a priority within the inclusive bounds, after `start`.
    #[func]
    pub fn next_hub_by_priority(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      priority_min: i32,
      priority_max: i32,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_by_priority(&context.bind().inner, conversation, priority_min, priority_max, start)
    }

    /// The hubs carrying all, or exactly, the given tags, after `start`.
    #[func]
    pub fn next_hub_from_tags(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      include: Array<i64>,
      exact: bool,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      let wanted: Vec<engine::LouderName> = include.iter_shared().map(|tag| tag as engine::LouderName).collect();
      engine::query::next_hub_from_tags(&context.bind().inner, conversation, &wanted, exact, start)
    }

    /// The hubs carrying certain flags, after `start`.
    #[func]
    pub fn next_hub_from_flags(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      include: i32,
      exact: bool,
      start: engine::LouderPosition,
    ) -> engine::LouderPosition {
      engine::query::next_hub_from_flags(
        &context.bind().inner,
        conversation,
        engine::HubFlags::from_bits_truncate(include),
        i32::from(exact),
        start,
      )
    }
  }
}