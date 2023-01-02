use repl::{evaluate, read, REPLError};
use lisp_lang::evaluation::scope::INITIAL_SCOPE;

mod display;
mod repl;

const HISTORY_PATH: &str = ".flow_history";

fn main() {
    let config = rustyline::Config::builder()
        .auto_add_history(true)
        .color_mode(rustyline::ColorMode::Enabled)
        .build();

    let mut rl = rustyline::Editor::<()>::with_config(config).unwrap();
    let mut scope = INITIAL_SCOPE.clone();

    rl.load_history(HISTORY_PATH).unwrap_or_default();

    loop {
        match read(&mut rl).and_then(|input| evaluate(scope.clone(), input.as_str())) {
            Ok((new_scope, result )) => {
                if !result.value.is_void()  {
                    println!("{}", result);
                }
                scope = new_scope;
            }
            Err(err) => {
                println!("{}", err);
                if let REPLError::ReadlineError(_) = err {
                    break;
                }
            }
        }
    }

    rl.save_history(HISTORY_PATH).unwrap();
}
