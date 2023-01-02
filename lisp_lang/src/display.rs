use std::fmt::Formatter;

use crate::{parsing::LispVal, evaluation::{scope::Scope, error::EvalError}};

impl std::fmt::Display for LispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LispVal::Void() => write!(f, "void"),
            LispVal::Symbol(atom) => write!(f, "{}", atom),
            LispVal::Number(n) => write!(f, "{}", n.to_string()),
            LispVal::String(s) => write!(f, "\"{}\"", s.to_string()),
            LispVal::Unevaluated(expr) => write!(f, "'{}", expr.to_string()),
            LispVal::Boolean(b) => write!(f, "{}", b.to_string()),
            LispVal::Function { parameters: args, body, applied } => {
                write!(f, "(fn '({}) '({}))", args.join(" "), body.to_string())?;
                if !applied.is_empty() {
                    write!(f, ", {}", std::convert::Into::<LispVal>::into(applied.clone()))?;
                }
                Ok(())
            }
            LispVal::List(values) => write!(
                f,
                "({})",
                values
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        }
    }
}

impl std::fmt::Display for Scope {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let values = self.bindings
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect::<Vec<_>>().join(", ");

        write!(f, "{{ {values} }}")
    }
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
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
                write!(f, "Invalid function call, got `{head}` of type `{}`. \nIs this supposed to be a list? If so, use `{}`", head.to_type(), correct_expr)
            }
            EvalError::UnknownIdentifier(identifier) => {
                write!(f, "Unknown identifier `{}`.", identifier)
            }
        }
    }
}
