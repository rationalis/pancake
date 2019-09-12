#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate flamer;

pub mod eval;
pub mod ops;
pub mod parse;

pub mod types {
    use inlinable_string::InlinableString;
    use std::collections::HashMap;
    use std::convert::TryFrom;

    pub const SPECIAL_IDENTS : [&str;6] = [
        "call", "let", "fn", "true", "false", "not"];

    pub type NumType = i32;
    pub type Identifier = InlinableString;
    pub type IsFunction = bool;

    pub type Stack = Vec<Atom>;

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum OpA {
        Add,
        Sub,
        Mult,
        Div,
        Mod,
        Less,
        Greater,
        LEq,
        GEq,
        Eq
    }

    impl TryFrom<&str> for OpA {
        type Error = &'static str;

        fn try_from(op: &str) -> Result<Self, Self::Error> {
            Ok(
                match op {
                    "+" => Self::Add,
                    "-" => Self::Sub,
                    "*" => Self::Mult,
                    "/" => Self::Div,
                    "%" => Self::Mod,
                    "<" => Self::Less,
                    ">" => Self::Greater,
                    "<=" => Self::LEq,
                    ">=" => Self::GEq,
                    "=" => Self::Eq,
                    _ => { return Err("Unrecognized arithmetic operator."); }
                }
            )
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum OpB {
        And,
        Or,
        Cond,
        If
    }

    impl TryFrom<&str> for OpB {
        type Error = &'static str;

        fn try_from(op: &str) -> Result<Self, Self::Error> {
            Ok(
                match op {
                    "and" => Self::And,
                    "or" => Self::Or,
                    "cond" => Self::Cond,
                    "if" => Self::If,
                    _ => { return Err("Unrecognized boolean operator."); }
                }
            )
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum OpS {
        Dup,
        Drop,
        Swap,
        List,
        Map,
        Splat,
        Repeat,
        Get
    }

    impl TryFrom<&str> for OpS {
        type Error = &'static str;

        fn try_from(op: &str) -> Result<Self, Self::Error> {
            Ok(
                match op {
                    "dup" => Self::Dup,
                    "drop" => Self::Drop,
                    "swap" => Self::Swap,
                    "list" => Self::List,
                    "map" => Self::Map,
                    "splat" => Self::Splat,
                    "repeat" => Self::Repeat,
                    "get" => Self::Get,
                    _ => { return Err("Unrecognized stack operator."); }
                }
            )
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Atom {
        Bool(bool),
        Num(NumType),

        List(Vec<Atom>),

        NotOp,
        BooleanOp(OpB),
        ArithmeticOp(OpA),
        StackOp(OpS),

        QuotationStart, // [
        QuotationEnd, // ]
        Quotation(Vec<Atom>),
        Function(Vec<Identifier>, Vec<Atom>),

        DefVar,
        DefFn(Vec<Identifier>),
        DefOp(IsFunction),

        Call,

        Symbol(Identifier),
        Plain(Identifier)
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
            if SPECIAL_IDENTS.contains(&ident) ||
                OpB::try_from(ident).is_ok() ||
                OpS::try_from(ident).is_ok() {

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
        pub lazy: bool
    }

    fn blank_frame() -> Frame {
        Frame {
            stack: Stack::new(),
            context: Context::new(),
            params: Context::new(),
            lazy: false
        }
    }

    #[derive(Debug, Default)]
    pub struct Env(Vec<Frame>);

    impl Env {
        pub fn new() -> Env {
            Env(vec![blank_frame()])
        }

        fn last_frame(&mut self) -> &mut Frame {
            if let Some(frame) = self.0.last_mut() {
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
            self.0.push(f)
        }

        pub fn pop(&mut self) -> Option<Frame> {
            self.0.pop()
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
            if let Some(f) = self.0.last() {
                if let Some(atom) = f.params.get(ident) {
                    return Some(atom.clone());
                }
            }
            for frame in self.0.iter().rev() {
                let context = &frame.context;
                if let Some(atom) = context.get(ident) {
                    return Some(atom.clone());
                }
            }
            None
        }

        pub fn lazy_mode(&self) -> bool {
            self.0.last().unwrap().lazy
        }
    }
}
