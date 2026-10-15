#[cfg(feature = "godot-bindings")]
use super::godot_api::{Array, GString, Gd};
use super::context::LouderContext;
use super::event::*;
use super::name::*;
use super::actor::LouderActor;
use super::object::{ArithOp, CmpOp, LouderVariant, VariantKind};

#[cfg(feature = "godot-bindings")]
use super::name::LouderInternContext;

/// A snippet of code, identified by the interned copy of its own source text.
///
/// Expressions are stored as their own text rather than as parsed trees,
/// because a conversation usually contains far more of them than any one run
/// ever reaches.
pub type LouderExpression = LouderName;

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
  Int(i32),
  Float(f32),
  Str(String),
  Bool(bool),

  /// A story state variable, named by its source text.
  Var(String),

  Neg(Box<Expr>),
  Not(Box<Expr>),

  Binary(BinOp, Box<Expr>, Box<Expr>),
  Assign(String, Box<Expr>),
  /// `+=` and friends, desugared into a binary node at parse time.
  PreInc(String, i32),
  PostInc(String, i32),

  /// `name(a, b)`
  Call(String, Vec<Expr>),
  /// `$(a, b)` — suspends for the host to answer.
  Ask(Vec<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
  Add,
  Sub,
  Mul,
  Div,
  Mod,
  Eq,
  Ne,
  Lt,
  Le,
  Gt,
  Ge,
  And,
  Or,
  BitAnd,
  BitOr,
  BitXor,
  Shl,
  Shr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
  Int(i32),
  Float(f32),
  Str(String),
  Ident(String),
  Op(&'static str),
  Ask,
  Eof,
}

godot_class! {
  /// An expression the engine could not read.
  #[derive(Debug, Clone, PartialEq, Default)]
  pub struct ParseError {
    pub message: String,
    pub offset: usize,
  }
}

impl ParseError {
  fn new(message: impl Into<String>, offset: usize) -> ParseError {
    ParseError {
      message: message.into(),
      offset,
    }
  }
}

/// Multi-character operators, longest first so that `<=` wins over `<`.
const OPERATORS: &[&str] = &[
  "<<=", ">>=", "&&", "||", "==", "!=", "<=", ">=", "<<", ">>", "++", "--", "+=", "-=", "*=", "/=",
  "%=", "&=", "|=", "^=", "+", "-", "*", "/", "%", "=", "<", ">", "!", "&", "|", "^", "(", ")", ",",
];

pub fn tokenize(source: &str) -> Result<Vec<Token>, ParseError> {
  let bytes: Vec<char> = source.chars().collect();
  let mut tokens = Vec::new();
  let mut i = 0;

  while i < bytes.len() {
    let c = bytes[i];

    if c.is_whitespace() {
      i += 1;
      continue;
    }

    // A `$` immediately followed by `(` opens an ask.
    if c == '$' {
      if i + 1 < bytes.len() && bytes[i + 1] == '(' {
        tokens.push(Token::Ask);
        i += 2;
        continue;
      }
      return Err(ParseError::new("expected `(` after `$`", i));
    }

    if c.is_ascii_digit() {
      let start = i;
      let mut is_float = false;
      while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == '.') {
        if bytes[i] == '.' {
          // Only treat `.` as part of the number when a digit follows, so that
          // `1.foo` does not silently become a float.
          if is_float || i + 1 >= bytes.len() || !bytes[i + 1].is_ascii_digit() {
            break;
          }
          is_float = true;
        }
        i += 1;
      }

      let text: String = bytes[start..i].iter().collect();
      if is_float {
        let value = text.parse::<f32>().map_err(|_| ParseError::new("malformed float", start))?;
        tokens.push(Token::Float(value));
      } else {
        // Values that do not fit an i32 fall back to a float rather than
        // failing the whole expression.
        match text.parse::<i32>() {
          Ok(value) => tokens.push(Token::Int(value)),
          Err(_) => {
            let value = text.parse::<f32>().map_err(|_| ParseError::new("malformed number", start))?;
            tokens.push(Token::Float(value));
          }
        }
      }
      continue;
    }

    if c == '_' || c.is_alphabetic() {
      let start = i;
      while i < bytes.len() && (bytes[i] == '_' || bytes[i].is_alphanumeric()) {
        i += 1;
      }
      tokens.push(Token::Ident(bytes[start..i].iter().collect()));
      continue;
    }

    if c == '"' || c == '\'' {
      let quote = c;
      i += 1;
      let mut value = String::new();
      loop {
        if i >= bytes.len() {
          return Err(ParseError::new("unterminated string", start_of(&bytes, i)));
        }
        if bytes[i] == '\\' && i + 1 < bytes.len() {
          let escaped = bytes[i + 1];
          value.push(match escaped {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            other => other,
          });
          i += 2;
          continue;
        }
        if bytes[i] == quote {
          i += 1;
          break;
        }
        value.push(bytes[i]);
        i += 1;
      }
      tokens.push(Token::Str(value));
      continue;
    }

    let rest: String = bytes[i..].iter().collect();
    match OPERATORS.iter().find(|op| rest.starts_with(**op)) {
      Some(op) => {
        tokens.push(Token::Op(op));
        i += op.chars().count();
      }
      None => return Err(ParseError::new(format!("unexpected character `{c}`"), i)),
    }
  }

  tokens.push(Token::Eof);
  Ok(tokens)
}

