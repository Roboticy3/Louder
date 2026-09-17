use super::{LouderName, Map};

/// Extra data attached to an actor, hub, or speaker, keyed by name.
pub(crate) struct HomogeneousObject {
  data: Map<LouderName, LouderVariant>,
}

/// One unit of data in an actor, hub, speaker, or story state.
enum LouderVariant {
  Int(i32),
  Float(f32),
  String(String),
}

/// The type of data stored in a variant.
///
/// Over the C API this is an `int`, so the variants carry explicit `#[repr(i32)]`.
#[repr(i32)]
pub enum LouderVariantType {
  /// An integer value.
  Int,
  /// A floating point value.
  Float,
  /// A string value.
  String,
}