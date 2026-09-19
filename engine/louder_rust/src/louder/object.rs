use std::hash::Hash;

use crate::louder::context::LouderContext;

/// A generic hash map backing the engine's internal data stores.
pub type Map<K, V> = std::collections::HashMap<K, V>;

// An encoding supported by the engine. Same as the rust "encoding" crate.
pub type LouderEncoding = usize;

/// A name stored in the engine.
///
/// Names map strings to integers. Obtain ids with [`crate::get_named_id`] and
/// [`crate::declare_name`], and read strings back with [`crate::read_name`].
/// Conversations are id'd by names containing the file they were loaded from.
pub type LouderName = i32;
pub const LOUDER_NAME_NONE: LouderName = -1;

/// The value of a name stored in the engine.
/// 
/// Names map strings to integers. Obtain ids with [`crate::get_named_id`] and
/// [`crate::declare_name`], and read strings back with [`crate::read_name`].
/// Conversations are id'd by names containing the file they were loaded from.
pub struct LouderNameContext {
  pub bytes: Vec<u8>,
  pub encoding: LouderEncoding,
}

impl PartialEq for LouderNameContext {
  fn eq(&self, other: &Self) -> bool {
    self.bytes == other.bytes && self.encoding == other.encoding
  }
}

impl Clone for LouderNameContext {
  fn clone(&self) -> Self {
    LouderNameContext {
      bytes: self.bytes.clone(),
      encoding: self.encoding,
    }
  }
}

/// The map between names and their string values.
/// 
/// Names map strings to integers. Obtain ids with [`crate::get_named_id`] and
/// [`crate::declare_name`], and read strings back with [`crate::read_name`].
/// Conversations are id'd by names containing the file they were loaded from.
pub type LouderNameMap = Map<LouderName, LouderNameContext>;


/// Extra data attached to an actor, hub, or speaker, keyed by name.
pub struct HomogeneousObject {
  data: Map<LouderName, LouderVariant>,
}

/// One unit of data in an actor, hub, speaker, or story state.
pub enum LouderVariant {
  Int(i32),
  Float(f32),
  String(String),
}

/// The type of data stored in a variant.
///
/// Over the C API this is an `int`, so the variants carry explicit `#[repr(i32)]`.
#[repr(i32)]
pub enum LouderVariantType {
  /// An integer value.
  Int,
  /// A floating point value.
  Float,
  /// A string value.
  String,
}