fn start_of(bytes: &[char], i: usize) -> usize {
  if i >= bytes.len() {
    bytes.len()
  } else {
    i
  }
}

struct Parser {
  tokens: Vec<Token>,
  position: usize,
}

impl Parser {
  fn peek(&self) -> &Token {
    self.tokens.get(self.position).unwrap_or(&Token::Eof)
  }

  fn next(&mut self) -> Token {
    let token = self.peek().clone();
    self.position += 1;
    token
  }

  fn eat_op(&mut self, op: &str) -> bool {
    if matches!(self.peek(), Token::Op(found) if *found == op) {
      self.position += 1;
      return true;
    }
    false
  }

  fn expect_op(&mut self, op: &str) -> Result<(), ParseError> {
    if self.eat_op(op) {
      return Ok(());
    }
    Err(ParseError::new(format!("expected `{op}`"), self.position))
  }

  fn parse_expression(&mut self) -> Result<Expr, ParseError> {
    self.parse_binary(0)
  }

  /// Precedence climbing. Each level lists its operators from loosest to
  /// tightest binding.
  fn parse_binary(&mut self, level: usize) -> Result<Expr, ParseError> {
    const LEVELS: &[&[(&str, BinOp)]] = &[
      &[("||", BinOp::Or)],
      &[("&&", BinOp::And)],
      &[("|", BinOp::BitOr)],
      &[("^", BinOp::BitXor)],
      &[("&", BinOp::BitAnd)],
      &[("==", BinOp::Eq), ("!=", BinOp::Ne)],
      &[("<=", BinOp::Le), (">=", BinOp::Ge), ("<", BinOp::Lt), (">", BinOp::Gt)],
      &[("<<", BinOp::Shl), (">>", BinOp::Shr)],
      &[("+", BinOp::Add), ("-", BinOp::Sub)],
      &[("*", BinOp::Mul), ("/", BinOp::Div), ("%", BinOp::Mod)],
    ];

    if level >= LEVELS.len() {
      return self.parse_unary();
    }

    let mut left = self.parse_binary(level + 1)?;
    loop {
      let op = match self.peek() {
        Token::Op(found) => *found,
        _ => break,
      };

      // Match the operator against this level, longest first within the level.
      let matched = LEVELS[level]
        .iter()
        .find(|(candidate, _)| *candidate == op)
        .map(|(_, bin)| *bin);

      let Some(bin) = matched else { break };
      self.position += 1;
      let right = self.parse_binary(level + 1)?;
      left = Expr::Binary(bin, Box::new(left), Box::new(right));
    }

    Ok(left)
  }

  fn parse_unary(&mut self) -> Result<Expr, ParseError> {
    if self.eat_op("!") {
      return Ok(Expr::Not(Box::new(self.parse_unary()?)));
    }

    if self.eat_op("-") {
      return Ok(Expr::Neg(Box::new(self.parse_unary()?)));
    }

    if self.eat_op("++") {
      if let Token::Ident(name) = self.next() {
        return Ok(Expr::PreInc(name, 1));
      }
      return Err(ParseError::new("`++` needs a variable", self.position));
    }

    if self.eat_op("--") {
      if let Token::Ident(name) = self.next() {
        return Ok(Expr::PreInc(name, -1));
      }
      return Err(ParseError::new("`--` needs a variable", self.position));
    }

    self.parse_postfix()
  }

