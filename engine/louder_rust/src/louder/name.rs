use std::hash::Hash;

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
pub struct LouderNameContent {
  pub bytes: Vec<u8>,
  pub encoding: LouderEncoding,
}

impl PartialEq for LouderNameContent {
  fn eq(&self, other: &Self) -> bool {
    self.bytes == other.bytes && self.encoding == other.encoding
  }
}

impl Clone for LouderNameContent {
  fn clone(&self) -> Self {
    LouderNameContent {
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
pub type LouderNameMap = Map<LouderName, LouderNameContent>;