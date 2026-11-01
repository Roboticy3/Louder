//! The rules a host reads back from the engine.
//!
//! Every event, motion, choice, and ask carries one of these, so a host can see
//! which rule produced it. The enum is the complete set named in the engine
//! guide, plus a few the engine adds for its own bookkeeping (`NONE`, `HELD`).

use std::fmt;

/// The rule that decided an event, motion, choice, or ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum LouderRule {
  /// Nothing in particular; host bookkeeping and unsupported operations.
  #[default]
  NONE = 0,

  // -- meta ---------------------------------------------------------------
  COPY,
  EXPERIMENT,

  // -- visibility ---------------------------------------------------------
  PUBLIC_VOICE,
  PRIVATE_HUB,
  VISIBLE_HUB,
  PRIORITY,
  HUB_VOICE,
  VISIBLE_VOICE,
  VISIBILITY_RULE_1,
  VISIBILITY_RULE_2,
  VISIBILITY_RULE_3,
  VISIBILITY_COUNTER_1,
  SPECTATOR,
  SPECTATOR_VISIBILITY,

  // -- choice -------------------------------------------------------------
  CHOICE_HUB,
  CHOICE_TEXT,
  STANDARD,
  SKILL_RULE,
  VISIBILITY_CHECK,
  REMOVAL,
  MEANINGFUL_CHOICE,
  SOFTLOCK,
  SPOKE,
  DEBUG_ACTOR,
  DEBUG,
  AUTO_HUBS,
  AUTO_TRACES,
  AUTO_TRAP,
  STANDARD_CHOOSE,
  AUTO_CHOOSE,
  AUTO_VISIBLE,
  SINGLEPLAYER_RULE_1,
  SINGLEPLAYER_RULE_2,
  WARP,
  NON_WARP,
  FORCE_POSITION,
  HELD,

  // -- evaluation ---------------------------------------------------------
  EVALUATION_RULE,
  STANDARD_EVALUATION,
  EVALUATION_ANDEDNESS,
  EVALUATION_VISIBILITY_COUNTER,
  INTERRUPT,
  PARSE_ERROR,
  LINK_HUBS,
  DEFAULT,
  DOMINATION,
  LINK_RULE,

  // -- event --------------------------------------------------------------
  APPEARANCE,
  STANDARD_APPEARANCE,
  BACKGROUND,
  ASK,
  ANSWER,
  WARP_EVENT,
  DO_NOTHING,
  DO_SOMETHING,

  // -- channel ------------------------------------------------------------
  CHANNEL,
  CHANNEL_CHANGE,
  CHANNEL_CHANGE_WARP,
  CHANNEL_SEPARATION,
}

impl LouderRule {
  /// Build one from its integer value, returning `NONE` for unknown values.
  pub fn from_i32(value: i32) -> Self {
    match value {
      0 => Self::NONE,
      1 => Self::COPY,
      2 => Self::EXPERIMENT,
      3 => Self::PUBLIC_VOICE,
      4 => Self::PRIVATE_HUB,
      5 => Self::VISIBLE_HUB,
      6 => Self::PRIORITY,
      7 => Self::HUB_VOICE,
      8 => Self::VISIBLE_VOICE,
      9 => Self::VISIBILITY_RULE_1,
      10 => Self::VISIBILITY_RULE_2,
      11 => Self::VISIBILITY_RULE_3,
      12 => Self::VISIBILITY_COUNTER_1,
      13 => Self::SPECTATOR,
      14 => Self::SPECTATOR_VISIBILITY,
      15 => Self::CHOICE_HUB,
      16 => Self::CHOICE_TEXT,
      17 => Self::STANDARD,
      18 => Self::SKILL_RULE,
      19 => Self::VISIBILITY_CHECK,
      20 => Self::REMOVAL,
      21 => Self::MEANINGFUL_CHOICE,
      22 => Self::SOFTLOCK,
      23 => Self::SPOKE,
      24 => Self::DEBUG_ACTOR,
      25 => Self::DEBUG,
      26 => Self::AUTO_HUBS,
      27 => Self::AUTO_TRACES,
      28 => Self::AUTO_TRAP,
      29 => Self::STANDARD_CHOOSE,
      30 => Self::AUTO_CHOOSE,
      31 => Self::AUTO_VISIBLE,
      32 => Self::SINGLEPLAYER_RULE_1,
      33 => Self::SINGLEPLAYER_RULE_2,
      34 => Self::WARP,
      35 => Self::NON_WARP,
      36 => Self::FORCE_POSITION,
      37 => Self::HELD,
      38 => Self::EVALUATION_RULE,
      39 => Self::STANDARD_EVALUATION,
      40 => Self::EVALUATION_ANDEDNESS,
      41 => Self::EVALUATION_VISIBILITY_COUNTER,
      42 => Self::INTERRUPT,
      43 => Self::PARSE_ERROR,
      44 => Self::LINK_HUBS,
      45 => Self::DEFAULT,
      46 => Self::DOMINATION,
      47 => Self::LINK_RULE,
      48 => Self::APPEARANCE,
      49 => Self::STANDARD_APPEARANCE,
      50 => Self::BACKGROUND,
      51 => Self::ASK,
      52 => Self::ANSWER,
      53 => Self::WARP_EVENT,
      54 => Self::DO_NOTHING,
      55 => Self::DO_SOMETHING,
      56 => Self::CHANNEL,
      57 => Self::CHANNEL_CHANGE,
      58 => Self::CHANNEL_CHANGE_WARP,
      59 => Self::CHANNEL_SEPARATION,
      _ => Self::NONE,
    }
  }
}

