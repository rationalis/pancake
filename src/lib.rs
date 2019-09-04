#[macro_use] extern crate lazy_static;

pub mod eval;
pub mod ops;

pub mod types {
    use std::collections::HashMap;

    pub const SPECIAL_CHARS : &str = "+-*/%[]'";
    pub const ARITHMETIC_OPS : [&'static str;10] = [
        "+","-","*","/","%","<",">","<=",">=","="];
    pub const BOOLEAN_OPS : [&'static str;4] = [
        "and", "or", "cond", "if"];
    pub const STACK_OPS : [&'static str;3] = [
        "dup", "drop", "swap"];
    pub const SPECIAL_IDENTS : [&'static str;6] = [
        "call", "let", "fn", "true", "false", "not"];

    pub type NumType = i32;
    pub type Identifier = String;
    pub type UnparsedExpr = String;
    pub type IsFunction = bool;

    pub type Stack = Vec<Atom>;
    pub type Frame = (Stack, Context, bool);

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Atom {
        Bool(bool),
        Num(NumType),

        NotOp,
        BooleanOp(String),
        ArithmeticOp(String),
        StackOp(String),

        QuotationStart, // [
        QuotationEnd, // ]
        Quotation(Vec<Atom>, IsFunction),

        DefUnparsed(Identifier, UnparsedExpr, IsFunction),
        DefOp(IsFunction),

        Call,

        Symbol(Identifier),
        Plain(Identifier)
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

        fn last_frame(&mut self) -> &mut Frame {
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

        pub fn pop_atom(&mut self) -> Atom {
            if let Some(a) = self.last_frame().0.pop() {
                a
            } else {
                panic!("Popped atom from empty frame");
            }
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

        pub fn lazy_mode(&self) -> bool {
            self.0.last().unwrap().2
        }
    }
}