  fn parse_postfix(&mut self) -> Result<Expr, ParseError> {
    let mut expr = self.parse_primary()?;

    loop {
      if let Token::Ident(name) = self.peek().clone() {
        if (name == "true" || name == "false") && !matches!(expr, Expr::Var(_)) {
          self.position += 1;
          expr = Expr::Bool(name == "true");
          continue;
        }
      }

      if self.eat_op("++") {
        if let Expr::Var(name) = expr {
          expr = Expr::PostInc(name, 1);
          continue;
        }
        return Err(ParseError::new("`++` needs a variable", self.position));
      }

      if self.eat_op("--") {
        if let Expr::Var(name) = expr {
          expr = Expr::PostInc(name, -1);
          continue;
        }
        return Err(ParseError::new("`--` needs a variable", self.position));
      }

      break;
    }

    Ok(expr)
  }

  fn parse_primary(&mut self) -> Result<Expr, ParseError> {
    match self.next() {
      Token::Int(value) => Ok(Expr::Int(value)),
      Token::Float(value) => Ok(Expr::Float(value)),
      Token::Str(value) => Ok(Expr::Str(value)),
      Token::Ask => {
        let mut args = Vec::new();
        if !self.eat_op(")") {
          loop {
            args.push(self.parse_expression()?);
            if self.eat_op(",") {
              continue;
            }
            self.expect_op(")")?;
            break;
          }
        }
        Ok(Expr::Ask(args))
      }
      Token::Ident(name) => {
        if name == "true" || name == "false" {
          return Ok(Expr::Bool(name == "true"));
        }

        // A leading identifier may turn out to be a compound assignment.
        if let Token::Op(op) = self.peek().clone() {
          if matches!(op, "=" | "+=" | "-=" | "*=" | "/=" | "%=") {
            self.position += 1;
            let value = self.parse_expression()?;
            return Ok(Expr::Assign(name, Box::new(value)));
          }
        }

        if self.eat_op("(") {
          let mut args = Vec::new();
          if !self.eat_op(")") {
            loop {
              args.push(self.parse_expression()?);
              if self.eat_op(",") {
                continue;
              }
              self.expect_op(")")?;
              break;
            }
          }
          return Ok(Expr::Call(name, args));
        }

        Ok(Expr::Var(name))
      }
      Token::Op("(") => {
        let inner = self.parse_expression()?;
        self.expect_op(")")?;
        Ok(inner)
      }
      Token::Eof => Err(ParseError::new("expression ended early", self.position)),
      other => Err(ParseError::new(format!("unexpected token {other:?}"), self.position)),
    }
  }
}

pub fn parse(source: &str) -> Result<Expr, ParseError> {
  let tokens = tokenize(source)?;
  let mut parser = Parser {
    tokens,
    position: 0,
  };
  let expr = parser.parse_expression()?;
  if parser.peek() != &Token::Eof {
    return Err(ParseError::new("trailing tokens after expression", parser.position));
  }
  Ok(expr)
}

/// Whether this expression only reads story state.
///
/// This is the distinction the engine guide draws between conditions and
/// statements: a condition is re-run every time a hub is checked, while a
/// statement runs once when the hub is first covered.
pub fn reads_only(expr: &Expr) -> bool {
  !matches!(expr, Expr::Assign(..) | Expr::PreInc(..) | Expr::PostInc(..))
}

