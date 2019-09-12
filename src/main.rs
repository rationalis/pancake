use std::io;

use pancake::eval::eval_line;
use pancake::types::{Atom, Context, Env, Op};

fn main() {
    println!("Op size in bytes: {}", std::mem::size_of::<Op>());
    println!("Context size in bytes: {}", std::mem::size_of::<Context>());
    println!("Atom size in bytes: {}", std::mem::size_of::<Atom>());
    println!("Env size in bytes: {}", std::mem::size_of::<Env>());
    let mut env = Env::new();
    loop {
        let mut line = String::new();

        io::stdin()
            .read_line(&mut line)
            .expect("Failed to read line");

        eval_line(&line, &mut env);

        println!("Env: {:#?}", env);
    }
}
