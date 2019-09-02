#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate im_rc;

use std::io;
//use im_rc::vector::Vector;

mod eval;

pub(crate) mod types {
    use std::collections::HashMap;

    pub const SPECIAL_CHARS : &str = "+-*/%[]";
    pub const ARITHMETIC_OPS : &str = "+-*/%";
    pub const SPECIAL_IDENTS : [&'static str;1] = ["call"];

    pub type NumType = i32;
    pub type IsFunction = bool;

    pub type Stack = Vec<Atom>;
    pub type Frame = (Stack, Context, bool);

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Atom {
        Num(NumType),
        ArithmeticOp(char),
        QuotationStart, // [
        QuotationEnd, // ]
        Quotation(Vec<Atom>, IsFunction),
        Call,
        Let,
        Defn,
        Plain(String)
    }

    #[derive(Debug)]
    pub struct Context(HashMap<String, Atom>);

    impl Context {
        fn new() -> Context {
            Context(HashMap::new())
        }

        fn get(&self, ident: &String) -> Option<&Atom> {
            self.0.get(ident)
        }

        fn insert(&mut self, ident: String, atom: Atom) {
            if let Some(_) = self.0.insert(ident, atom) {
                panic!("Attempted to rebind existing variable.");
            }
        }
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

        // TODO: Add fns last_atom, pop_atom

        pub fn push_atom(&mut self, atom: Atom) {
            self.last_frame().0.push(atom)
        }

        pub fn push_blank(&mut self, lazy: bool) {
            let frame = (Stack::new(), Context::new(), lazy);
            self.0.push(frame)
        }

        pub fn pop(&mut self) -> Option<Frame> {
            self.0.pop()
        }

        pub fn bind_var(&mut self, ident: String, atom: Atom) {
            self.last_frame().1.insert(ident, atom)
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

        pub fn eval_with_new_scope(&mut self, expr: &String, lazy: bool)
                                   -> Atom {
            self.push_blank(false);
            crate::eval::eval_line(&expr, self);
            let mut stack : Stack = self.pop().unwrap().0;
            if let Some(atom) = stack.pop() {
                return atom;
            } else {
                panic!("Expected result but stack was empty.");
            }
        }

        pub fn lazy_mode(&self) -> bool {
            self.0.last().unwrap().2
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
