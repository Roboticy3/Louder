//! The macro an adapter crate uses to turn its `impl` blocks into Godot
//! classes, and the feature that turns it off.
//!
//! The engine declares no Godot classes. What it does instead is hand an adapter
//! the one piece of syntax that makes declaring them possible:
//! [`louder_rust::godot_api!`](macro@crate::godot_api). An adapter writes
//! ordinary inherent `impl` blocks inside it, and the macro puts `#[godot_api]`
//! on each one, which registers every `#[func]` method in it with the class the
//! block's `Self` names.
//!
//! ```
//! # #[cfg(feature = "godot-bindings")]
//! louder_rust::godot_api! {
//!   impl Hub {
//!     #[func]
//!     pub fn get_speaker(&self) -> LouderName {
//!       self.inner.speaker()
//!     }
//!   }
//! }
//! ```
//!
//! # Why the classes are not a macro too
//!
//! The obvious shape is a macro on each `struct`, wrapping the declaration and
//! standing in for `#[derive(GodotClass)] #[class(init, tool)]`. That does not
//! work here, because a class has to be declared as a plain `struct` with
//! attributes the compiler and the derive macro can both see. Writing the two
//! attributes out is therefore not a workaround but the only way to say it:
//!
//! ```ignore
//! #[derive(Clone)]
//! #[derive(::godot::prelude::GodotClass)]
//! #[class(init, tool)]
//! pub struct Hub {
//!   inner: louder_rust::louder::hub::Hub,
//! }
//! ```
//!
//! # Naming
//!
//! A registered method is named after the Rust function it came from, so
//! `pub fn get_speaker` is `get_speaker` in GDScript and nothing else is needed
//! to say so. An adapter never writes `#[func(rename = "...")]`: godot-rust
//! reads that value as a token stream, so a string literal arrives carrying its
//! own quotes and the method ends up registered under `"add_actor"`, reachable
//! only through `call`. An adapter that needs a method to read differently in
//! GDScript names the Rust function that way instead.
//!
//! `#[func]` is left in the source either way. `#[godot_api]` consumes it before
//! the compiler ever sees it, so it never reaches the build as a dangling
//! attribute.
//!
//! # One block per class
//!
//! A class may have only one `#[godot_api]` block, since a second would register
//! itself over the first. An `IRefCounted` implementation, which is how a class
//! with a constructor Godot should be able to call gets one, gets its own.

/// Declares an `impl` block as part of a class's Godot API.
///
/// Every `#[func]` method in the block is registered with the class the block's
/// `Self` names, which is how an adapter exposes the engine to GDScript.
#[cfg(feature = "godot-bindings")]
#[macro_export]
macro_rules! godot_api {
  ( $( #[$attr:meta] )* impl $( $rest:tt )* ) => {
    $( #[$attr] )*
    #[::godot::prelude::godot_api]
    impl $( $rest )*
  };
}