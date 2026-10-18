//! One loaded conversation: its hubs, and the story state private to it.

use crate::hub::Hub;
use crate::engine;
use louder_rust::godot_api;

use godot::prelude::*;

/// The contents of one loaded conversation.
///
/// A snapshot of what the engine held, so writing to one of these changes the
/// copy rather than the engine's own. What the engine holds changes through the
/// [`LouderContext`](crate::context::LouderContext).
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Conversation {
  pub(crate) inner: engine::Conversation,
}

godot_api! {
  impl Conversation {
    /// An empty conversation under `name`.
    #[func]
    pub fn create(name: engine::LouderName) -> Gd<Conversation> {
      Gd::from_object(Conversation { inner: engine::Conversation::new(name) })
    }

    /// The name this conversation is loaded under.
    #[func]
    pub fn get_name(&self) -> engine::LouderName {
      self.inner.name()
    }

    /// How many hubs this conversation holds.
    #[func]
    pub fn get_hub_count(&self) -> i32 {
      self.inner.hub_count() as i32
    }

    /// Every hub position, in ascending order.
    #[func]
    pub fn hub_positions(&self) -> Array<i64> {
      let mut positions: Vec<engine::LouderPosition> =
        self.inner.hubs().values().map(engine::Hub::position).collect();
      positions.sort_unstable();
      positions.into_iter().map(|position| position as i64).collect()
    }

    /// The hub at `position`, if this conversation has one there.
    #[func]
    pub fn get_hub(&self, position: engine::LouderPosition) -> Option<Gd<Hub>> {
      self.inner.hub(position).map(|hub| Gd::from_object(Hub { inner: hub.clone() }))
    }

    /// Insert a hub, reporting whether one already stood at its position.
    #[func]
    pub fn add_hub(&mut self, hub: Gd<Hub>) -> bool {
      self.inner.add_hub(hub.bind().inner.clone())
    }

    /// Whether an actor has already travelled through this hub.
    #[func]
    pub fn is_covered(&self, position: engine::LouderPosition) -> bool {
      self.inner.is_covered(position)
    }

    /// Record that this hub has been travelled through.
    #[func]
    pub fn mark_covered(&mut self, position: engine::LouderPosition) {
      self.inner.mark_covered(position);
    }

    /// Where a story state variable of this name is stored, as a
    /// [`VariableScope`](engine::VariableScope) index.
    ///
    /// A leading underscore means unsaved and an uppercase first letter means
    /// shared, so `_a` and `__1` are local and unsaved, `_A` is shared and
    /// unsaved, `a` is local and saved, and `A` is shared and saved.
    #[func]
    pub fn classify_variable(name: GString) -> i32 {
      scope_index(engine::classify_variable(&name.to_string()))
    }
  }
}

/// The scope as the `int` a host switches on.
pub(crate) fn scope_index(scope: engine::VariableScope) -> i32 {
  match scope {
    engine::VariableScope::LocalSaved => 0,
    engine::VariableScope::SharedSaved => 1,
    engine::VariableScope::LocalUnsaved => 2,
    engine::VariableScope::SharedUnsaved => 3,
  }
}