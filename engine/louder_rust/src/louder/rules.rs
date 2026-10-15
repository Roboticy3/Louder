//! The rules a host turns on and off.
//!
//! The rules themselves are a set of bits, which crosses into Godot as an
//! `int`. [`DialogueRules`] is the class that turns those bits into the names a
//! host actually wants to write in GDScript.

#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, GString};

bitflags::bitflags! {
  /// The dialogue rules a host can turn on and off.
  ///
  /// Both are on by default, because each is what makes the engine usable
  /// without every conversation having to be written around its absence. They
  /// are gathered here rather than left as separate switches so that a host
  /// reads and writes the whole configuration in one call, and so that adding a
  /// rule does not add another pair of engine calls.
  #[repr(C)]
  #[derive(Clone, Copy)]
  pub struct DialogueRules: i32 {
    /// The spoke rule. A lone choice on a hub may be taken by an actor who
    /// neither speaks nor hears the voice behind it, since there is nobody else
    /// to hand it to and no branch for it to be lost on. Without it, linear
    /// dialogue hands the lead back and forth between every actor instead of
    /// letting one of them carry it.
    const SPOKE = 0b00000001;

    /// The singleplayer rule. A hub that only the actor who just chose can see,
    /// reached from a hub that had several choices, resolves on its own instead
    /// of being shown. How many choices it carries is beside the point: it is
    /// the consequence of a choice rather than one, so there is nothing in it
    /// for that actor to decide. This is the "continue" of a more traditional
    /// singleplayer dialogue system, made implicit, so that a private branch
    /// can be routed through without being written as a decision.
    const SINGLEPLAYER = 0b00000010;
  }
}

godot_class! {
  /// The dialogue rules, read and written by name.
  ///
  /// A host reads and writes the whole configuration at once through
  /// [`LouderContext::get_dialogue_rules`](super::context::LouderContext) and
  /// [`LouderContext::set_dialogue_rules`](super::context::LouderContext), which
  /// take the bits. This class is how a GDScript writer gets at those bits
  /// without hard-coding them: pass the names they care about, and read back the
  /// names currently set.
  pub struct LouderDialogueRules {}
}

godot_api! {
  impl LouderDialogueRules {
    /// The bits of a fresh engine, which has every rule on.
    #[func]
    pub fn defaults() -> i32 {
      DialogueRules::all().bits()
    }

    /// The bits named by `names`, ignoring anything that is not a rule.
    #[func]
    pub fn from_names(names: Array<GString>) -> i32 {
      let mut rules = DialogueRules::empty();
      for name in names.iter_shared() {
        rules |= match &*name.to_string() {
          "SPOKE" => DialogueRules::SPOKE,
          "SINGLEPLAYER" => DialogueRules::SINGLEPLAYER,
          _ => DialogueRules::empty(),
        };
      }
      rules.bits()
    }

    /// The names of the rules set in `bits`.
    #[func]
    pub fn to_names(bits: i32) -> Array<GString> {
      let rules = DialogueRules::from_bits_truncate(bits);
      let mut names: Vec<GString> = Vec::new();
      if rules.contains(DialogueRules::SPOKE) {
        names.push(GString::from("SPOKE"));
      }
      if rules.contains(DialogueRules::SINGLEPLAYER) {
        names.push(GString::from("SINGLEPLAYER"));
      }
      names.into_iter().collect()
    }
  }
}