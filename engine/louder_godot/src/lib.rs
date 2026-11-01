//! The Godot adapter.
//!
//! This crate is the engine as Godot sees it: one GDExtension, a set of
//! classes, and the methods GDScript calls on them. None of the logic lives
//! here. Everything below is a wrapper around `louder_rust`, which is the engine
//! and knows nothing about Godot, so a host written in some other language is a
//! second wrapper over the same types rather than a fork of them.
//!
//! # The wrapper classes
//!
//! Every class here holds an engine value in an `inner` field and offers it
//! through methods. Two kinds of value are involved:
//!
//! - A [`context::LouderContext`] is the engine's own handle to a running
//!   dialogue, and every other class is read out of it. It is the only class a
//!   host holds on to; everything else is a snapshot of what the engine held at
//!   the moment it was asked for.
//! - [`hub::Hub`], [`actor::Actor`], [`speaker::Speaker`],
//!   [`conversation::Conversation`] and the rest are those snapshots. Writing to
//!   one of them changes only that copy, which is why anything that moves an
//!   actor or changes a conversation goes through the context instead.
//!
//! An engine value could not be declared as a Godot class directly, because Rust
//! only allows an `impl` block inside the crate that defines the type it is for.
//! The wrappers are what make the split possible: each one is defined here, so
//! each one can carry an `impl` block here.
//!
//! # Naming
//!
//! A method is registered under the name of the Rust function it came from, and
//! the Rust function is named what GDScript should call it. There is no
//! `#[func(rename = "...")]` anywhere in this crate: godot-rust reads that value
//! as a token stream, so a string literal arrives with its own quotes still on
//! and the method ends up registered under `"add_actor"`, which only `call` can
//! reach. Naming the function is the same thing said without the trap.

use godot::prelude::*;

/// The engine, under the name this crate gives it.
///
/// Every module reaches the engine through here rather than naming the
/// `louder_rust` path itself, so that the relationship between a class and the
/// engine type it wraps is stated once and in one place.
pub(crate) mod engine {
  pub use louder_rust::louder;
  pub use louder_rust::louder::actor::{Actor, ActorFlags, ActorState, LouderActor, LOUDER_ACTOR_NONE};
  pub use louder_rust::louder::callback::LouderCallback;
  pub use louder_rust::louder::context::LouderContext;
  pub use louder_rust::louder::conversation::{Conversation, VariableScope, classify_variable};
  pub use louder_rust::louder::event::{
    AdditionalDataChange,
    ChoiceAppearance,
    ConditionOutcome,
    EventData,
    Interrupt,
    LoadFailure,
    LouderAsk,
    LouderEvent,
    Motion,
  };
  pub use louder_rust::louder::expression::{
    Evaluation,
    EvaluationScope,
    Evaluator,
    ExpressionKind,
    LouderExpression,
    ParseError,
    classify,
    condition_holds,
    parse,
    reads_only,
    tokenize,
  };
  pub use louder_rust::louder::hub::{Choice, Hub, HubFlags, LouderPosition};
  pub use louder_rust::louder::json;
  pub use louder_rust::louder::json_loader::{
    LOUDER_LOAD_ALREADY_LOADED,
    LOUDER_LOAD_MALFORMED_JSON,
    LOUDER_LOAD_NOT_A_LIST,
    LOUDER_LOAD_NOT_AN_OBJECT,
    LOUDER_LOAD_OK,
    load_config,
    load_conversation,
  };
  pub use louder_rust::louder::name::{
    LOUDER_NAME_NONE,
    LouderContextResult,
    LouderInternContext,
    LouderName,
    LouderNameContent,
    LouderNameContext,
    split_inline,
    split_lines,
  };
  pub use louder_rust::louder::object::{HomogeneousObject, LouderVariant};
  pub use louder_rust::louder::query;
  pub use louder_rust::louder::rule::{LouderRule, get_rule_string};
  pub use louder_rust::louder::rules::DialogueRules;
  pub use louder_rust::louder::simulator::{MotionKind, Simulator};
  pub use louder_rust::louder::speaker::{Speaker, SpeakerFlags, SpeakerMap};
}

/// The classes, gathered for the modules that need to name one another's types.
/// The classes, one module each, matching the module of the engine type they
/// wrap.
pub mod actor;
pub mod context;
pub mod conversation;
pub mod event;
pub mod expression;
pub mod hub;
pub mod json;
pub mod loader;
pub mod name;
pub mod object;
pub mod query;
pub mod rules;
pub mod simulator;
pub mod speaker;
pub mod status;

/// How a [`LouderVariant`](engine::LouderVariant) crosses into Godot.
///
/// Not a class of its own: the conversion is a free function rather than a
/// method, because a variant belongs to the engine and Rust does not allow an
/// `impl` block for a type outside the crate that defines it.
pub mod variant;

// Generated wrappers (placeholder).
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

/// The entry point Godot loads.
///
/// Every `#[derive(GodotClass)]` below registers itself through the shard
/// registry as this library is initialised, so nothing has to be listed here.
/// This is what makes those registrations happen, and what makes the classes and
/// their methods visible to Godot once it has them.
pub struct LouderGdextension;

#[gdextension]
unsafe impl ExtensionLibrary for LouderGdextension {}