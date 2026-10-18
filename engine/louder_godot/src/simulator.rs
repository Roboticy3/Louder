//! The rules that decide who sees what, and how actors move through hubs.
//!
//! The simulator holds no state of its own and works entirely on a
//! [`LouderContext`], so every method here is a static one taking one. It
//! answers what an actor can see without changing anything, and moves actors by
//! writing to the context.

use crate::context::LouderContext;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// What an actor may see and where it may go.
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Simulator {}

godot_api! {
  impl Simulator {
    /// Whether `hub_position` may appear to `actor` at all.
    ///
    /// Auto hubs never appear, debug actors see everything, and everyone else is
    /// filtered by the hub's owners and its conditions.
    #[func]
    pub fn hub_appears(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      hub_position: engine::LouderPosition,
    ) -> bool {
      engine::Simulator::hub_appears(&context.bind().inner, conversation, actor, hub_position)
    }

    /// Whether a single choice is available to `actor` from `hub_position`.
    ///
    /// This is "may this actor choose that", not merely "is there a choice
    /// there": the engine's own rules decide.
    #[func]
    pub fn choice_available(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      hub_position: engine::LouderPosition,
      choice_index: i32,
    ) -> bool {
      if choice_index < 0 {
        return false;
      }
      engine::Simulator::choice_available(
        &context.bind().inner,
        conversation,
        actor,
        hub_position,
        choice_index as usize,
      )
    }

    /// The indices of the choices available to `actor` from `hub_position`.
    #[func]
    pub fn available_choices(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      hub_position: engine::LouderPosition,
    ) -> Array<i64> {
      engine::Simulator::available_choices(&context.bind().inner, conversation, actor, hub_position)
        .into_iter()
        .map(|index| index as i64)
        .collect()
    }

    /// The speakers an actor hears: the ones it speaks as, the ones it listens
    /// to, and the speakers those are listening to.
    #[func]
    pub fn speakers_heard_by(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
    ) -> Array<i64> {
      engine::Simulator::speakers_heard_by(&context.bind().inner, conversation, actor)
        .into_iter()
        .map(|name| name as i64)
        .collect()
    }

    /// The speakers an actor listens to directly, without the speakers reachable
    /// through who it speaks as.
    #[func]
    pub fn speakers_listened_to_by(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
    ) -> Array<i64> {
      engine::Simulator::speakers_listened_to_by(&context.bind().inner, conversation, actor)
        .into_iter()
        .map(|name| name as i64)
        .collect()
    }

    /// Whether any actor other than `actor` speaks as `speaker`.
    ///
    /// This is what keeps a speaker from being handed to a second actor.
    #[func]
    pub fn speaker_is_taken_by(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      speaker: engine::LouderName,
      actor: engine::LouderActor,
    ) -> bool {
      engine::Simulator::speaker_is_taken_by(&context.bind().inner, conversation, speaker, actor)
    }

    /// Select `choice_index` for `actor`, moving it to the linked hub.
    ///
    /// Returns true on success. Auto hubs along the way are resolved by taking
    /// the first choice available at each one.
    #[func]
    pub fn select_choice(
      mut context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      choice_index: i32,
    ) -> bool {
      engine::Simulator::select_choice(&mut context.bind_mut().inner, conversation, actor, choice_index).is_ok()
    }

    /// Move an actor to `target` as if it had moved through the network.
    ///
    /// Other actors sharing the actor's channel may be pulled along, exactly as
    /// if the actor had chosen. `fake_channel` and `fake_priority` override the
    /// destination's own when they are not the blanks. Returns true on success.
    #[func]
    pub fn force_warp(
      mut context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      target_position: engine::LouderPosition,
      fake_channel: engine::LouderName,
      fake_priority: i32,
    ) -> bool {
      engine::Simulator::force_move(
        &mut context.bind_mut().inner,
        conversation,
        actor,
        target_position,
        engine::MotionKind::Warp,
        fake_channel,
        fake_priority,
      )
      .is_ok()
    }

    /// Move an actor to `target` directly, letting followers follow it.
    ///
    /// Returns true on success.
    #[func]
    pub fn force_position(
      mut context: Gd<LouderContext>,
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      target_position: engine::LouderPosition,
      fake_channel: engine::LouderName,
      fake_priority: i32,
    ) -> bool {
      engine::Simulator::force_move(
        &mut context.bind_mut().inner,
        conversation,
        actor,
        target_position,
        engine::MotionKind::Force,
        fake_channel,
        fake_priority,
      )
      .is_ok()
    }
  }
}