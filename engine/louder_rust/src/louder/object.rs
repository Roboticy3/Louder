#[cfg(feature = "godot-bindings")]
use super::context::LouderContext;
#[cfg(feature = "godot-bindings")]
use super::godot_api::Array;
#[cfg(feature = "godot-bindings")]
use super::name::LouderInternContext;
use super::name::{Map, LouderName};

/// Extra data attached to an actor, hub, or speaker, keyed by name.
///
/// This is the "homogeneous object" of the engine guide: a flat mapping between
/// string names and [`LouderVariant`]s. Key order is not meaningful, so
/// [`HomogeneousObject::sorted_keys`] is provided for output that must be
/// deterministic.
///
/// A [`LouderVariant`] has no built-in Godot counterpart, so each value is
/// read and written through the kind-specific `get_value_*` and `set_value_*`
/// methods below rather than through one method that would have to return a
/// tagged union across the boundary.
#[derive(Clone)]
#[cfg_attr(feature = "godot-bindings", derive(::godot::prelude::GodotClass))]
#[cfg_attr(feature = "godot-bindings", class(init, tool))]
pub struct HomogeneousObject {
  data: Map<LouderName, LouderVariant>,
}

impl HomogeneousObject {
  pub fn new() -> HomogeneousObject {
    HomogeneousObject {
      data: Map::new(),
    }
  }

  /// The variant stored under `key`, or `None` if the key is absent.
  pub fn get(&self, key: LouderName) -> Option<&LouderVariant> {
    self.data.get(&key)
  }

  /// Store `value` under `key`, returning the variant that was replaced.
  pub fn set(&mut self, key: LouderName, value: LouderVariant) -> Option<LouderVariant> {
    self.data.insert(key, value)
  }

  /// Forget `key`, returning the variant that was stored under it.
  pub fn remove(&mut self, key: LouderName) -> Option<LouderVariant> {
    self.data.remove(&key)
  }

  /// Forget every key.
  pub fn clear(&mut self) {
    self.data.clear();
  }

  pub fn len(&self) -> usize {
    self.data.len()
  }

  pub fn is_empty(&self) -> bool {
    self.data.is_empty()
  }

  /// Every key in ascending id order.
  ///
  /// [`Map`] iteration order is unspecified, so any output built from this
  /// object must be built from these keys instead of from [`Self::iter`] to
  /// stay reproducible between runs.
  pub fn sorted_keys(&self) -> Vec<LouderName> {
    let mut keys: Vec<LouderName> = self.data.keys().copied().collect();
    keys.sort_unstable();
    keys
  }

  /// Every key and variant in unspecified order.
  pub fn iter(&self) -> impl Iterator<Item = (&LouderName, &LouderVariant)> {
    self.data.iter()
  }
}

impl Default for HomogeneousObject {
  fn default() -> Self {
    return HomogeneousObject::new()
  }
}

godot_api! {
  impl HomogeneousObject {
    /// How many keys are stored.
    #[func]
    pub fn size(&self) -> i32 {
      self.len() as i32
    }

    /// Whether `key` has a value.
    #[func]
    pub fn has(&self, key: LouderName) -> bool {
      self.get(key).is_some()
    }

    /// Every key, in ascending id order.
    #[func]
    pub fn keys(&self) -> Array<i64> {
      self.sorted_keys().into_iter().map(|key| key as i64).collect()
    }

    /// The value under `key` read as an integer, or 0.
    #[func]
    pub fn get_value_int(&self, key: LouderName) -> i32 {
      self.get(key).map(LouderVariant::as_int).unwrap_or(0)
    }

    /// The value under `key` read as a float, or 0.0.
    #[func]
    pub fn get_value_float(&self, key: LouderName) -> f64 {
      self.get(key).map(LouderVariant::as_float).unwrap_or(0.0) as f64
    }

    /// The name under `key`, or
    /// [`LOUDER_NAME_NONE`](super::name::LOUDER_NAME_NONE) if the value is not a
    /// string.
    #[func]
    pub fn get_value_name(&self, key: LouderName) -> LouderName {
      match self.get(key) {
        Some(LouderVariant::String(name)) => *name,
        _ => super::name::LOUDER_NAME_NONE,
      }
    }

    /// Store an integer under `key`.
    #[func]
    pub fn set_value_int(&mut self, key: LouderName, value: i32) {
      self.set(key, LouderVariant::Int(value));
    }

    /// Store a float under `key`.
    #[func]
    pub fn set_value_float(&mut self, key: LouderName, value: f64) {
      self.set(key, LouderVariant::Float(value as f32));
    }

    /// Store a name under `key`.
    #[func]
    pub fn set_value_name(&mut self, key: LouderName, value: LouderName) {
      self.set(key, LouderVariant::String(value));
    }

    /// Forget `key`.
    #[func]
    pub fn forget(&mut self, key: LouderName) {
      HomogeneousObject::remove(self, key);
    }

    /// Forget every key.
    #[func]
    pub fn forget_all(&mut self) {
      HomogeneousObject::clear(self);
    }
  }
}

