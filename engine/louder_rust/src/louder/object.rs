use strum_macros::EnumDiscriminants;

use super::name::{Map, LouderName};


/// Extra data attached to an actor, hub, or speaker, keyed by name.
pub struct HomogeneousObject {
  data: Map<LouderName, LouderVariant>,
}

/// One unit of data in an actor, hub, speaker, or story state.
#[derive(Debug, EnumDiscriminants)]
#[strum_discriminants(name(LouderVariantType), repr(u32))]
pub enum LouderVariant {
  Int(i32),
  Float(f32),
  String(LouderName),
}