/// Which operations an expression performs.
///
/// See [`reads_only`], which is the same question asked of an expression that
/// has already been parsed. An expression that does not parse is reported as a
/// condition, so that whatever is asking falls back on its own rule for an
/// expression it cannot read.
pub fn classify(source: &str) -> ExpressionKind {
  match parse(source) {
    Ok(expr) if reads_only(&expr) => ExpressionKind::Condition,
    Ok(_) => ExpressionKind::Statement,
    Err(_) => ExpressionKind::Condition,
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
  Condition,
  Statement,
}

/// What happened while evaluating one expression.
pub enum Evaluation {
  /// The expression produced a value.
  Value(LouderVariant),
  /// The expression raised an ask and cannot finish until the host answers.
  Suspended,
}

godot_class! {
  /// Identifies who an expression is being evaluated for, so that story state,
  /// events, and asks can all be attributed correctly.
  #[derive(Debug, Clone, Default)]
  pub struct EvaluationScope {
    pub conversation: LouderName,
    pub actor: LouderActor,
    pub channel: LouderName,
    /// The motion that triggered this evaluation, if any.
    pub motion: Option<Motion>,
  }
}

impl EvaluationScope {
  pub fn new(conversation: LouderName, actor: LouderActor, channel: LouderName) -> EvaluationScope {
    EvaluationScope {
      conversation,
      actor,
      channel,
      motion: None,
    }
  }

  fn event(&self, data: EventData) -> LouderEvent {
    LouderEvent {
      conversation: self.conversation,
      channel: self.channel,
      actor: self.actor,
      data,
    }
  }
}

/// Functions callable from an expression.
///
/// The engine has no built-in behaviour for these: the host program supplies
/// them, which is what makes asks and game-specific queries possible without
/// the engine knowing anything about the game.
pub trait ExpressionHost {
  /// Handle a call to `name`. Return `None` to report an unknown function.
  fn call(&mut self, name: &str, args: &[LouderVariant]) -> Option<LouderVariant>;
}

impl ExpressionHost for LouderContext {
  fn call(&mut self, _name: &str, _args: &[LouderVariant]) -> Option<LouderVariant> {
    return None;
  }
}

godot_class! {
  /// Evaluates expressions against a loaded conversation.
  ///
  /// Evaluation is deliberately restartable. Asks suspend the whole expression,
  /// so an expression that raises an ask is evaluated again from the beginning
  /// once the verdict arrives. To make that safe, every write to story state is
  /// deferred until the expression has evaluated completely, so a retry cannot
  /// apply the same assignment twice.
  #[derive(Default, Clone)]
  pub struct Evaluator {
    scope: EvaluationScope,
  }
}

impl Evaluator {
  pub fn new(scope: EvaluationScope) -> Evaluator {
    Evaluator { scope }
  }

  pub fn scope(&self) -> &EvaluationScope {
    &self.scope
  }

  /// Evaluate one expression, recording conditions and statements as events.
  pub fn evaluate(
    &self,
    context: &mut LouderContext,
    source: &str,
  ) -> Result<Evaluation, ParseError> {
    let expr = parse(source)?;
    self.eval(context, &expr)
  }

  fn eval(
    &self,
    context: &mut LouderContext,
    expr: &Expr,
  ) -> Result<Evaluation, ParseError> {
    let node = expr;
    match node {
      Expr::Int(value) => Ok(Evaluation::Value(LouderVariant::Int(*value))),
      Expr::Float(value) => Ok(Evaluation::Value(LouderVariant::Float(*value))),
      Expr::Bool(value) => Ok(Evaluation::Value(LouderVariant::from_bool(*value))),
      Expr::Str(value) => {
        let name = context.intern_name(value);
        Ok(Evaluation::Value(LouderVariant::String(name)))
      }

      Expr::Var(name) => {
        let key = context.intern_name(name);
        let value = context.get_state(self.scope.conversation, key);
        Ok(Evaluation::Value(value))
      }

      Expr::Neg(inner) => match self.eval(context, inner)? {
        Evaluation::Suspended => Ok(Evaluation::Suspended),
        Evaluation::Value(value) => Ok(Evaluation::Value(LouderVariant::Int(-value.as_int()))),
      },

      Expr::Not(inner) => match self.eval(context, inner)? {
        Evaluation::Suspended => Ok(Evaluation::Suspended),
        Evaluation::Value(value) => {
          let truthy = self.truthy(context, &value);
          Ok(Evaluation::Value(LouderVariant::from_bool(!truthy)))
        }
      },

      Expr::Binary(op, lhs, rhs) => {
        let left = match self.eval(context, lhs)? {
          Evaluation::Suspended => return Ok(Evaluation::Suspended),
          Evaluation::Value(value) => value,
        };
        let right = match self.eval(context, rhs)? {
          Evaluation::Suspended => return Ok(Evaluation::Suspended),
          Evaluation::Value(value) => value,
        };
        Ok(Evaluation::Value(self.apply(context, *op, &left, &right)))
      }

      // The write happens only after the right hand side has fully evaluated,
      // so that a suspension retries without having already assigned.
      Expr::Assign(name, value) => {
        let computed = match self.eval(context, value)? {
          Evaluation::Suspended => return Ok(Evaluation::Suspended),
          Evaluation::Value(computed) => computed,
        };

        let key = context.intern_name(name);
        context.set_state(self.scope.conversation, key, computed.clone());
        let statement = EventData::Statement(key);
        context.push_event(self.scope.event(statement));
        Ok(Evaluation::Value(computed))
      }

      Expr::PreInc(name, step) => {
        let key = context.intern_name(name);
        let current = context.get_state(self.scope.conversation, key);
        let updated = current.arith(if *step > 0 { ArithOp::Add } else { ArithOp::Sub }, &LouderVariant::Int(step.abs()));
        context.set_state(self.scope.conversation, key, updated.clone());
        let statement = EventData::Statement(key);
        context.push_event(self.scope.event(statement));
        Ok(Evaluation::Value(updated))
      }

      Expr::PostInc(name, step) => {
        let key = context.intern_name(name);
        let current = context.get_state(self.scope.conversation, key);
        let updated = current.arith(if *step > 0 { ArithOp::Add } else { ArithOp::Sub }, &LouderVariant::Int(step.abs()));
        context.set_state(self.scope.conversation, key, updated);
        let statement = EventData::Statement(key);
        context.push_event(self.scope.event(statement));
        Ok(Evaluation::Value(current))
      }

      Expr::Call(name, args) => {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
          match self.eval(context, arg)? {
            Evaluation::Suspended => return Ok(Evaluation::Suspended),
            Evaluation::Value(value) => values.push(value),
          }
        }

        let resolved: Vec<LouderVariant> = values;
        match context.call(name, &resolved) {
          Some(value) => Ok(Evaluation::Value(value)),
          None => {
            // An unknown function evaluates to 0 rather than aborting the whole
            // conversation, matching how unset variables behave.
            Ok(Evaluation::Value(LouderVariant::Int(0)))
          }
        }
      }

      Expr::Ask(args) => {
        let mut values = Vec::with_capacity(args.len());
        for arg in args {
          match self.eval(context, arg)? {
            Evaluation::Suspended => return Ok(Evaluation::Suspended),
            Evaluation::Value(value) => values.push(value),
          }
        }

        let callback = context.intern_name(&args_to_text(args));
        let motion = self.scope.motion.clone().unwrap_or(Motion {
          source_position: super::hub::LouderPosition::MIN,
          target_position: super::hub::LouderPosition::MIN,
          choice_index: -1,
          background: false,
        });

        context.push_ask(LouderAsk {
          conversation: self.scope.conversation,
          channel: self.scope.channel,
          actor: self.scope.actor,
          motion,
          ask: callback,
        });

        Ok(Evaluation::Suspended)
      }
    }
  }

  fn truthy(&self, context: &LouderContext, value: &LouderVariant) -> bool {
    let text = match value {
      LouderVariant::String(name) => context.resolve_name(*name),
      _ => None,
    };
    value.is_truthy(text)
  }

  fn apply(&self, context: &mut LouderContext, op: BinOp, left: &LouderVariant, right: &LouderVariant) -> LouderVariant {
    // `+` concatenates when either side is a string, which is what lets text be
    // built up inside expressions. This is the one operator the two evaluation
    // passes cannot share, because building a string means interning it.
    if op == BinOp::Add && (left.kind() == VariantKind::String || right.kind() == VariantKind::String) {
      return self.intern(context, &join_strings(context, left, right));
    }

    apply_binary(context, op, left, right)
  }

  fn intern(&self, context: &mut LouderContext, text: &str) -> LouderVariant {
    let name = context.intern_name(text);
    LouderVariant::String(name)
  }
}

