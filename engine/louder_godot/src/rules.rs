//! The dialogue rules, read and written by name rather than by bit.

use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The dialogue rules as a host writes them.
///
/// A host reads and writes the whole configuration at once through
/// [`LouderContext::get_dialogue_rules`](crate::context::LouderContext) and
/// [`LouderContext::set_dialogue_rules`](crate::context::LouderContext), which
/// take the bits. This class is how a GDScript writer gets at those bits
/// without hard-coding them: pass the names they care about, and read back the
/// names currently set.
#[derive(GodotClass)]
#[class(init, tool)]
pub struct LouderDialogueRules {}

godot_api! {
  impl LouderDialogueRules {
    /// The bits of a fresh engine, which has every rule on.
    #[func]
    pub fn defaults() -> i32 {
      engine::DialogueRules::all().bits()
    }

    /// The bits named by `names`, ignoring anything that is not a rule.
    #[func]
    pub fn from_names(names: Array<GString>) -> i32 {
      let mut rules = engine::DialogueRules::empty();
      for name in names.iter_shared() {
        if let Some(found) = engine::DialogueRules::from_name(&name.to_string()) {
          rules |= found;
        }
      }
      rules.bits()
    }

    /// The names of the rules set in `bits`.
    #[func]
    pub fn to_names(bits: i32) -> Array<GString> {
      engine::DialogueRules::from_bits_truncate(bits)
        .names()
        .into_iter()
        .map(GString::from)
        .collect()
    }
  }
}