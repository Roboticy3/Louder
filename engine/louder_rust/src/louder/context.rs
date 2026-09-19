use std::hash::{DefaultHasher, Hash, Hasher};

use crate::louder::object::LouderNameContext;

use super::object::{LouderName, LouderNameMap, Map};
use super::speaker::Speaker;
use super::actor::Actor;
use super::conversation::Conversation;
use super::event::{LouderEvent, LouderAsk};

/// The opaque handle to the engine.
///
/// Obtained from [`crate::init_louder`] and released with
/// [`crate::deinit_louder`]. The contents are internal to the engine and not
/// part of the public API.
pub struct LouderContext {
  hasher: DefaultHasher,
  names: LouderNameMap,
  speakers: Map<LouderName, Speaker>,
  actors: Map<LouderName, Actor>,
  conversations: Map<LouderName, Conversation>,
  events: Vec<LouderEvent>,
  asks: Vec<LouderAsk>,
}

pub enum LouderContextResult {
  Found(LouderName),
  NotFound(LouderName),
}

impl LouderContext {
  pub fn new() -> LouderContext {
    LouderContext {
      hasher: DefaultHasher::new(),
      names: LouderNameMap::new(),
      speakers: Map::new(),
      actors: Map::new(),
      conversations: Map::new(),
      events: Vec::new(),
      asks: Vec::new(),
    }
  }

  pub fn find_name(&self, name_context: &LouderNameContext) -> LouderContextResult {
    let mut hasher = DefaultHasher::new();
    name_context.bytes.hash(&mut hasher);
    name_context.encoding.hash(&mut hasher);
    
    let mut name = hasher.finish() as LouderName;
    let mut iterations = 0;
    while self.names.contains_key(&name) {
      if &self.names[&name] == name_context {
        return LouderContextResult::Found(name);
      }

      name = name.wrapping_add(iterations * iterations + 1);
      iterations += 1;
    }

    LouderContextResult::NotFound(name)
  }

  pub fn has_name(&self, name: &LouderName) -> bool {
    self.names.contains_key(name)
  }

  pub fn get_name_context(&self, name: &LouderName) -> Option<&LouderNameContext> {
    self.names.get(name)
  }

  pub fn add_name(&mut self, name_context: &LouderNameContext) -> LouderContextResult {
    match self.find_name(name_context) {
      LouderContextResult::Found(name) => LouderContextResult::Found(name),
      LouderContextResult::NotFound(name) => {
        self.names.insert(name, name_context.clone());
        LouderContextResult::NotFound(name)
      }
    }
  }
}