/// The text of `left + right` where either side is a string.
fn join_strings(context: &LouderContext, left: &LouderVariant, right: &LouderVariant) -> String {
  let mut joined = String::new();
  for operand in [left, right] {
    match operand {
      LouderVariant::String(name) => {
        if let Some(text) = context.resolve_name(*name) {
          joined.push_str(text);
        }
      }
      other => joined.push_str(&other.as_int().to_string()),
    }
  }
  joined
}

/// Every binary operator except `+` on two strings.
///
/// Both evaluation passes come through here, so an operator cannot mean one
/// thing while a hub is being checked for visibility and another while an actor
/// is travelling onto it.
fn apply_binary(
  context: &LouderContext,
  op: BinOp,
  left: &LouderVariant,
  right: &LouderVariant,
) -> LouderVariant {
  // Equality between two strings compares their text; every other comparison
  // falls back to the numeric rules.
  if matches!(op, BinOp::Eq | BinOp::Ne) && left.kind() == VariantKind::String && right.kind() == VariantKind::String {
    let a = context.resolve_name(string_id(left)).unwrap_or("");
    let b = context.resolve_name(string_id(right)).unwrap_or("");
    let equal = a == b;
    return LouderVariant::from_bool(if op == BinOp::Eq { equal } else { !equal });
  }

  // The guide's truthiness rules: 0, false and the empty string are all falsy.
  let truthy = |value: &LouderVariant| {
    let text = match value {
      LouderVariant::String(name) => context.resolve_name(*name),
      _ => None,
    };
    value.is_truthy(text)
  };

  match op {
    BinOp::Add => left.arith(ArithOp::Add, right),
    BinOp::Sub => left.arith(ArithOp::Sub, right),
    BinOp::Mul => left.arith(ArithOp::Mul, right),
    BinOp::Div => left.arith(ArithOp::Div, right),
    BinOp::Mod => left.arith(ArithOp::Mod, right),
    BinOp::Eq => LouderVariant::from_bool(left.cmp_numeric(CmpOp::Eq, right)),
    BinOp::Ne => LouderVariant::from_bool(left.cmp_numeric(CmpOp::Ne, right)),
    BinOp::Lt => LouderVariant::from_bool(left.cmp_numeric(CmpOp::Lt, right)),
    BinOp::Le => LouderVariant::from_bool(left.cmp_numeric(CmpOp::Le, right)),
    BinOp::Gt => LouderVariant::from_bool(left.cmp_numeric(CmpOp::Gt, right)),
    BinOp::Ge => LouderVariant::from_bool(left.cmp_numeric(CmpOp::Ge, right)),
    BinOp::And => LouderVariant::from_bool(truthy(left) && truthy(right)),
    BinOp::Or => LouderVariant::from_bool(truthy(left) || truthy(right)),
    BinOp::BitAnd => LouderVariant::Int(left.as_int() & right.as_int()),
    BinOp::BitOr => LouderVariant::Int(left.as_int() | right.as_int()),
    BinOp::BitXor => LouderVariant::Int(left.as_int() ^ right.as_int()),
    BinOp::Shl => LouderVariant::Int(left.as_int().wrapping_shl(right.as_int() as u32)),
    BinOp::Shr => LouderVariant::Int(left.as_int().wrapping_shr(right.as_int() as u32)),
  }
}