/// One unit of data in an actor, hub, speaker, or story state.
///
/// Booleans are not a separate kind: they are carried as [`LouderVariant::Int`]
/// 1 or 0, which is what makes them interoperable with the arithmetic and
/// comparison rules of the engine guide.
#[derive(Debug, Clone)]
#[repr(C)]
pub enum LouderVariant {
  Int(i32),
  Float(f32),
  String(LouderName),
}

impl Default for LouderVariant {
  /// Uninitialized variables read as 0.
  fn default() -> Self {
    return LouderVariant::Int(0)
  }
}

impl LouderVariant {
  /// The value an unset variable reads as.
  pub fn zero() -> LouderVariant {
    LouderVariant::Int(0)
  }

  pub fn from_bool(value: bool) -> LouderVariant {
    LouderVariant::Int(if value { 1 } else { 0 })
  }

  pub fn from_i32(value: i32) -> LouderVariant {
    LouderVariant::Int(value)
  }

  pub fn from_f32(value: f32) -> LouderVariant {
    LouderVariant::Float(value)
  }

  pub fn from_name(value: LouderName) -> LouderVariant {
    LouderVariant::String(value)
  }

  /// The discriminant of this variant, for callers that branch on the kind
  /// without inspecting the payload.
  pub fn kind(&self) -> VariantKind {
    match self {
      LouderVariant::Int(_) => VariantKind::Int,
      LouderVariant::Float(_) => VariantKind::Float,
      LouderVariant::String(_) => VariantKind::String,
    }
  }

  /// This variant read as an integer.
  ///
  /// Floats truncate and strings go to zero, per the engine guide's rule that
  /// variants can always be treated as integers.
  pub fn as_int(&self) -> i32 {
    match self {
      LouderVariant::Int(i) => *i,
      LouderVariant::Float(f) => *f as i32,
      LouderVariant::String(_) => 0,
    }
  }

  /// This variant read as a float.
  ///
  /// Strings go to zero, matching [`Self::as_int`].
  pub fn as_float(&self) -> f32 {
    match self {
      LouderVariant::Int(i) => *i as f32,
      LouderVariant::Float(f) => *f,
      LouderVariant::String(_) => 0.0,
    }
  }

  /// Whether this variant counts as true in a condition.
  ///
  /// An empty string and the string `"0"` are both false, which is what lets a
  /// condition written as a bare string behave like a number.
  ///
  /// `text` is the already-resolved content of a string variant; the caller
  /// reads it out of the context, since a [`LouderName`] is meaningless
  /// without it. It is ignored for the numeric kinds.
  pub fn is_truthy(&self, text: Option<&str>) -> bool {
    match self {
      LouderVariant::Int(i) => *i != 0,
      LouderVariant::Float(f) => *f != 0.0,
      LouderVariant::String(_) => match text {
        None => false,
        Some(t) => !t.is_empty() && t != "0",
      },
    }
  }
}

/// Which kind of data a [`LouderVariant`] holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VariantKind {
  Int,
  Float,
  String,
}

/// Arithmetic operators available to expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArithOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
}

/// Relational operators available to expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
  Eq,
  Ne,
  Lt,
  Le,
  Gt,
  Ge,
}

