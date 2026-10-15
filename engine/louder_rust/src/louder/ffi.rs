//! The vocabulary of the C boundary.
//!
//! Everything here exists for the host rather than for the engine: how an
//! operation reports that the thing it was asked for is not there, how an
//! optional value is handed back, and how bytes cross between a caller's buffer
//! and a Rust value. The engine itself never picks an error code or writes into
//! memory it does not own, so all of that is kept together in one place and the
//! entry points in the crate root are left with nothing to decide.
//!
//! None of it is reachable from GDScript. The three buffer helpers below exist
//! because a C caller owns its own memory and the engine does not; Godot hands
//! over a [`String`] that is already copied, so the
//! corresponding methods on [`LouderContext`](super::context::LouderContext)
//! take and return strings directly and never need to ask these questions.
//! [`LouderError`] is the one piece of vocabulary that survives, and
//! [`LouderStatus`] is the class that names its codes.

use super::name::{LouderName, LouderNameContent};

/// An error code returned by most operations.
///
/// - [`LouderError::Ok`]: no error.
/// - [`LouderError::Missing`]: the requested object does not exist.
#[repr(usize)]
pub enum LouderError {
  Ok = 0,
  Missing,
}

/// Custom Option type exposed to the C FFI
#[repr(C)]
pub enum LouderOption<T> {
  None,
  Some(T)
}

/// The status codes an engine operation reports.
///
/// A Godot caller mostly gets a `bool` back, since an operation either worked
/// or the thing it was asked for was not there. This class is for the cases
/// where the code itself matters, such as a load failure, which has a family
/// of them; see [`LouderLoader`](super::json_loader::LouderLoader).
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
pub struct LouderStatus {}

godot_api! {
  impl LouderStatus {
    /// The code an operation returns when it worked.
    #[func]
    pub fn ok() -> i32 {
      LouderError::Ok as i32
    }

    /// The code an operation returns when the thing it was asked for is not
    /// there.
    #[func]
    pub fn missing() -> i32 {
      LouderError::Missing as i32
    }

    /// What a code means, for a host that would rather read a name than a
    /// number.
    #[func]
    pub fn name(code: i32) -> super::godot_api::GString {
      super::godot_api::GString::from(match code {
        0 => "OK",
        1 => "MISSING",
        _ => "UNKNOWN",
      })
    }
  }
}

/// Copy `text` into `buffer`, truncating if needed, and null terminate.
///
/// Returns the number of bytes written, excluding the terminator.
pub(crate) fn write_text(buffer: *mut u8, max_size: usize, text: &str) -> usize {
  if buffer.is_null() {
    return 0;
  }

  let bytes = text.as_bytes();
  let length = bytes.len().min(max_size);

  unsafe {
    if length > 0 {
      std::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer, length);
    }
    *buffer.add(length) = 0;
  }

  length
}

/// Copy the `max_size` bytes at `buffer` into a name.
///
/// `None` if the pointer is null, which is how a caller that has nothing to say
/// is told apart from one that said an empty name.
pub(crate) fn read_name_content(buffer: *const u8, max_size: usize) -> Option<LouderNameContent> {
  if buffer.is_null() {
    return None;
  }

  Some(LouderNameContent {
    bytes: unsafe { std::slice::from_raw_parts(buffer, max_size).to_vec() },
  })
}

/// Copy the `count` names at `names` into a list.
///
/// `None` if the arguments do not describe a list: a negative count is refused
/// outright rather than read as an enormous one, and so is a null pointer
/// claiming to carry entries. A zero count is an empty list either way, since a
/// host with nothing to pass may legitimately pass nothing at all.
pub(crate) fn read_name_list(names: *const LouderName, count: i32) -> Option<Vec<LouderName>> {
  if count < 0 || (names.is_null() && count > 0) {
    return None;
  }

  if count == 0 {
    return Some(Vec::new());
  }

  Some(unsafe { std::slice::from_raw_parts(names, count as usize) }.to_vec())
}