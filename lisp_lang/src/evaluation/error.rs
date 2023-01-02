use crate::parsing::{error::LispValUnwrapError, LispType, LispVal};

#[derive(Debug)]
pub enum EvalError {
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
}


impl std::error::Error for EvalError {}

impl EvalError {
    pub fn from_arg<'a>(position: usize, name: &'a str) -> impl Fn(LispValUnwrapError) -> Self + 'a {
        move |e| EvalError::InvalidArgumentType {
            name: name.to_string(),
            expected: e.expected,
            got: e.got,
            position,
        }
    }
}
