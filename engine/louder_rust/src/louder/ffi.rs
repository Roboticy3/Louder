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
//! over a [`String`] that is already copied, so an adapter can take and return
//! text directly and never need to ask these questions. [`LouderError`] is the
//! one piece of vocabulary that survives, and an adapter is free to name its
//! codes however it likes.
//!
//! Bytes are bytes. Nothing here knows what encoding a caller wrote them in or
//! reads them back out, because the engine never displays anything and so has no
//! opinion on the matter.

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

/// Copy `bytes` into `buffer`, truncating if needed, and null terminate.
///
/// Returns the number of bytes written, excluding the terminator. The bytes are
/// copied through unchanged, so whatever the caller put in is what comes back
/// out; the terminator is a convenience for callers treating the buffer as C
/// text, and the return value is what a caller should trust instead.
pub(crate) fn write_bytes(buffer: *mut u8, max_size: usize, bytes: &[u8]) -> usize {
  if buffer.is_null() {
    return 0;
  }

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
