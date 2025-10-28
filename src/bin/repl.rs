#![allow(unused_imports, unused_variables)]

use rustyline::error::ReadlineError;
use rustyline::Editor;

use cfg_if::cfg_if;

use l::byte::I;
use l::vm::V;

// ANCHOR: repl
fn main() {
    let mut rl = Editor::<()>::new();
    println!("l prompt. Expressions are line evaluated.");
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let byte_code = I::fstring(&line);

                match byte_code {
                    Ok(it) => {
                        println!("byte code: {:?}", it);

                        let mut vm = V::new(it);
                        vm.r();
                        println!("{}", vm.pop_last());
                    },
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
