use super::context::LouderContext;

/// A generic hash map backing the engine's internal data stores.
pub type Map<K, V> = std::collections::HashMap<K, V>;

/// A generic hash set backing the engine's internal data stores.
pub type Set<T> = std::collections::HashSet<T>;

/// A name stored in the engine.
///
/// Names map bytes to integers. Obtain ids with [`crate::get_named_id`] and
/// [`crate::declare_name`], and read them back with [`crate::read_name`], which
/// hands over the bytes that went in. Conversations are id'd by names containing
/// the file they were loaded from.
pub type LouderName = i32;
pub const LOUDER_NAME_NONE: LouderName = -1;

/// The channel every conversation starts on, before any hub changes it.
pub const LOUDER_MAIN_CHANNEL: &str = "main";

/// The bytes behind a [`LouderName`].
///
/// Whatever a caller declared is kept as it was given and handed back the same
/// way, so a host that reads it as text is doing the decoding the engine never
/// does.
#[derive(Default, PartialEq, Eq, Clone)]
pub struct LouderNameContent {
  pub bytes: Vec<u8>,
}

impl LouderNameContent {
  pub fn new(text: &str) -> LouderNameContent {
    LouderNameContent {
      bytes: text.as_bytes().to_vec(),
    }
  }

  /// The content as UTF-8, or `None` if the bytes are not valid UTF-8.
  pub fn as_str(&self) -> Option<&str> {
    std::str::from_utf8(&self.bytes).ok()
  }
}


/// The map between names and their string values.
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

/// Name lookup that can also create names, which only the engine itself needs.
///
/// This is kept apart from [`LouderNameContext`] because the program driving
/// the engine should look names up and read them back, but should assign their
/// ids through [`crate::declare_name`] instead of growing the table behind the
/// engine's back.
pub trait LouderInternContext: LouderNameContext {
  /// The id of `text`, assigning a new one if it has not been declared.
  fn intern_name(&mut self, text: &str) -> LouderName;

  /// The id of `text`, or [`LOUDER_NAME_NONE`] if `text` is empty.
  fn intern_optional(&mut self, text: &str) -> LouderName;

  /// Intern every entry of a delimited list.
  fn intern_list(&mut self, parts: &[&str]) -> Vec<LouderName>;

  /// The content of `name` as a UTF-8 string slice.
  fn resolve_name(&self, name: LouderName) -> Option<&str>;
}

impl LouderInternContext for LouderContext {
  fn intern_name(&mut self, text: &str) -> LouderName {
    let content = LouderNameContent::new(text);
    match self.add_name(&content) {
      LouderContextResult::Found(name) => name,
      LouderContextResult::NotFound(name) => name,
    }
  }

  fn intern_optional(&mut self, text: &str) -> LouderName {
    if text.is_empty() {
      return LOUDER_NAME_NONE;
    }

    self.intern_name(text)
  }

  fn intern_list(&mut self, parts: &[&str]) -> Vec<LouderName> {
    let mut names = Vec::with_capacity(parts.len());
    for part in parts {
      names.push(self.intern_optional(part));
    }
    names
  }

  fn resolve_name(&self, name: LouderName) -> Option<&str> {
    self.get_name_content(&name).and_then(|c| c.as_str())
  }
}

/// Split on commas and/or runs of whitespace.
///
/// Tags are documented as "separated by commas or whitespace", so both
/// separators are accepted and empty entries are dropped.
pub fn split_inline(text: &str) -> Vec<&str> {
  text.split(|c: char| c == ',' || c.is_whitespace())
    .filter(|s| !s.is_empty())
    .collect()
}

/// Split on newlines, dropping empty entries.
///
/// Expressions, callbacks, recipients, and a speaker's listening list are all
/// documented as newline separated. `\r` is tolerated so documents saved with
/// Windows line endings load the same way.
pub fn split_lines(text: &str) -> Vec<&str> {
  text.split('\n')
    .map(|s| s.trim_end_matches('\r').trim())
    .filter(|s| !s.is_empty())
    .collect()
}
