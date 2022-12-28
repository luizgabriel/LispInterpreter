use crate::parsing::{LispType, LispVal, error::LispValUnwrapError};

#[derive(Debug)]
pub enum EvalError {
    InvalidArgumentsCount {
        expected: usize,
        got: usize,
    },
    InvalidArgumentType {
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
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::InvalidArgumentsCount { expected, got } => {
                if got < expected {
                    return write!(
                        f,
                        "Too few arguments, expected `{}`, got `{}`",
                        expected, got
                    );
                } else if got > expected {
                    return write!(
                        f,
                        "Too many arguments, expected `{}`, got `{}`",
                        expected, got
                    );
                } else {
                    unreachable!();
                }
            }
            EvalError::InvalidArgumentType {
                expected,
                got,
                position,
            } => write!(
                f,
                "Invalid argument type at position `{}`, expected `{}`, got `{}`",
                position, expected, got
            ),
            EvalError::InvalidConcatenation { left, right } => write!(
                f,
                "Invalid argument types, cannot concat `{}` and `{}`",
                left, right
            ),
            EvalError::InvalidFunctionCall { values } => {
                let correct_expr = LispVal::Unevaluated(Box::new(LispVal::List(values.clone())));
                let head = values.first().unwrap();
                write!(f, "Invalid function call, expected identifier, got `{}`. \nIs this supposed to be a list? If so, use `{}`", head, correct_expr)
            }
            EvalError::UnknownIdentifier(identifier) => {
                write!(f, "Unknown identifier `{}`", identifier)
            }
        }
    }
}

impl std::error::Error for EvalError {}

impl EvalError {
    pub fn from(e: LispValUnwrapError, position: usize) -> Self {
        EvalError::InvalidArgumentType {
            expected: e.expected,
            got: e.got,
            position,
        }
    }
}