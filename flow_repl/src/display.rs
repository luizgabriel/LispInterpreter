use colored::Colorize;
use regex::Regex;

use flow_lang::parsing::*;

fn transform_single_quoted_text<F: Fn(&str) -> String>(transform: F) -> impl Fn(&str) -> String
{
    move |s| {
        let re = Regex::new(r#"`(?:[^`\\]|\\.)*`"#).unwrap();
        let mut result = String::new();
        let mut last_match_end = 0;

        for capture in re.captures_iter(s) {
            let quoted_text = &capture[0];
            let start = capture.get(0).unwrap().start();
            let end = capture.get(0).unwrap().end();

            result.push_str(&s[last_match_end..start]);
            result.push_str(&transform(&quoted_text[1..quoted_text.len() - 1]));
            last_match_end = end;
        }

        result.push_str(&s[last_match_end..]);
        result
    }
}

fn colorize_quoted_expressions(s: &str) -> String {
    let transform = |s: &str| -> String {
        parse(s).ok()
            .map(|(_, v)| ColoredLispVal::new(v).to_string())
            .unwrap_or(s.to_string())
    };

    transform_single_quoted_text(transform)(s)
}

pub struct ColoredLispVal {
    value: LispVal,
}

impl ColoredLispVal {
    pub fn new(value: LispVal) -> Self {
        Self { value }
    }
}

#[derive(Debug)]
pub struct ColoredError<T: std::error::Error> {
    error: T,
}

impl<T: std::error::Error> ColoredError<T> {
    pub fn new(error: T) -> Self {
        Self { error }
    }
}

impl<T: std::error::Error> std::fmt::Display for ColoredError<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", colorize_quoted_expressions(&self.error.to_string()))
    }
}

impl std::fmt::Display for ColoredLispVal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.value {
            LispVal::Void() => write!(f, ""),
            LispVal::Symbol(atom) => write!(f, "{}", atom.bright_blue()),
            LispVal::Number(n) => write!(f, "{}", n.to_string().bright_green()),
            LispVal::String(s) => write!(f, "{}{}{}", "\"".bright_green().italic(), s.bright_green(), "\"".bright_green().italic()),
            LispVal::Unevaluated(expr) => write!(f, "{}{}", "'".bright_blue().italic(), ColoredLispVal::new(*expr.clone())),
            LispVal::List(values) => {
                let inner_values = values.iter()
                    .map(|v| ColoredLispVal::new(v.clone()).to_string())
                    .collect::<Vec<String>>()
                    .join(" ");
                write!(f, "({})", inner_values)
            }
        }
    }
}
