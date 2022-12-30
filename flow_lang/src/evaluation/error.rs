use crate::parsing::{LispType, LispVal, error::LispValUnwrapError};

use super::scope::Scope;

#[derive(Debug)]
pub enum EvalError {
    InvalidArgumentsCount {
        name: String,
        expected: usize,
        got: usize,
    },
    InvalidArgumentType {
        name: String,
        expected: LispType,
        got: LispType,
        position: usize,
    },
    InvalidConcatenation {
        left: LispType,
        right: LispType,
    },
    InvalidFunctionCall {
        values: Vec<LispVal>,
    },
    UnknownIdentifier(String),
    ListOverflow {
        access: usize,
        count: usize,
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::InvalidArgumentsCount { expected, got, name } => {
                if got < expected {
                    return write!(
                        f,
                        "Too few arguments for `{}`, expected `{}`, got `{}`",
                        name, expected, got
                    );
                } else if got > expected {
                    return write!(
                        f,
                        "Too many arguments for `{}`, expected `{}`, got `{}`",
                        name, expected, got
                    );
                } else {
                    unreachable!();
                }
            }
            EvalError::InvalidArgumentType {
                name,
                expected,
                got,
                position,
            } => write!(
                f,
                "Invalid argument type for `{}` at position `{}`, expected `{}`, got `{}`",
                name, position, expected, got
            ),
            EvalError::InvalidConcatenation { left, right } => write!(
                f,
                "Invalid argument types, cannot concat `{}` and `{}`",
                left, right
            ),
            EvalError::InvalidFunctionCall { values } => {
                let correct_expr = LispVal::Unevaluated(Box::new(LispVal::List(values.clone())));
                let head = values.get(0).unwrap();
                write!(f, "Invalid function call, expected identifier, got `{}`. \nIs this supposed to be a list? If so, use `{}`", head, correct_expr)
            }
            EvalError::UnknownIdentifier(identifier) => {
                write!(f, "Unknown identifier `{}`", identifier)
            }
            EvalError::ListOverflow { access, count } => write!(
                f,
                "List overflow, tried to access `{}` in list of length `{}`",
                access, count
            ),
        }
    }
}

impl std::error::Error for EvalError {}

impl EvalError {
    pub fn from(e: LispValUnwrapError, position: usize, scope: &Scope) -> Self {
        EvalError::InvalidArgumentType {
            name: scope.context.clone(),
            expected: e.expected,
            got: e.got,
            position,
        }
    }
}