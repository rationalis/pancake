#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate im_rc;

use std::io;
//use im_rc::vector::Vector;

mod eval;

pub(crate) mod types {
    use std::collections::HashMap;

    pub const SPECIAL_CHARS : &str = "+-*/%[]";
    pub const ARITHMETIC_OPS : &str = "+-*/%";

    pub type NumType = i32;
    pub type IsFunction = bool;

    pub type Stack = Vec<Atom>;
    pub type Context = HashMap<String, Atom>;
    pub type Frame = (Stack, Context, bool);

    #[derive(Debug, Clone)]
    pub enum Atom {
        Num(NumType),
        ArithmeticOp(char),
        QuotationStart, // [
        QuotationEnd, // ]
        FunctionEnd, // implicit ]
        Quotation(Vec<Atom>, IsFunction),
        Plain(String)
    }

    #[derive(Debug)]
    pub struct Env(Vec<Frame>);

    impl Env {
        pub fn new() -> Env {
            Env(vec![(Stack::new(), Context::new(), false)])
        }

        pub fn last_frame(&mut self) -> &mut Frame {
            if let Some(frame) = self.0.last_mut() {
                return frame;
            } else {
                panic!("Tried to get a frame from an empty stack.")
            }
        }

        pub fn push_blank(&mut self, lazy: bool) {
            let frame = (Stack::new(), Context::new(), lazy);
            self.0.push(frame)
        }

        pub fn pop(&mut self) -> Option<Frame> {
            self.0.pop()
        }

        pub fn find_var(&mut self, ident: &String) -> Option<Atom> {
            for frame in self.0.iter().rev() {
                let context = &frame.1;
                if let Some(atom) = context.get(ident) {
                    return Some(atom.clone());
                }
            }
            None
        }
    }
}


fn main() {
    let mut env = types::Env::new();
    loop {
        let mut line = String::new();

        io::stdin().read_line(&mut line)
            .expect("Failed to read line");

        eval::eval_line(&line, &mut env);

        println!("Env: {:?}", env);
    }
}
