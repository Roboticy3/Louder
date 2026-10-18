//! Reading, writing, and running the expressions attached to hubs.

use crate::context::LouderContext;
use crate::engine;
use crate::event::Motion;
use louder_rust::godot_api;

use godot::prelude::*;

/// An expression the engine could not read.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct ParseError {
  pub(crate) inner: engine::ParseError,
}

godot_api! {
  impl ParseError {
    /// What went wrong, in the engine's own words.
    #[func]
    pub fn get_message(&self) -> GString {
      GString::from(self.inner.message.as_str())
    }

    /// How far into the expression the engine got before giving up.
    #[func]
    pub fn get_offset(&self) -> i32 {
      self.inner.offset as i32
    }
  }
}

/// Identifies who an expression is being evaluated for, so that story state,
/// events, and asks can all be attributed correctly.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct EvaluationScope {
  pub(crate) inner: engine::EvaluationScope,
}

godot_api! {
  impl EvaluationScope {
    /// A scope attributing events and asks to `actor` in `conversation`.
    #[func]
    pub fn create(
      conversation: engine::LouderName,
      actor: engine::LouderActor,
      channel: engine::LouderName,
    ) -> Gd<EvaluationScope> {
      Gd::from_object(EvaluationScope {
        inner: engine::EvaluationScope::new(conversation, actor, channel),
      })
    }

    /// The conversation story state is read from.
    #[func]
    pub fn get_conversation(&self) -> engine::LouderName {
      self.inner.conversation
    }

    /// The actor this evaluation is for.
    #[func]
    pub fn get_actor(&self) -> engine::LouderActor {
      self.inner.actor
    }

    /// The channel this evaluation is for.
    #[func]
    pub fn get_channel(&self) -> engine::LouderName {
      self.inner.channel
    }

    /// The move that triggered this evaluation, if any.
    #[func]
    pub fn get_motion(&self) -> Option<Gd<Motion>> {
      self.inner.motion.as_ref().map(|motion| Gd::from_object(Motion { inner: motion.clone() }))
    }

    /// Record the move that triggered this evaluation.
    #[func]
    pub fn set_motion(&mut self, motion: Gd<Motion>) {
      self.inner.motion = Some(motion.bind().inner.clone());
    }
  }
}

/// Evaluates expressions against a loaded conversation.
///
/// Evaluation is deliberately restartable. Asks suspend the whole expression,
/// so an expression that raises an ask is evaluated again from the beginning
/// once the verdict arrives. To make that safe, every write to story state is
/// deferred until the expression has evaluated completely, so a retry cannot
/// apply the same assignment twice.
#[derive(Clone, Default)]
#[derive(GodotClass)]
#[class(init, tool)]
pub struct Evaluator {
  pub(crate) inner: engine::Evaluator,
}

godot_api! {
  impl Evaluator {
    /// An evaluator that attributes everything it produces to `scope`.
    #[func]
    pub fn create(scope: Gd<EvaluationScope>) -> Gd<Evaluator> {
      Gd::from_object(Evaluator {
        inner: engine::Evaluator::new(scope.bind().inner.clone()),
      })
    }

    /// Who this evaluator attributes its events and asks to.
    #[func]
    pub fn get_scope(&self) -> Gd<EvaluationScope> {
      Gd::from_object(EvaluationScope { inner: self.inner.scope().clone() })
    }

    /// Evaluate `source`, reporting what happened.
    ///
    /// Returns 0 for a value, 1 if the expression raised an ask and is waiting
    /// on the host, and 2 if it could not be read. Use the `evaluate_*` methods
    /// for the value itself.
    #[func]
    pub fn evaluate(&self, mut context: Gd<LouderContext>, source: GString) -> i32 {
      match self.inner.evaluate(&mut context.bind_mut().inner, &source.to_string()) {
        Ok(engine::Evaluation::Value(_)) => 0,
        Ok(engine::Evaluation::Suspended) => 1,
        Err(_) => 2,
      }
    }

    /// Evaluate `source` and read the result as an integer, or 0.
    #[func]
    pub fn evaluate_int(&self, mut context: Gd<LouderContext>, source: GString) -> i32 {
      match self.inner.evaluate(&mut context.bind_mut().inner, &source.to_string()) {
        Ok(engine::Evaluation::Value(value)) => value.as_int(),
        _ => 0,
      }
    }

    /// Evaluate `source` and read the result as a float, or 0.0.
    #[func]
    pub fn evaluate_float(&self, mut context: Gd<LouderContext>, source: GString) -> f64 {
      match self.inner.evaluate(&mut context.bind_mut().inner, &source.to_string()) {
        Ok(engine::Evaluation::Value(value)) => value.as_float() as f64,
        _ => 0.0,
      }
    }

    /// Evaluate `source` and read the result as a name, or
    /// [`LOUDER_NAME_NONE`](engine::LOUDER_NAME_NONE).
    #[func]
    pub fn evaluate_name(&self, mut context: Gd<LouderContext>, source: GString) -> engine::LouderName {
      match self.inner.evaluate(&mut context.bind_mut().inner, &source.to_string()) {
        Ok(engine::Evaluation::Value(engine::LouderVariant::String(name))) => name,
        _ => engine::LOUDER_NAME_NONE,
      }
    }

    /// What went wrong reading `source`, if anything did.
    #[func]
    pub fn parse(source: GString) -> Option<Gd<ParseError>> {
      match engine::parse(&source.to_string()) {
        Ok(_) => None,
        Err(error) => Some(Gd::from_object(ParseError { inner: error })),
      }
    }

    /// Whether `source` is a condition or a statement, as an
    /// [`ExpressionKind`](engine::ExpressionKind) index.
    #[func]
    pub fn classify(source: GString) -> i32 {
      match engine::classify(&source.to_string()) {
        engine::ExpressionKind::Condition => 0,
        engine::ExpressionKind::Statement => 1,
      }
    }

    /// Whether `source` only reads story state, and so can be re-run freely.
    #[func]
    pub fn reads_only(source: GString) -> bool {
      match engine::parse(&source.to_string()) {
        Ok(expression) => engine::reads_only(&expression),
        Err(_) => true,
      }
    }

    /// The tokens of `source`, as text, with the end of input as a final empty
    /// entry. Useful for reporting where an expression stopped making sense.
    #[func]
    pub fn tokenize(source: GString) -> Array<GString> {
      match engine::tokenize(&source.to_string()) {
        Ok(tokens) => tokens.iter().map(|token| GString::from(format!("{token:?}").as_str())).collect(),
        Err(_) => Array::new(),
      }
    }

    /// Whether the condition `source` holds, without changing anything.
    ///
    /// An ask inside a condition contributes 0, since it has no verdict yet,
    /// and an unknown function contributes 0 too.
    #[func]
    pub fn condition_holds(
      context: Gd<LouderContext>,
      conversation: engine::LouderName,
      source: GString,
    ) -> bool {
      let Ok(expression) = engine::parse(&source.to_string()) else {
        return false;
      };
      engine::condition_holds(&context.bind().inner, conversation, &expression)
    }
  }
}