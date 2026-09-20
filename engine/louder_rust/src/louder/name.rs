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
#[derive(PartialEq, Clone)]
pub struct LouderNameContent {
  pub bytes: Vec<u8>,
}

/// The map between names and their string values.
/// 
/// Names map strings to integers. Obtain ids with [`crate::get_named_id`] and
/// [`crate::declare_name`], and read strings back with [`crate::read_name`].
/// Conversations are id'd by names containing the file they were loaded from.
pub type LouderNameMap = Map<LouderName, LouderNameContent>;

pub enum LouderContextResult {
  Found(LouderName),
  NotFound(LouderName),
}

pub trait LouderNameContext {
  fn find_name(&self, name_context: &LouderNameContent) -> LouderContextResult;

  fn has_name(&self, name: &LouderName) -> bool;

  fn get_name_content(&self, name: &LouderName) -> Option<&LouderNameContent>;

  fn add_name(&mut self, name_context: &LouderNameContent) -> LouderContextResult;
}