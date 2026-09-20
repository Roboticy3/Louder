use super::name::{Map, LouderName};


/// Extra data attached to an actor, hub, or speaker, keyed by name.
pub struct HomogeneousObject {
  data: Map<LouderName, LouderVariant>,
}

/// One unit of data in an actor, hub, speaker, or story state.
#[derive(Debug, Clone)]
#[repr(C)]
pub enum LouderVariant {
  Int(i32),
  Float(f32),
  String(LouderName),
}