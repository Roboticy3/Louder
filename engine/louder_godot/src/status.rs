//! The two codes every engine operation reports.
//!
//! A GDScript caller mostly gets a `bool` back, since an operation either worked
//! or the thing it was asked for was not there. This class is for the cases
//! where the code itself matters.

use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The vocabulary of [`LouderError`](engine::louder::ffi::LouderError), named.
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderStatus {}

godot_api! {
  impl LouderStatus {
    /// The code an operation returns when it worked.
    #[func]
    pub fn ok() -> i32 {
      engine::louder::ffi::LouderError::Ok as i32
    }

    /// The code an operation returns when the thing it was asked for is not
    /// there.
    #[func]
    pub fn missing() -> i32 {
      engine::louder::ffi::LouderError::Missing as i32
    }

    /// What a code means, for a host that would rather read a name than a
    /// number.
    #[func]
    pub fn name(code: i32) -> GString {
      GString::from(match code {
        0 => "OK",
        1 => "MISSING",
        _ => "UNKNOWN",
      })
    }
  }
}