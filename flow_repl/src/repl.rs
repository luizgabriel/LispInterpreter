use std::fmt;
use rustyline::Editor;
use flow_lang::{evaluation::*, parsing::*};
use rustyline::error::ReadlineError;
use crate::display::{ColoredLispVal, ColoredError};
use colored::Colorize;

pub enum REPLError {
    ReadlineError(String),
    ParseError(String),
    EvaluationError(String),
}

impl fmt::Display for REPLError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            REPLError::ReadlineError(s) => write!(f, "{}", s),
            REPLError::ParseError(s) => write!(f, "Parse Error: {}", s),
            REPLError::EvaluationError(s) => write!(f, "Evaluation Error: {}", s),
        }
    }
}

fn to_repl_readline_error(e: ReadlineError) -> REPLError {
    match e {
        ReadlineError::Interrupted => REPLError::ReadlineError("CTRL-C".to_string()),
        ReadlineError::Eof => REPLError::ReadlineError("CTRL-D".to_string()),
        err => REPLError::ReadlineError(format!("Error: {:?}", err)),
    }
}

pub fn read(rl: &mut Editor::<()>) -> Result<String, REPLError> {
    // Read a line of input from the user
    let input = rl.readline(&format!("{} ", ">".bright_blue().bold()))
        .map_err(to_repl_readline_error)?;

    // Save the input to history
    rl.add_history_entry(input.as_str());
    Ok(input)
}

pub fn evaluate(input: String) -> Result<ColoredLispVal, REPLError> {
    // Parse the input
    let expr = parse(&input)
        .map(|(_, expr)| expr)
        .map_err(|e| REPLError::ParseError(e.to_string()))?;

    // Evaluate the expression
    eval(&expr)
        .map(ColoredLispVal::new)
        .map_err(ColoredError::new)
        .map_err(|e| REPLError::EvaluationError(e.to_string()))
}