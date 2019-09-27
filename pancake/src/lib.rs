#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate lazy_static;
//#[macro_use] extern crate flamer;

pub mod eval;
pub mod ops;
pub mod parse;

pub mod types {
    use inlinable_string::InlinableString;
    use std::collections::HashMap;

    pub const SPECIAL_IDENTS: [&str; 6] = ["call", "let", "fn", "true", "false", "not"];

    pub type NumType = i32;
    pub type Identifier = InlinableString;
    pub type IsFunction = bool;

    pub type Stack = Vec<Atom>;

    pub type Arity = Option<(u8, u8)>;

    #[derive(Clone)]
    pub struct Op(fn(&mut Env), Option<(u8, u8)>);

    use std::fmt;
    impl fmt::Debug for Op {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Op {}", self.0 as usize)
        }
    }

    impl PartialEq for Op {
        fn eq(&self, other: &Self) -> bool {
            self.0 as usize == other.0 as usize
        }
    }

    impl Eq for Op {}

    impl Op {
        pub fn new(pair: (fn(&mut Env), Arity)) -> Self {
            Self(pair.0, pair.1)
        }
        pub fn f(&self) -> fn(&mut Env) {
            self.0
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Atom {
        Bool(bool),
        Num(NumType),

        List(Vec<Atom>),

        Op(Op),

        QuotationStart, // [
        QuotationEnd,   // ]
        Quotation(Vec<Atom>),
        Function(Vec<Identifier>, Vec<Atom>),

        DefVar,
        DefVarLiteral,
        DefFnLiteral,

        Call,

        Symbol(Identifier),
        Plain(Identifier),
    }

    #[derive(Debug)]
    pub struct Context(HashMap<InlinableString, Atom>);

    impl Context {
        fn new() -> Context {
            Context(HashMap::new())
        }

        fn with_capacity(cap: usize) -> Context {
            Context(HashMap::with_capacity(cap))
        }

        fn get(&self, ident: &str) -> Option<&Atom> {
            self.0.get(ident)
        }

        fn insert(&mut self, ident: &str, atom: Atom) {
            use crate::ops;
            if SPECIAL_IDENTS.contains(&ident)
                || ops::get_boolean_op(ident).is_some()
                || ops::get_stack_op(ident).is_some()
            {
                panic!("Attempted to rebind reserved word {}.", ident);
            }

            if self.0.insert(InlinableString::from(ident), atom).is_some() {
                panic!("Attempted to rebind existing variable {}.", ident);
            }
        }
    }

    #[derive(Debug)]
    pub struct Frame {
        pub stack: Stack,
        pub context: Context,
        pub params: Context,
        pub lazy: bool,
    }

    fn blank_frame() -> Frame {
        Frame {
            stack: Stack::with_capacity(10),
            context: Context::new(),
            params: Context::new(),
            lazy: false,
        }
    }

    #[derive(Debug, Default)]
    pub struct Env {
        frames: Vec<Frame>,
        pub loop_like: bool,
        pub using_for_else: bool,
        pub for_else: bool,
    }

    impl Env {
        pub fn new() -> Env {
            Env {
                frames: vec![blank_frame()],
                loop_like: false,
                using_for_else: false,
                for_else: true,
            }
        }

        fn last_frame(&mut self) -> &mut Frame {
            if let Some(frame) = self.frames.last_mut() {
                frame
            } else {
                panic!("Tried to get a frame from an empty stack.")
            }
        }

        pub fn append_atoms(&mut self, mut atoms: Vec<Atom>) {
            self.last_frame().stack.append(&mut atoms)
        }

        pub fn push_atom(&mut self, atom: Atom) {
            self.last_frame().stack.push(atom)
        }

        pub fn pop_atom(&mut self) -> Atom {
            if let Some(a) = self.last_frame().stack.pop() {
                a
            } else {
                panic!("Popped atom from empty frame");
            }
        }

        pub fn push_blank(&mut self, lazy: bool) {
            let mut f = blank_frame();
            f.lazy = lazy;
            self.frames.push(f)
        }

        pub fn pop(&mut self) -> Option<Frame> {
            self.frames.pop()
        }

        pub fn bind_var(&mut self, ident: &str, atom: Atom) {
            self.last_frame().context.insert(ident, atom)
        }

        pub fn bind_params(&mut self, idents: Vec<Identifier>) {
            let mut bound_params = Context::with_capacity(idents.len());
            for ident in idents.iter().rev() {
                bound_params.insert(ident, self.pop_atom())
            }
            self.push_blank(false);
            self.last_frame().params = bound_params;
        }

        pub fn unbind_params(&mut self) {
            let mut frame = self.pop().unwrap();
            self.last_frame().stack.append(&mut frame.stack)
        }

        pub fn find_var(&self, ident: &Identifier) -> Option<Atom> {
            if let Some(f) = self.frames.last() {
                if let Some(atom) = f.params.get(ident) {
                    return Some(atom.clone());
                }
            }
            for frame in self.frames.iter().rev() {
                let context = &frame.context;
                if let Some(atom) = context.get(ident) {
                    return Some(atom.clone());
                }
            }
            None
        }

        pub fn lazy_mode(&self) -> bool {
            self.frames.last().unwrap().lazy
        }
    }
}
