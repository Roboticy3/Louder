use super::context::LouderContext;
use super::event::*;
use super::name::*;
use super::actor::LouderActor;
use super::hub::LouderPosition;
use super::rule::LouderRule;
use super::object::{ArithOp, CmpOp, LouderVariant, VariantKind};

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

  /// A special variable, named by its source text.
  ///
  /// Special variables are written with a `%` in front, and are read from the
  /// engine rather than from story state. The name keeps its `%`, so that a
  /// special variable can be told from an ordinary one by looking at it.
  Special(String),

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

/// The name of the special variable that says who speaks a hub.
pub const SPEAKER_VARIABLE: &str = "$speaker";

/// Whether a name is a special variable rather than a story state one.
///
/// Special variables are written with a `%` in front, and that is what tells
/// them apart from the variables an author defines.
pub fn is_special(name: &str) -> bool {
  name.starts_with('%') || name.starts_with('$')
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
  /// A special variable, with its `%` still attached.
  Special(String),
  Op(&'static str),
  Ask,
  Eof,
}

/// An expression the engine could not read.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParseError {
  pub message: String,
  pub offset: usize,
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

    // A `$` opens a special identifier, as the engine guide describes.
    // The old `$(...)` ask syntax is out of date and no longer supported.
    if c == '$' {
      let start = i;
      i += 1;
      if i >= bytes.len() || !(bytes[i] == '_' || bytes[i].is_alphabetic()) {
        return Err(ParseError::new("expected an identifier after `$`", start));
      }
      let name_start = i;
      while i < bytes.len() && (bytes[i] == '_' || bytes[i].is_alphanumeric()) {
        i += 1;
      }
      let name: String = bytes[name_start..i].iter().collect();
      tokens.push(Token::Special(format!("${name}")));
      continue;
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

/// Whether a token ends an operand, and so can be followed by an operator.
///
/// This is what tells a `%` that opens a special variable from one that is the
/// modulo operator: the operator follows an operand, and a special variable
/// does not.
fn ends_operand(token: Option<&Token>) -> bool {
  matches!(
    token,
    Some(Token::Ident(_))
      | Some(Token::Int(_))
      | Some(Token::Float(_))
      | Some(Token::Str(_))
      | Some(Token::Special(_))
      | Some(Token::Op(")"))
  )
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
      if let Token::Ident(name) | Token::Special(name) = self.next() {
        return Ok(Expr::PreInc(name, 1));
      }
      return Err(ParseError::new("`++` needs a variable", self.position));
    }

    if self.eat_op("--") {
      if let Token::Ident(name) | Token::Special(name) = self.next() {
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
        if let Expr::Var(name) | Expr::Special(name) = expr {
          expr = Expr::PostInc(name, 1);
          continue;
        }
        return Err(ParseError::new("`++` needs a variable", self.position));
      }

      if self.eat_op("--") {
        if let Expr::Var(name) | Expr::Special(name) = expr {
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

        Ok(Expr::Call(name, Vec::new()))
      }
      Token::Special(name) => {
        // A special variable may turn out to be a write to it.
        if let Token::Op(op) = self.peek().clone()
          && matches!(op, "=" | "+=" | "-=" | "*=" | "/=" | "%=")
        {
          self.position += 1;
          let value = self.parse_expression()?;
          return Ok(Expr::Assign(name, Box::new(value)));
        }

        Ok(Expr::Special(name))
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
  !matches!(
    expr,
    Expr::Assign(..) | Expr::PreInc(..) | Expr::PostInc(..) | Expr::Call(..) | Expr::Ask(..)
  )
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

/// Identifies who an expression is being evaluated for, so that story state,
/// events, and asks can all be attributed correctly.
#[derive(Debug, Clone, Default)]
pub struct EvaluationScope {
  pub conversation: LouderName,
  pub actor: LouderActor,
  pub channel: LouderName,
  /// The hub the evaluation is about.
  ///
  /// A special variable is read from the hub in hand, so that `%speaker` says
  /// who would speak *this* hub rather than whichever one comes first.
  pub hub: LouderPosition,
  /// The motion that triggered this evaluation, if any.
  pub motion: Option<Motion>,

  /// The rule tagging events produced by this evaluation.
  pub rule: LouderRule,
}

impl EvaluationScope {
  pub fn new(conversation: LouderName, actor: LouderActor, channel: LouderName) -> EvaluationScope {
    EvaluationScope {
      conversation,
      actor,
      channel,
      hub: super::hub::BLANK_POSITION,
      motion: None,
      rule: LouderRule::NONE,
    }
  }

  fn event(&self, data: EventData) -> LouderEvent {
    LouderEvent {
      conversation: self.conversation,
      channel: self.channel,
      actor: self.actor,
      rule: self.rule,
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

  /// Evaluate one expression that has already been parsed.
  pub fn evaluate_expr(
    &self,
    context: &mut LouderContext,
    expr: &Expr,
  ) -> Result<Evaluation, ParseError> {
    self.eval(context, expr)
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

      Expr::Special(name) => Ok(Evaluation::Value(self.special_value(context, name))),

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

        // A special variable is not story state. The engine applies a write to
        // `%speaker` to the move that made it, and the hub keeps the identity it
        // had, so the write itself changes nothing here.
        if is_special(name) {
          return Ok(Evaluation::Value(computed));
        }

        let key = context.intern_name(name);
        context.set_state(self.scope.conversation, key, computed.clone());
        let statement = EventData::Statement(key);
        context.push_event(self.scope.event(statement));
        Ok(Evaluation::Value(computed))
      }

      Expr::PreInc(name, step) => {
        // A special variable is read from the engine, and there is nothing to
        // count, so the increment is worth nothing.
        if is_special(name) {
          return Ok(Evaluation::Value(self.special_value(context, name)));
        }

        let key = context.intern_name(name);
        let current = context.get_state(self.scope.conversation, key);
        let updated = current.arith(if *step > 0 { ArithOp::Add } else { ArithOp::Sub }, &LouderVariant::Int(step.abs()));
        context.set_state(self.scope.conversation, key, updated.clone());
        let statement = EventData::Statement(key);
        context.push_event(self.scope.event(statement));
        Ok(Evaluation::Value(updated))
      }

      Expr::PostInc(name, step) => {
        if is_special(name) {
          return Ok(Evaluation::Value(self.special_value(context, name)));
        }

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

        // An identifier is resolved by the host through an interrupt. The
        // callback carries the function name and the literals its arguments
        // resolved to.
        let callback_text = if values.is_empty() {
          name.clone()
        } else {
          format!(
            "{}({})",
            name,
            values
              .iter()
              .map(|value| variant_literal(context, value))
              .collect::<Vec<String>>()
              .join(", ")
          )
        };
        let callback = context.intern_name(&callback_text);

        for index in 0..context.ask_count() {
          let Some(existing) = context.ask(index) else {
            continue;
          };
          if existing.ask != callback
            || existing.conversation != self.scope.conversation
            || existing.actor != self.scope.actor
            || existing.channel != self.scope.channel
          {
            continue;
          }
          if let Some(verdict) = context.ask_verdict(index) {
            return Ok(Evaluation::Value(LouderVariant::Int(verdict)));
          }
          return Ok(Evaluation::Suspended);
        }

        let motion = self.scope.motion.clone().unwrap_or(Motion {
          source_position: super::hub::LouderPosition::MIN,
          target_position: super::hub::LouderPosition::MIN,
          choice_index: -1,
          speaker: LOUDER_NAME_NONE,
          background: false,
          rule: LouderRule::NONE,
        });

        context.push_ask(LouderAsk {
          conversation: self.scope.conversation,
          channel: self.scope.channel,
          actor: self.scope.actor,
          rule: LouderRule::ASK,
          motion,
          ask: callback,
        });

        Ok(Evaluation::Suspended)
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

        // If this ask has already been answered, use the verdict instead of
        // suspending again. An unanswered copy of the same ask means we are
        // still waiting on the host.
        for index in 0..context.ask_count() {
          let Some(existing) = context.ask(index) else {
            continue;
          };
          if existing.ask != callback
            || existing.conversation != self.scope.conversation
            || existing.actor != self.scope.actor
            || existing.channel != self.scope.channel
          {
            continue;
          }
          if let Some(verdict) = context.ask_verdict(index) {
            return Ok(Evaluation::Value(LouderVariant::Int(verdict)));
          }
          return Ok(Evaluation::Suspended);
        }

        let motion = self.scope.motion.clone().unwrap_or(Motion {
          source_position: super::hub::LouderPosition::MIN,
          target_position: super::hub::LouderPosition::MIN,
          choice_index: -1,
          speaker: LOUDER_NAME_NONE,
          background: false,
          rule: LouderRule::NONE,
        });

        context.push_ask(LouderAsk {
          conversation: self.scope.conversation,
          channel: self.scope.channel,
          actor: self.scope.actor,
          rule: LouderRule::ASK,
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

  /// The value of a special variable, read from the engine.
  ///
  /// A special variable is not story state, so it is read from the hub and the
  /// actor in hand rather than from the state table.
  fn special_value(&self, context: &LouderContext, name: &str) -> LouderVariant {
    special_value(
      context,
      self.scope.conversation,
      self.scope.actor,
      self.scope.hub,
      name,
    )
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

/// The value of a special variable.
///
/// Special variables are read from the engine rather than from story state, so
/// they need to know who is asking and which hub is being asked about.
pub fn special_value(
  context: &LouderContext,
  conversation: LouderName,
  actor: LouderActor,
  hub: LouderPosition,
  name: &str,
) -> LouderVariant {
  match name {
    // The speaker that would speak this hub, seen from where the actor stands.
    SPEAKER_VARIABLE => {
      let speaker = super::simulator::Simulator::hub_speaker_for(context, conversation, actor, hub);
      LouderVariant::String(speaker)
    }
    // A special variable the engine does not know is worth nothing, which is
    // how an unset variable behaves.
    _ => LouderVariant::Int(0),
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
/// The actor and the hub are needed because a condition may ask the engine
/// about the hub it is attached to, as `%speaker` does.
///
/// Anything that would need to mutate resolves to 0: an ask has no verdict
/// yet, and an unknown function has no implementation.
///
/// The result is the condition's own value, not its truthiness, so that this
/// can be used for nested subexpressions. Use [`condition_holds`] for the
/// yes-or-no answer.
pub fn eval_condition(
  context: &LouderContext,
  conversation: LouderName,
  actor: LouderActor,
  hub: LouderPosition,
  expr: &Expr,
) -> LouderVariant {
  match expr {
    Expr::Int(value) => LouderVariant::Int(*value),
    Expr::Float(value) => LouderVariant::Float(*value),
    Expr::Bool(value) => LouderVariant::from_bool(*value),
    Expr::Str(value) => LouderVariant::String(lookup(context, value)),
    Expr::Var(name) => context.get_state(conversation, lookup(context, name)),
    Expr::Special(name) => special_value(context, conversation, actor, hub, name),

    Expr::Neg(inner) => {
      let value = eval_condition(context, conversation, actor, hub, inner);
      LouderVariant::Int(-value.as_int())
    }
    Expr::Not(inner) => {
      let value = eval_condition(context, conversation, actor, hub, inner);
      let text = string_text(context, &value);
      LouderVariant::from_bool(!value.is_truthy(text.as_deref()))
    }

    Expr::Binary(op, lhs, rhs) => {
      let left = eval_condition(context, conversation, actor, hub, lhs);
      let right = eval_condition(context, conversation, actor, hub, rhs);
      apply_readonly(context, *op, &left, &right)
    }

    // A statement in a condition position cannot happen: classification sends
    // these to the statement pass, so treating them as their own value is a
    // safety net rather than a real path.
    Expr::Assign(_, value) => eval_condition(context, conversation, actor, hub, value),
    Expr::PreInc(name, _) | Expr::PostInc(name, _) => {
      if is_special(name) {
        special_value(context, conversation, actor, hub, name)
      } else {
        context.get_state(conversation, lookup(context, name))
      }
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
pub fn condition_holds(
  context: &LouderContext,
  conversation: LouderName,
  actor: LouderActor,
  hub: LouderPosition,
  expr: &Expr,
) -> bool {
  let value = eval_condition(context, conversation, actor, hub, expr);
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

/// Whether an expression writes anything or asks anything.
///
/// The engine reads the value of a special variable before the move it belongs
/// to is announced, and an expression that writes or asks would then do so
/// twice: once there, and once when the hub's own statements run.
fn has_side_effects(expr: &Expr) -> bool {
  match expr {
    Expr::Assign(..) | Expr::PreInc(..) | Expr::PostInc(..) | Expr::Ask(_) | Expr::Call(..) => true,
    Expr::Neg(inner) | Expr::Not(inner) => has_side_effects(inner),
    Expr::Binary(_, lhs, rhs) => has_side_effects(lhs) || has_side_effects(rhs),
    Expr::Call(_, args) => args.iter().any(has_side_effects),
    _ => false,
  }
}

/// The special variable an expression writes to, and the value it writes.
///
/// The engine has one writable special variable, `%speaker`, and a write to it
/// belongs to the move that made it rather than to the hub: it changes who the
/// hub appears to speak as for that move, and leaves the hub itself alone.
///
/// The value is only read when it can be read without side effects, since the
/// write is evaluated before the move is announced and a second write to story
/// state would be counted twice.
pub fn special_write(
  context: &mut LouderContext,
  scope: &EvaluationScope,
  expr: &Expr,
) -> Option<(String, LouderVariant)> {
  let Expr::Assign(name, value) = expr else {
    return None;
  };

  if !is_special(name) || has_side_effects(value) {
    return None;
  }

  let evaluator = Evaluator::new(scope.clone());
  match evaluator.evaluate_expr(context, value) {
    Ok(Evaluation::Value(value)) => Some((name.clone(), value)),
    _ => None,
  }
}

/// Render a resolved value as the literal the host sees in an ask's callback.
fn variant_literal(context: &LouderContext, value: &LouderVariant) -> String {
  match value {
    LouderVariant::Int(value) => value.to_string(),
    LouderVariant::Float(value) => value.to_string(),
    LouderVariant::String(name) => context.resolve_name(*name).unwrap_or("").to_string(),
    other => other.as_int().to_string(),
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
    Expr::Special(name) => name.clone(),
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



