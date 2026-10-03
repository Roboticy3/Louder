use super::name::{Map, LouderName};
use super::object::{HomogeneousObject, LouderVariant};
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

impl VariableScope {
  pub fn is_shared(&self) -> bool {
    matches!(self, VariableScope::SharedSaved | VariableScope::SharedUnsaved)
  }

  pub fn is_saved(&self) -> bool {
    matches!(self, VariableScope::LocalSaved | VariableScope::SharedSaved)
  }
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

/// The contents of one loaded conversation: its hubs, its additional data, and
/// the story state that is private to it.
///
/// Story state that is shared across conversations is held by the context
/// instead, so that loading and unloading conversations does not disturb it.
pub struct Conversation {
  name: LouderName,
  hubs: Map<LouderPosition, Hub>,
  additional_data: HomogeneousObject,

  local_saved: VariableMap,
  local_unsaved: VariableMap,

  covered: Map<LouderPosition, bool>,
}

impl Conversation {
  pub fn new(name: LouderName) -> Conversation {
    Conversation {
      name,
      hubs: Map::new(),
      additional_data: HomogeneousObject::new(),
      local_saved: Map::new(),
      local_unsaved: Map::new(),
      covered: Map::new(),
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

  /// Every hub position in ascending order.
  ///
  /// Positions are sparse, so this is the canonical iteration order that the
  /// `next_hub_*` functions walk.
  pub fn sorted_positions(&self) -> Vec<LouderPosition> {
    let mut positions: Vec<LouderPosition> = self.hubs.keys().copied().collect();
    positions.sort_unstable();
    positions
  }

  pub fn additional_data(&self) -> &HomogeneousObject {
    &self.additional_data
  }

  pub fn additional_data_mut(&mut self) -> &mut HomogeneousObject {
    &mut self.additional_data
  }

  /// The story state bucket matching `scope`, for scopes private to this
  /// conversation.
  pub fn variables_mut(&mut self, scope: VariableScope) -> &mut VariableMap {
    match scope {
      VariableScope::LocalSaved => &mut self.local_saved,
      VariableScope::LocalUnsaved => &mut self.local_unsaved,
      VariableScope::SharedSaved | VariableScope::SharedUnsaved => {
        unreachable!("shared story state is owned by the context")
      }
    }
  }

  /// The story state bucket matching `scope`, for scopes private to this
  /// conversation.
  pub fn variables(&self, scope: VariableScope) -> &VariableMap {
    match scope {
      VariableScope::LocalSaved => &self.local_saved,
      VariableScope::LocalUnsaved => &self.local_unsaved,
      VariableScope::SharedSaved | VariableScope::SharedUnsaved => {
        unreachable!("shared story state is owned by the context")
      }
    }
  }

  /// Whether this hub has already been travelled through.
  ///
  /// Statements only run the first time a hub is covered, so this is what makes
  /// them effectively one-shot.
  pub fn is_covered(&self, position: LouderPosition) -> bool {
    self.covered.contains_key(&position)
  }

  pub fn mark_covered(&mut self, position: LouderPosition) {
    self.covered.insert(position, true);
  }

  /// Every covered hub position in ascending order.
  pub fn covered_positions(&self) -> Vec<LouderPosition> {
    let mut positions: Vec<LouderPosition> = self.covered.keys().copied().collect();
    positions.sort_unstable();
    positions
  }
}