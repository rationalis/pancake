use std::io;

use pancake::eval::eval_line;
use pancake::types::Env;

fn main() {
    let mut env = Env::new();
    loop {
        let mut line = String::new();

        io::stdin().read_line(&mut line)
            .expect("Failed to read line");

        eval_line(&line, &mut env);

        println!("Env: {:#?}", env);
    }
}
