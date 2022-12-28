use rustyline::Editor;

use repl::{evaluate, read, REPLError};

mod display;
mod repl;

const HISTORY_PATH: &str = ".flow_history";

fn main() {
    // Set up the rustyline editor to handle input and output
    let mut rl = Editor::<()>::new().unwrap();

    // Load any previously saved history
    rl.load_history(HISTORY_PATH).unwrap_or_default();

    loop {
        match read(&mut rl).and_then(evaluate) {
            Ok(result) => println!("{}", result),
            Err(REPLError::ReadlineError(s)) => {
                println!("{}", s);
                break;
            }
            Err(REPLError::ParseError(e)) => println!("{}", e),
            Err(REPLError::EvaluationError(e)) => println!("{}", e),
        }
    }

    // Save the history to a file
    rl.save_history(HISTORY_PATH).unwrap();
}
