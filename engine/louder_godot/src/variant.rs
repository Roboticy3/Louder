//! The bridge between an engine [`LouderVariant`](crate::engine::LouderVariant)
//! and Godot's own dynamic value.
//!
//! A variant is a tagged union, which has no counterpart among Godot's built-in
//! types, so a variant reaches GDScript as a [`Variant`]: the one Godot type
//! able to hold an int, a float, or a string without saying which in its Rust
//! declaration. The integer and float kinds map across one for one, and a
//! string becomes the text its name stands for.
//!
//! These are free functions rather than methods on the variant because the
//! variant belongs to the engine, and Rust does not allow an `impl` block for a
//! type outside the crate that defines it.

use crate::context::LouderContext;
use crate::engine;
use crate::engine::LouderInternContext;

use godot::prelude::*;

/// This variant as Godot sees it, reading a string through `context`.
pub(crate) fn to_godot(variant: &engine::LouderVariant, context: &engine::LouderContext) -> Variant {
  match variant {
    engine::LouderVariant::Int(value) => Variant::from(*value as i64),
    engine::LouderVariant::Float(value) => Variant::from(*value as f64),
    engine::LouderVariant::String(name) => {
      let text = context.resolve_name(*name).unwrap_or("");
      Variant::from(GString::from(text))
    }
  }
}

/// Read a variant out of Godot, interning a string so it can be stored.
///
/// Anything the engine has no kind for reads as 0, which is also what an unset
/// variable reads as.
pub(crate) fn from_godot(context: &mut engine::LouderContext, value: &Variant) -> engine::LouderVariant {
  if let Ok(found) = value.try_to::<bool>() {
    return engine::LouderVariant::from_bool(found);
  }
  if let Ok(found) = value.try_to::<i64>() {
    return engine::LouderVariant::Int(found as i32);
  }
  if let Ok(found) = value.try_to::<f64>() {
    return engine::LouderVariant::Float(found as f32);
  }
  if let Ok(found) = value.try_to::<GString>() {
    return engine::LouderVariant::String(context.intern_name(&found.to_string()));
  }

  engine::LouderVariant::zero()
}

/// [`to_godot`] against the engine a [`LouderContext`] wraps.
pub(crate) fn to_godot_from(variant: &engine::LouderVariant, context: &LouderContext) -> Variant {
  to_godot(variant, &context.inner)
}

/// [`from_godot`] against the engine a [`LouderContext`] wraps.
pub(crate) fn from_godot_from(context: &mut LouderContext, value: &Variant) -> engine::LouderVariant {
  from_godot(&mut context.inner, value)
}