impl fmt::Display for LouderRule {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}", get_rule_string(*self))
  }
}

pub fn get_rule_string(rule: LouderRule) -> String {
  match rule {
    LouderRule::NONE => "none",
    LouderRule::COPY => "copy",
    LouderRule::EXPERIMENT => "experiment",
    LouderRule::PUBLIC_VOICE => "public_voice",
    LouderRule::PRIVATE_HUB => "private_hub",
    LouderRule::VISIBLE_HUB => "visible_hub",
    LouderRule::PRIORITY => "priority",
    LouderRule::HUB_VOICE => "hub_voice",
    LouderRule::VISIBLE_VOICE => "visible_voice",
    LouderRule::VISIBILITY_RULE_1 => "visibility_rule_1",
    LouderRule::VISIBILITY_RULE_2 => "visibility_rule_2",
    LouderRule::VISIBILITY_RULE_3 => "visibility_rule_3",
    LouderRule::VISIBILITY_COUNTER_1 => "visibility_counter_1",
    LouderRule::SPECTATOR => "spectator",
    LouderRule::SPECTATOR_VISIBILITY => "spectator_visibility",
    LouderRule::CHOICE_HUB => "choice_hub",
    LouderRule::CHOICE_TEXT => "choice_text",
    LouderRule::STANDARD => "standard",
    LouderRule::SKILL_RULE => "skill",
    LouderRule::VISIBILITY_CHECK => "invisible",
    LouderRule::REMOVAL => "removal",
    LouderRule::MEANINGFUL_CHOICE => "meaningful_choice",
    LouderRule::SOFTLOCK => "softlock",
    LouderRule::SPOKE => "spoke",
    LouderRule::DEBUG_ACTOR => "debug_actor",
    LouderRule::DEBUG => "debug",
    LouderRule::AUTO_HUBS => "auto_hubs",
    LouderRule::AUTO_TRACES => "auto_traces",
    LouderRule::AUTO_TRAP => "auto_trap",
    LouderRule::STANDARD_CHOOSE => "standard_choose",
    LouderRule::AUTO_CHOOSE => "auto_choose",
    LouderRule::AUTO_VISIBLE => "auto_visible",
    LouderRule::SINGLEPLAYER_RULE_1 => "singleplayer_rule_1",
    LouderRule::SINGLEPLAYER_RULE_2 => "singleplayer_rule_2",
    LouderRule::WARP => "warp",
    LouderRule::NON_WARP => "held",
    LouderRule::FORCE_POSITION => "force_position",
    LouderRule::HELD => "held",
    LouderRule::EVALUATION_RULE => "evaluation_rule",
    LouderRule::STANDARD_EVALUATION => "standard_evaluation",
    LouderRule::EVALUATION_ANDEDNESS => "evaluation_andedness",
    LouderRule::EVALUATION_VISIBILITY_COUNTER => "evaluation_visibility_counter",
    LouderRule::INTERRUPT => "interrupt",
    LouderRule::PARSE_ERROR => "parse_error",
    LouderRule::LINK_HUBS => "link_hubs",
    LouderRule::DEFAULT => "default",
    LouderRule::DOMINATION => "domination",
    LouderRule::LINK_RULE => "link_rule",
    LouderRule::APPEARANCE => "appearance",
    LouderRule::STANDARD_APPEARANCE => "standard_appearance",
    LouderRule::BACKGROUND => "background",
    LouderRule::ASK => "ask",
    LouderRule::ANSWER => "answer",
    LouderRule::WARP_EVENT => "warp_event",
    LouderRule::DO_NOTHING => "do_nothing",
    LouderRule::DO_SOMETHING => "do_something",
    LouderRule::CHANNEL => "channel",
    LouderRule::CHANNEL_CHANGE => "channel_change",
    LouderRule::CHANNEL_CHANGE_WARP => "channel_change_warp",
    LouderRule::CHANNEL_SEPARATION => "channel_separation",
  }
  .to_string()
}