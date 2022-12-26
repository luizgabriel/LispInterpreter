mod parsing;
mod evaluation;

use rustyline::error::ReadlineError;
use rustyline::Editor;


fn main() {
    // Set up the rustyline editor to handle input and output
    let mut rl = Editor::<()>::new().expect("Could not initialize rustyline::Editor.");

    // Load any previously saved history
    rl.load_history("history.txt").unwrap_or_default();

    loop {
        // Read a line of input from the user
        let readline = rl.readline("> ");
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
        let expr = match parsing::parse(&input) {
            Ok((_, expr)) => expr,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        // Evaluate the expression
        let result = match evaluation::eval(&expr) {
            Ok(result) => result,
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        };

        println!("{}", result);
    }

    // Save the history to a file
    rl.save_history("history.txt").unwrap();
}
