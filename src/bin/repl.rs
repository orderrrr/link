#![allow(unused_imports, unused_variables)]

use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;

use cfg_if::cfg_if;

use l::byte::I;
use l::vm::V;

// ANCHOR: repl
fn main() {
    let mut rl = DefaultEditor::new().unwrap();
    println!("l prompt. Expressions are line evaluated.");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = rl.add_history_entry(&line);
                let byte_code = I::fstring(&line);

                match byte_code {
                    Ok(it) => {
                        println!("byte code: {:?}", it);

                        let mut vm = V::new(it);
                        vm.r();
                        match &vm.error {
                            Some(e) => println!("{}", e),
                            None => match vm.pop_last() {
                                Some(result) => println!("{}", result),
                                None => println!("(no result)"),
                            },
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            }
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
        }
    }
}
