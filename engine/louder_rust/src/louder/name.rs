use super::context::LouderContext;
#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, GString, Gd, PackedByteArray};

/// A generic hash map backing the engine's internal data stores.
pub type Map<K, V> = std::collections::HashMap<K, V>;

/// A generic hash set backing the engine's internal data stores.
pub type Set<T> = std::collections::HashSet<T>;

/// An encoding supported by the engine. Same as the rust "encoding" crate.
pub type LouderEncoding = usize;

/// A name stored in the engine.
///
/// Names map strings to integers. Obtain ids with [`crate::get_named_id`] and
/// [`crate::declare_name`], and read strings back with [`crate::read_name`].
/// Conversations are id'd by names containing the file they were loaded from.
pub type LouderName = i32;
pub const LOUDER_NAME_NONE: LouderName = -1;

/// The channel every conversation starts on, before any hub changes it.
pub const LOUDER_MAIN_CHANNEL: &str = "main";

godot_class! {
  /// The value of a name stored in the engine.
  ///
  /// Names map strings to integers. Obtain ids with the static methods below and
  /// read strings back the same way, rather than through
  /// [`crate::get_named_id`] and [`crate::read_name`], which are the same
  /// operations written for the C boundary.
  #[derive(Default, PartialEq, Eq, Clone)]
  pub struct LouderNameContent {
    pub bytes: Vec<u8>,
  }
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

godot_api! {
  impl LouderNameContent {
    /// A name holding `text`.
    #[func]
    pub fn from_text(text: GString) -> Gd<LouderNameContent> {
      Gd::from_object(LouderNameContent::new(&text.to_string()))
    }

    /// The content as text, or an empty string if the bytes are not UTF-8.
    #[func]
    pub fn text(&self) -> GString {
      GString::from(self.as_str().unwrap_or(""))
    }

    /// Replace the content with `text`.
    #[func]
    pub fn set_text(&mut self, text: GString) {
      self.bytes = text.to_string().into_bytes();
    }

    /// The raw content, byte for byte.
    #[func]
    pub fn bytes(&self) -> PackedByteArray {
      self.bytes.iter().copied().collect()
    }

    /// How many bytes the content is.
    #[func]
    pub fn size(&self) -> i32 {
      self.bytes.len() as i32
    }

    /// The id `context` already gives `text`, or [`LOUDER_NAME_NONE`].
    #[func]
    pub fn find(context: Gd<LouderContext>, text: GString) -> LouderName {
      let content = LouderNameContent::new(&text.to_string());
      match context.bind().find_name(&content) {
        LouderContextResult::Found(name) => name,
        LouderContextResult::NotFound(_) => LOUDER_NAME_NONE,
      }
    }

    /// The id `context` gives `text`, assigning one if it has none.
    ///
    /// A name that was already declared keeps the id it had.
    #[func]
    pub fn declare(context: Gd<LouderContext>, text: GString) -> LouderName {
      let mut context = context;
      let mut context = context.bind_mut();
      let content = LouderNameContent::new(&text.to_string());
      match context.add_name(&content) {
        LouderContextResult::Found(_) => LOUDER_NAME_NONE,
        LouderContextResult::NotFound(name) => name,
      }
    }

    /// The text a name id stands for, or an empty string if there is none.
    #[func]
    pub fn resolve(context: Gd<LouderContext>, name: LouderName) -> GString {
      match context.bind().resolve_name(name) {
        Some(text) => GString::from(text),
        None => GString::from(""),
      }
    }

    /// `text` split on commas and/or runs of whitespace.
    #[func]
    pub fn split_inline(text: GString) -> Array<GString> {
      split_inline(&text.to_string())
        .into_iter()
        .map(GString::from)
        .collect()
    }

    /// `text` split on newlines, dropping empty entries.
    #[func]
    pub fn split_lines(text: GString) -> Array<GString> {
      split_lines(&text.to_string())
        .into_iter()
        .map(GString::from)
        .collect()
    }
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