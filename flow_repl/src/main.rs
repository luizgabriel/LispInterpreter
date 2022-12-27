use rustyline::error::ReadlineError;
use rustyline::Editor;
use flow_lang::{evaluation::*, parsing::*};
use colored::*;

const HISTORY_PATH: &str = ".flow_history";

struct ColoredLispVal {
    value: LispVal,
}

impl ColoredLispVal {
    fn new(value: LispVal) -> Self {
        Self { value }
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

fn main() {
    // Set up the rustyline editor to handle input and output
    let mut rl = Editor::<()>::new().unwrap();

    // Load any previously saved history
    rl.load_history(HISTORY_PATH).unwrap_or_default();

    loop {
        // Read a line of input from the user
        let readline = rl.readline(&format!("{} ", ">".bright_blue().bold()));
        let input = match readline {
            Ok(line) => line,
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        };

        // Save the input to history
        rl.add_history_entry(input.as_str());

        // Parse the input
        let expr = match parse(&input) {
            Ok((_, expr)) => expr,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        // Evaluate the expression
        let result = match eval(&expr) {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        println!("{}", ColoredLispVal::new(result));
    }

    // Save the history to a file
    rl.save_history(HISTORY_PATH).unwrap();
}