/// Evaluate a condition without touching the engine.
///
/// Conditions are the read-only half of an expression set: by construction
/// they contain no assignments, so they can be evaluated against a shared
/// context. That matters because the FFI exposes choice and hub queries through
/// a `const` handle, and asking "what can this actor see" must not be able to
/// change the conversation.
///
/// Anything that would need to mutate resolves to 0: an ask has no verdict
/// yet, and an unknown function has no implementation.
///
/// The result is the condition's own value, not its truthiness, so that this
/// can be used for nested subexpressions. Use [`condition_holds`] for the
/// yes-or-no answer.
pub fn eval_condition(context: &LouderContext, conversation: LouderName, expr: &Expr) -> LouderVariant {
  match expr {
    Expr::Int(value) => LouderVariant::Int(*value),
    Expr::Float(value) => LouderVariant::Float(*value),
    Expr::Bool(value) => LouderVariant::from_bool(*value),
    Expr::Str(value) => LouderVariant::String(lookup(context, value)),
    Expr::Var(name) => context.get_state(conversation, lookup(context, name)),

    Expr::Neg(inner) => {
      let value = eval_condition(context, conversation, inner);
      LouderVariant::Int(-value.as_int())
    }
    Expr::Not(inner) => {
      let value = eval_condition(context, conversation, inner);
      let text = string_text(context, &value);
      LouderVariant::from_bool(!value.is_truthy(text.as_deref()))
    }

    Expr::Binary(op, lhs, rhs) => {
      let left = eval_condition(context, conversation, lhs);
      let right = eval_condition(context, conversation, rhs);
      apply_readonly(context, *op, &left, &right)
    }

    // A statement in a condition position cannot happen: classification sends
    // these to the statement pass, so treating them as their own value is a
    // safety net rather than a real path.
    Expr::Assign(_, value) => eval_condition(context, conversation, value),
    Expr::PreInc(name, _) | Expr::PostInc(name, _) => {
      context.get_state(conversation, lookup(context, name))
    }

    Expr::Call(_, args) => {
      // The engine ships no built-in functions, and a read-only pass has
      // nowhere to route them, so a call contributes nothing here.
      let _ = args;
      LouderVariant::Int(0)
    }

    // An ask has no verdict while a decision is still being made.
    Expr::Ask(_) => LouderVariant::Int(0),
  }
}

/// Whether a condition holds, applying the truthiness rules of the engine
/// guide: 0, false, and the empty string are all falsy.
pub fn condition_holds(context: &LouderContext, conversation: LouderName, expr: &Expr) -> bool {
  let value = eval_condition(context, conversation, expr);
  let text = string_text(context, &value);
  value.is_truthy(text.as_deref())
}

fn string_text(context: &LouderContext, value: &LouderVariant) -> Option<String> {
  match value {
    LouderVariant::String(name) => context.resolve_name(*name).map(|t| t.to_string()),
    _ => None,
  }
}

