use rustyline::Editor;

use repl::{evaluate, read, REPLError};
use flow_lang::evaluation::scope::GLOBAL_SCOPE;

mod display;
mod repl;

const HISTORY_PATH: &str = ".flow_history";

fn main() {
    // Set up the rustyline editor to handle input and output
    let mut rl = Editor::<()>::new().unwrap();
    let mut scope = GLOBAL_SCOPE.clone();

    // Load any previously saved history
    rl.load_history(HISTORY_PATH).unwrap_or_default();

    loop {
        match read(&mut rl).and_then(|input| evaluate(scope.clone(), input.as_str())) {
            Ok((new_scope, result )) => {
                println!("{}", result);
                scope = new_scope;
            }
            Err(err) => {
                if let REPLError::ReadlineError(_) = err {
                    break;
                }
                println!("{}", err);
            }
        }
    }

    // Save the history to a file
    rl.save_history(HISTORY_PATH).unwrap();
}
