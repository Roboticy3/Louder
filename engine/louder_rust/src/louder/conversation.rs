#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, GString, Gd};
use super::name::{Map, Set, LouderName};
use super::object::LouderVariant;
use super::hub::{Hub, LouderPosition};

/// Where a story state variable is stored, decided by its name.
///
/// Two independent rules combine into the four cases of the engine guide: a
/// leading underscore means "not saved", and an uppercase first letter means
/// "shared across conversations".
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariableScope {
  /// Saved, and private to the conversation it was written in.
  LocalSaved,
  /// Saved, and visible to every conversation.
  SharedSaved,
  /// Discarded on save, and private to the conversation.
  LocalUnsaved,
  /// Discarded on save, and visible to every conversation.
  SharedUnsaved,
}

/// Classify a variable by the naming rules of the engine guide.
///
/// `_a` and `__1` are local and unsaved, `_A` and `__Shared` are shared and
/// unsaved, `a` is local and saved, and `A` is shared and saved. A name with
/// no letters at all falls back to local and saved.
pub fn classify_variable(name: &str) -> VariableScope {
  let significant = name.trim_start_matches('_');
  let shared = significant
    .chars()
    .next()
    .is_some_and(|c| c.is_ascii_uppercase());
  let saved = !name.starts_with('_');

  match (shared, saved) {
    (false, true) => VariableScope::LocalSaved,
    (true, true) => VariableScope::SharedSaved,
    (false, false) => VariableScope::LocalUnsaved,
    (true, false) => VariableScope::SharedUnsaved,
  }
}

/// A variable bucket: one of the four scopes, holding name/variant pairs.
pub type VariableMap = Map<LouderName, LouderVariant>;

/// The contents of one loaded conversation: its hubs, and the story state that
/// is private to it.
///
/// Story state that is shared across conversations is held by the
/// [`LouderContext`](super::context::LouderContext) instead, so that loading
/// and unloading conversations does not disturb it.
///
/// A loaded conversation is handed to GDScript as a copy; what the engine holds
/// changes through the context.
#[derive(Clone, Default)]
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
pub struct Conversation {
  name: LouderName,
  hubs: Map<LouderPosition, Hub>,

  local_saved: VariableMap,
  local_unsaved: VariableMap,

  covered: Set<LouderPosition>,
}

impl Conversation {
  pub fn new(name: LouderName) -> Conversation {
    Conversation {
      name,
      hubs: Map::new(),
      local_saved: Map::new(),
      local_unsaved: Map::new(),
      covered: Set::new(),
    }
  }

  pub fn name(&self) -> LouderName {
    self.name
  }

  pub fn hubs(&self) -> &Map<LouderPosition, Hub> {
    &self.hubs
  }

  pub fn hub(&self, position: LouderPosition) -> Option<&Hub> {
    self.hubs.get(&position)
  }

  pub fn hub_mut(&mut self, position: LouderPosition) -> Option<&mut Hub> {
    self.hubs.get_mut(&position)
  }

  pub fn hub_count(&self) -> usize {
    self.hubs.len()
  }

  /// Insert a hub, reporting whether a hub already occupied `position`.
  pub fn add_hub(&mut self, hub: Hub) -> bool {
    let position = hub.position();
    self.hubs.insert(position, hub).is_some()
  }

  /// The story state bucket matching `scope`.
  ///
  /// Shared buckets belong to the context, not to a conversation, so reaching one
  /// through here is a caller mistake rather than a state that can occur.
  pub fn variables_mut(&mut self, scope: VariableScope) -> &mut VariableMap {
    match scope {
      VariableScope::LocalSaved => &mut self.local_saved,
      VariableScope::LocalUnsaved => &mut self.local_unsaved,
      _ => unreachable!("shared story state is owned by the context"),
    }
  }

  /// The story state bucket matching `scope`.
  pub fn variables(&self, scope: VariableScope) -> &VariableMap {
    match scope {
      VariableScope::LocalSaved => &self.local_saved,
      VariableScope::LocalUnsaved => &self.local_unsaved,
      _ => unreachable!("shared story state is owned by the context"),
    }
  }

  /// Whether this hub has already been travelled through.
  ///
  /// Statements only run the first time a hub is covered, so this is what makes
  /// them effectively one-shot.
  pub fn is_covered(&self, position: LouderPosition) -> bool {
    self.covered.contains(&position)
  }

  pub fn mark_covered(&mut self, position: LouderPosition) {
    self.covered.insert(position);
  }
}

godot_api! {
  impl Conversation {
    /// An empty conversation under `name`.
    #[func]
    pub fn create(name: LouderName) -> Gd<Conversation> {
      Gd::from_object(Conversation::new(name))
    }

    /// The name this conversation is loaded under.
    #[func]
    pub fn get_name(&self) -> LouderName {
      self.name()
    }

    /// How many hubs this conversation holds.
    #[func]
    pub fn get_hub_count(&self) -> i32 {
      self.hub_count() as i32
    }

    /// Every hub position, in ascending order.
    #[func]
    pub fn hub_positions(&self) -> Array<i64> {
      let mut positions: Vec<LouderPosition> = self.hubs().values().map(Hub::position).collect();
      positions.sort_unstable();
      positions.into_iter().map(|position| position as i64).collect()
    }

    /// The hub at `position`, if this conversation has one there.
    #[func]
    pub fn get_hub(&self, position: LouderPosition) -> Option<Gd<Hub>> {
      self.hub(position).map(|hub| Gd::from_object(hub.clone()))
    }

    /// Insert a hub, reporting whether one already stood at its position.
    #[func(rename = "add_hub")]
    pub fn gd_add_hub(&mut self, hub: Gd<Hub>) -> bool {
      Conversation::add_hub(self, hub.bind().clone())
    }

    /// Whether an actor has already travelled through this hub.
    #[func(rename = "is_covered")]
    pub fn gd_is_covered(&self, position: LouderPosition) -> bool {
      Conversation::is_covered(self, position)
    }

    /// Record that this hub has been travelled through.
    #[func(rename = "mark_covered")]
    pub fn gd_mark_covered(&mut self, position: LouderPosition) {
      Conversation::mark_covered(self, position);
    }

    /// Where a story state variable of this name is stored, as a
    /// [`VariableScope`] index.
    ///
    /// A leading underscore means unsaved and an uppercase first letter means
    /// shared, so `_a` and `__1` are local and unsaved, `_A` is shared and
    /// unsaved, `a` is local and saved, and `A` is shared and saved.
    #[func]
    pub fn classify_variable(name: GString) -> i32 {
      match classify_variable(&name.to_string()) {
        VariableScope::LocalSaved => 0,
        VariableScope::SharedSaved => 1,
        VariableScope::LocalUnsaved => 2,
        VariableScope::SharedUnsaved => 3,
      }
    }
  }
}