fn lookup(context: &LouderContext, text: &str) -> LouderName {
  // A literal that was never interned cannot equal any stored value, so
  // [`LOUDER_NAME_NONE`] stands in for it. Empty strings intern to the same
  // sentinel, which keeps `x == ""` working.
  match context.find_name(&LouderNameContent::new(text)) {
    LouderContextResult::Found(name) => name,
    LouderContextResult::NotFound(_) => LOUDER_NAME_NONE,
  }
}

fn apply_readonly(
  context: &LouderContext,
  op: BinOp,
  left: &LouderVariant,
  right: &LouderVariant,
) -> LouderVariant {
  if op == BinOp::Add && (left.kind() == VariantKind::String || right.kind() == VariantKind::String) {
    // A read-only pass cannot intern a freshly built string, so the concatenation
    // resolves to the name-less string, which compares equal only to itself.
    // Conditions that depend on built text are evaluated for real during
    // travel, where interning is available.
    let _ = join_strings(context, left, right);
    return LouderVariant::String(LOUDER_NAME_NONE);
  }

  apply_binary(context, op, left, right)
}

fn string_id(value: &LouderVariant) -> LouderName {
  match value {
    LouderVariant::String(name) => *name,
    _ => LOUDER_NAME_NONE,
  }
}

/// Reconstruct the text of an ask's arguments for the host to read back.
fn args_to_text(args: &[Expr]) -> String {
  args.iter().map(expr_to_text).collect::<Vec<String>>().join(" ")
}

fn expr_to_text(expr: &Expr) -> String {
  match expr {
    Expr::Int(value) => value.to_string(),
    Expr::Float(value) => value.to_string(),
    Expr::Bool(value) => value.to_string(),
    Expr::Str(value) => value.clone(),
    Expr::Var(name) => name.clone(),
    Expr::Ask(args) => format!("($({}))", args_to_text(args)),
    Expr::Neg(inner) => format!("-{}", expr_to_text(inner)),
    Expr::Not(inner) => format!("!{}", expr_to_text(inner)),
    Expr::Call(name, args) => {
      format!("{}({})", name, args.iter().map(expr_to_text).collect::<Vec<String>>().join(", "))
    }
    Expr::Binary(op, lhs, rhs) => format!(
      "({} {} {})",
      expr_to_text(lhs),
      bin_op_text(*op),
      expr_to_text(rhs)
    ),
    Expr::Assign(name, value) => format!("{name} = {}", expr_to_text(value)),
    // Pre versus post, and up versus down, are separate facts, so both are
    // rendered: the node carries one as its variant and the other as the sign of
    // its step.
    Expr::PreInc(name, step) => format!("{}{name}", step_text(*step)),
    Expr::PostInc(name, step) => format!("{name}{}", step_text(*step)),
  }
}

fn step_text(step: i32) -> &'static str {
  if step < 0 {
    "--"
  } else {
    "++"
  }
}

fn bin_op_text(op: BinOp) -> &'static str {
  match op {
    BinOp::Add => "+",
    BinOp::Sub => "-",
    BinOp::Mul => "*",
    BinOp::Div => "/",
    BinOp::Mod => "%",
    BinOp::Eq => "==",
    BinOp::Ne => "!=",
    BinOp::Lt => "<",
    BinOp::Le => "<=",
    BinOp::Gt => ">",
    BinOp::Ge => ">=",
    BinOp::And => "&&",
    BinOp::Or => "||",
    BinOp::BitAnd => "&",
    BinOp::BitOr => "|",
    BinOp::BitXor => "^",
    BinOp::Shl => "<<",
    BinOp::Shr => ">>",
  }
}

godot_api! {
  impl ParseError {
    /// What went wrong, in the engine's own words.
    #[func]
    pub fn get_message(&self) -> GString {
      GString::from(self.message.as_str())
    }

    /// How far into the expression the engine got before giving up.
    #[func]
    pub fn get_offset(&self) -> i32 {
      self.offset as i32
    }
  }
}

godot_api! {
  impl EvaluationScope {
    /// A scope attributing events and asks to `actor` in `conversation`.
    #[func]
    pub fn create(conversation: LouderName, actor: LouderActor, channel: LouderName) -> Gd<EvaluationScope> {
      Gd::from_object(EvaluationScope::new(conversation, actor, channel))
    }

    /// The conversation story state is read from.
    #[func]
    pub fn get_conversation(&self) -> LouderName {
      self.conversation
    }

    /// The actor this evaluation is for.
    #[func]
    pub fn get_actor(&self) -> LouderActor {
      self.actor
    }

    /// The channel this evaluation is for.
    #[func]
    pub fn get_channel(&self) -> LouderName {
      self.channel
    }

    /// The move that triggered this evaluation, if any.
    #[func]
    pub fn get_motion(&self) -> Option<Gd<Motion>> {
      self.motion.as_ref().map(|motion| Gd::from_object(motion.clone()))
    }

    /// Record the move that triggered this evaluation.
    #[func(rename = "set_motion")]
    pub fn gd_set_motion(&mut self, motion: Gd<Motion>) {
      self.motion = Some(motion.bind().clone());
    }
  }
}

