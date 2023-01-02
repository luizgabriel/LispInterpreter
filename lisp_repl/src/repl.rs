use std::fmt;

use colored::Colorize;
use rustyline::error::ReadlineError;
use rustyline::Editor;

use lisp_lang::{evaluation::{*, scope::{Scope, MAIN_CONTEXT}}, parsing::*};

use crate::display::{ColoredError, ColoredLispVal};

#[derive(Debug)]
pub enum REPLError {
    ReadlineError(String),
    ParseError(String),
    EvaluationError(String),
}

impl std::error::Error for REPLError {}

impl fmt::Display for REPLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            REPLError::ReadlineError(s) => write!(f, "{}", s),
            REPLError::ParseError(s) => write!(f, "{} {}", "Parse Error:".red(), s),
            REPLError::EvaluationError(s) => write!(f, "{} {}", "Evaluation Error: ".red(), s),
        }
    }
}

fn to_readline_error(e: ReadlineError) -> REPLError {
    match e {
        ReadlineError::Interrupted => REPLError::ReadlineError("CTRL-C".to_string()),
        ReadlineError::Eof => REPLError::ReadlineError("CTRL-D".to_string()),
        err => REPLError::ReadlineError(format!("Error: {:?}", err)),
    }
}

pub fn read(rl: &mut Editor<()>) -> Result<String, REPLError> {
    let prompt = format!("{} ", ">".bright_blue().bold());
    let input = rl.readline(&prompt).map_err(to_readline_error)?;

    Ok(input)
}

fn unwrap_expression(parse_result: (&str, LispVal)) -> Result<LispVal, REPLError> {
    let (rest, expr) = parse_result;
    if rest.is_empty() {
        Ok(expr)
    } else {
        Err(REPLError::ParseError(format!("Unexpected input: {rest}")))
    }
}

pub fn evaluate(scope: Scope, input: &str) -> Result<(Scope, ColoredLispVal), REPLError> {
    let expr = parse(input)
        .map_err(|e| REPLError::ParseError(e.to_string()))
        .and_then(unwrap_expression)?;

    eval(scope, &expr)
        .map(|(new_scope, val)| (new_scope.with_context(MAIN_CONTEXT.to_string()), ColoredLispVal::new(val)))
        .map_err(ColoredError::new)
        .map_err(|e| REPLError::EvaluationError(e.to_string()))
}
