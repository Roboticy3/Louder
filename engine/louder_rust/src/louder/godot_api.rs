//! The macros that turn the engine's structures into Godot classes, and the
//! switch that turns them off.
//!
//! Every data type in [`crate::louder`] is registered with Godot by two macros
//! declared here:
//!
//! - [`godot_class!`] wraps a `struct` declaration, and stands in for the pair
//!   `#[derive(GodotClass)]` and `#[class(init, tool)]` that makes a struct a
//!   class inheriting `RefCounted`.
//! - [`godot_api!`] wraps an `impl` block, and stands in for `#[godot_api]`.
//!   The methods inside it carry `#[func]`, which is what registers them.
//!
//! Both are ordinary `macro_rules` macros, so the feature `godot-bindings`
//! decides what they expand to:
//!
//! - With the feature on, they emit the Godot attributes and the engine is
//!   visible to GDScript as a set of classes.
//! - With the feature off, `godot_class!` emits the bare struct and
//!   `godot_api!` emits nothing at all. The methods it would have registered
//!   are additions for Godot alone, so dropping them loses nothing: the engine
//!   keeps its ordinary Rust API, the C interface in the crate root is the only
//!   way in, and no class is registered.
//!
//! `#[func]` is left in the source either way. `#[godot_api]` consumes it before
//! the compiler ever sees it, and when the feature is off the impl block it
//! belongs to has been removed, so neither path leaves it dangling.
//!
//! # Naming
//!
//! A `godot_api!` block is an inherent `impl` like any other, so a `#[func]`
//! method cannot share a name with a method the engine already has. Most of
//! them are written `get_*` or `set_*`, which the engine's own accessors are
//! not. The few that would collide keep the engine's name in Godot and take a
//! `gd_` prefix in Rust:
//!
//! ```ignore
//! /// Whether this actor sees every hub but decides nothing.
//! #[func(rename = "is_spectator")]
//! pub fn gd_spectator(&self) -> bool {
//!   Actor::is_spectator(self)
//! }
//! ```
//!
//! There is one other rule worth knowing: a class may have only one
//! `#[godot_api]` block, since the second would register itself over the first.
//! Each class here therefore has one, and an `IRefCounted` implementation gets
//! its own.

/// Declares a struct as a Godot class.
///
/// With the `godot-bindings` feature the struct is given the `GodotClass` derive
/// and registered as inheriting `RefCounted`, with a constructor GDScript can
/// call as `Type.new()`. Every field therefore needs a `Default`.
///
/// A type the engine builds itself rather than a host declares
/// `no_init:` and supplies its own constructor through `IRefCounted`, so
/// GDScript cannot default-construct something with no meaningful default:
///
/// ```ignore
/// godot_class! {
///   no_init:
///   /// The engine's own handle.
///   pub struct LouderContext { /* ... */ }
/// }
///
/// godot_api! {
///   impl ::godot::classes::IRefCounted for LouderContext {
///     fn init(_base: ::godot::obj::Base<::godot::classes::RefCounted>) -> Self {
///       Self::new()
///     }
///   }
/// }
/// ```
#[cfg(feature = "godot-bindings")]
macro_rules! godot_class {
  (
    $( #[$attr:meta] )*
    pub struct $name:ident { $( $body:tt )* }
  ) => {
    $( #[$attr] )*
    #[derive(::godot::prelude::GodotClass)]
    #[class(init, tool)]
    pub struct $name { $( $body )* }
  };

  (
    no_init:
    $( #[$attr:meta] )*
    pub struct $name:ident { $( $body:tt )* }
  ) => {
    $( #[$attr] )*
    #[derive(::godot::prelude::GodotClass)]
    #[class(tool)]
    pub struct $name { $( $body )* }
  };
}

/// The same declaration, without the Godot attributes.
///
/// Only the `godot-bindings` half of [`godot_class!`] is repeated here; the
/// struct itself is what the engine needs either way.
#[cfg(not(feature = "godot-bindings"))]
macro_rules! godot_class {
  (
    $( #[$attr:meta] )*
    pub struct $name:ident { $( $body:tt )* }
  ) => {
    $( #[$attr] )*
    pub struct $name { $( $body )* }
  };

  (
    no_init:
    $( #[$attr:meta] )*
    pub struct $name:ident { $( $body:tt )* }
  ) => {
    $( #[$attr] )*
    pub struct $name { $( $body )* }
  };
}

/// Declares an `impl` block as part of a class's Godot API.
///
/// With the `godot-bindings` feature the block gets `#[godot_api]`, which
/// registers every `#[func]` method in it with the class the block's `Self`
/// names. Without the feature the block disappears, because everything in it
/// is written for Godot: the engine's own methods live in ordinary `impl`
/// blocks beside it.
#[cfg(feature = "godot-bindings")]
macro_rules! godot_api {
  ( $( #[$attr:meta] )* impl $( $rest:tt )* ) => {
    $( #[$attr] )*
    #[::godot::prelude::godot_api]
    impl $( $rest )*
  };
}

#[cfg(not(feature = "godot-bindings"))]
macro_rules! godot_api {
  ( $( #[$attr:meta] )* impl $( $rest:tt )* ) => {};
}

/// The Godot types the registered methods are written in, re-exported so that
/// the engine modules can name them without a path in every signature.
#[cfg(feature = "godot-bindings")]
pub(crate) use ::godot::builtin::{Array, GString, PackedByteArray, Variant};
#[cfg(feature = "godot-bindings")]
pub(crate) use ::godot::obj::Gd;