impl LouderVariant {
  /// Apply a numeric operator, coercing both sides as the engine guide
  /// describes.
  ///
  /// The result is a float when either operand is a float and an integer
  /// otherwise. Division and modulo by zero yield 0 rather than panicking,
  /// because a malformed expression must not take down the host program.
  ///
  /// String operands are *not* handled here: they coerce to 0, so the
  /// expression evaluator performs concatenation before calling this.
  pub fn arith(&self, op: ArithOp, other: &LouderVariant) -> LouderVariant {
    let both_int = self.kind() != VariantKind::Float && other.kind() != VariantKind::Float;

    if both_int {
      let a = self.as_int();
      let b = other.as_int();
      let result = match op {
        ArithOp::Add => a.wrapping_add(b),
        ArithOp::Sub => a.wrapping_sub(b),
        ArithOp::Mul => a.wrapping_mul(b),
        ArithOp::Div => if b == 0 { 0 } else { a.wrapping_div(b) },
        ArithOp::Mod => if b == 0 { 0 } else { a.wrapping_rem(b) },
      };
      return LouderVariant::Int(result);
    }

    let a = self.as_float();
    let b = other.as_float();
    let result = match op {
      ArithOp::Add => a + b,
      ArithOp::Sub => a - b,
      ArithOp::Mul => a * b,
      ArithOp::Div => a / b,
      ArithOp::Mod => a % b,
    };
    LouderVariant::Float(result)
  }

  /// Compare two variants numerically.
  ///
  /// Equality between two strings is *not* handled here, because resolving it
  /// requires reading the strings back out of the context. The evaluator
  /// handles that case before falling back to this.
  pub fn cmp_numeric(&self, op: CmpOp, other: &LouderVariant) -> bool {
    if self.kind() != VariantKind::Float && other.kind() != VariantKind::Float {
      let a = self.as_int();
      let b = other.as_int();
      return match op {
        CmpOp::Eq => a == b,
        CmpOp::Ne => a != b,
        CmpOp::Lt => a < b,
        CmpOp::Le => a <= b,
        CmpOp::Gt => a > b,
        CmpOp::Ge => a >= b,
      };
    }

    let a = self.as_float();
    let b = other.as_float();
    match op {
      CmpOp::Eq => a == b,
      CmpOp::Ne => a != b,
      CmpOp::Lt => a < b,
      CmpOp::Le => a <= b,
      CmpOp::Gt => a > b,
      CmpOp::Ge => a >= b,
    }
  }
}

/// The bridge between a [`LouderVariant`] and Godot's own dynamic value.
///
/// [`LouderVariant`] is a tagged union, which has no counterpart among Godot's
/// built-in types, so a variant reaches GDScript as a
/// [`Variant`](::godot::builtin::Variant): the one Godot
/// type able to hold an int, a float, or a string without saying which in its
/// Rust declaration. The integer and float kinds map across one for one, and a
/// string becomes the text its name stands for.
#[cfg(feature = "godot-bindings")]
impl LouderVariant {
  /// This variant as Godot sees it, reading a string through `context`.
  pub fn to_godot(&self, context: &LouderContext) -> super::godot_api::Variant {
    match self {
      LouderVariant::Int(value) => super::godot_api::Variant::from(*value as i64),
      LouderVariant::Float(value) => super::godot_api::Variant::from(*value as f64),
      LouderVariant::String(name) => {
        let text = context.resolve_name(*name).unwrap_or("");
        super::godot_api::Variant::from(super::godot_api::GString::from(text))
      }
    }
  }

  /// Read a variant out of Godot, interning a string so it can be stored.
  ///
  /// Anything the engine has no kind for reads as 0, which is also what an
  /// unset variable reads as.
  pub fn from_godot(context: &mut LouderContext, value: &super::godot_api::Variant) -> LouderVariant {
    use super::godot_api::GString;

    if let Ok(found) = value.try_to::<bool>() {
      return LouderVariant::from_bool(found);
    }
    if let Ok(found) = value.try_to::<i64>() {
      return LouderVariant::Int(found as i32);
    }
    if let Ok(found) = value.try_to::<f64>() {
      return LouderVariant::Float(found as f32);
    }
    if let Ok(found) = value.try_to::<GString>() {
      return LouderVariant::String(context.intern_name(&found.to_string()));
    }

    LouderVariant::zero()
  }
}