godot_api! {
  impl Evaluator {
    /// An evaluator that attributes everything it produces to `scope`.
    #[func]
    pub fn create(scope: Gd<EvaluationScope>) -> Gd<Evaluator> {
      Gd::from_object(Evaluator::new(scope.bind().clone()))
    }

    /// Who this evaluator attributes its events and asks to.
    #[func]
    pub fn get_scope(&self) -> Gd<EvaluationScope> {
      Gd::from_object(self.scope().clone())
    }

    /// Evaluate `source`, reporting what happened.
    ///
    /// Returns 0 for a value, 1 if the expression raised an ask and is waiting
    /// on the host, and 2 if it could not be read. Use the `evaluate_*` methods
    /// for the value itself.
    #[func(rename = "evaluate")]
    pub fn gd_evaluate(&self, context: Gd<LouderContext>, source: GString) -> i32 {
      let mut context = context;
      let mut context = context.bind_mut();
      match self.evaluate(&mut context, &source.to_string()) {
        Ok(Evaluation::Value(_)) => 0,
        Ok(Evaluation::Suspended) => 1,
        Err(_) => 2,
      }
    }

    /// Evaluate `source` and read the result as an integer, or 0.
    #[func]
    pub fn evaluate_int(&self, context: Gd<LouderContext>, source: GString) -> i32 {
      let mut context = context;
      let mut context = context.bind_mut();
      match self.evaluate(&mut context, &source.to_string()) {
        Ok(Evaluation::Value(value)) => value.as_int(),
        _ => 0,
      }
    }

    /// Evaluate `source` and read the result as a float, or 0.0.
    #[func]
    pub fn evaluate_float(&self, context: Gd<LouderContext>, source: GString) -> f64 {
      let mut context = context;
      let mut context = context.bind_mut();
      match self.evaluate(&mut context, &source.to_string()) {
        Ok(Evaluation::Value(value)) => value.as_float() as f64,
        _ => 0.0,
      }
    }

    /// Evaluate `source` and read the result as a name, or
    /// [`LOUDER_NAME_NONE`].
    #[func]
    pub fn evaluate_name(&self, context: Gd<LouderContext>, source: GString) -> LouderName {
      let mut context = context;
      let mut context = context.bind_mut();
      match self.evaluate(&mut context, &source.to_string()) {
        Ok(Evaluation::Value(LouderVariant::String(name))) => name,
        _ => LOUDER_NAME_NONE,
      }
    }

    /// What went wrong reading `source`, if anything did.
    #[func]
    pub fn parse(source: GString) -> Option<Gd<ParseError>> {
      match parse(&source.to_string()) {
        Ok(_) => None,
        Err(error) => Some(Gd::from_object(error)),
      }
    }

    /// Whether `source` is a condition or a statement, as an
    /// [`ExpressionKind`] index.
    #[func]
    pub fn classify(source: GString) -> i32 {
      match classify(&source.to_string()) {
        ExpressionKind::Condition => 0,
        ExpressionKind::Statement => 1,
      }
    }

    /// Whether `source` only reads story state, and so can be re-run freely.
    #[func]
    pub fn reads_only(source: GString) -> bool {
      match parse(&source.to_string()) {
        Ok(expression) => reads_only(&expression),
        Err(_) => true,
      }
    }

    /// The tokens of `source`, as text, with the end of input as a final empty
    /// entry. Useful for reporting where an expression stopped making sense.
    #[func]
    pub fn tokenize(source: GString) -> Array<GString> {
      match tokenize(&source.to_string()) {
        Ok(tokens) => tokens
          .iter()
          .map(|token| GString::from(format!("{token:?}").as_str()))
          .collect(),
        Err(_) => Array::new(),
      }
    }

    /// Whether the condition `source` holds, without changing anything.
    ///
    /// An ask inside a condition contributes 0, since it has no verdict yet,
    /// and an unknown function contributes 0 too.
    #[func]
    pub fn condition_holds(context: Gd<LouderContext>, conversation: LouderName, source: GString) -> bool {
      let Ok(expression) = parse(&source.to_string()) else {
        return false;
      };
      let context = context.bind();
      condition_holds(&context, conversation, &expression)
    }
  }
}
