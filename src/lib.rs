#[macro_use] extern crate lazy_static;

pub mod eval;
pub mod ops;
pub mod parse;

pub mod types {
    use inlinable_string::InlinableString;
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
    pub type SpecialIdentifier = &'static str;
    pub type Identifier = InlinableString;
    pub type UnparsedExpr = InlinableString;
    pub type IsFunction = bool;

    pub type Stack = Vec<Atom>;

    #[derive(Debug, Clone, Eq, PartialEq)]
    pub enum Atom {
        Bool(bool),
        Num(NumType),

        List(Vec<Atom>),

        NotOp,
        BooleanOp(SpecialIdentifier),
        ArithmeticOp(SpecialIdentifier),
        StackOp(SpecialIdentifier),

        QuotationStart, // [
        QuotationEnd, // ]
        Quotation(Vec<Atom>),
        Function(Vec<Identifier>, Vec<Atom>),

        DefUnparsedVar(Identifier, UnparsedExpr),
        DefUnparsedFn(Identifier, Vec<Identifier>, UnparsedExpr),
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

        fn get(&self, ident: &str) -> Option<&Atom> {
            self.0.get(ident)
        }

        fn insert(&mut self, ident: &str, atom: Atom) {
            if SPECIAL_IDENTS.contains(&ident) ||
                BOOLEAN_OPS.contains(&ident) ||
                STACK_OPS.contains(&ident) {

                panic!("Attempted to rebind reserved word {}.", ident);
            }

            if let Some(_) = self.0.insert(InlinableString::from(ident), atom) {
                panic!("Attempted to rebind existing variable {}.", ident);
            }
        }
    }

    #[derive(Debug)]
    pub struct Frame {
        pub stack: Stack,
        pub context: Context,
        pub params: Option<Context>,
        pub lazy: bool
    }

    fn blank_frame() -> Frame {
        return Frame {
            stack: Stack::new(),
            context: Context::new(),
            params: None,
            lazy: false
        };
    }

    #[derive(Debug)]
    pub struct Env(Vec<Frame>);

    impl Env {
        pub fn new() -> Env {
            Env(vec![blank_frame()])
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
            let mut bound_params = Context::new();
            for ident in idents.iter().rev() {
                bound_params.insert(ident, self.pop_atom())
            }
            self.push_blank(false);
            self.last_frame().params = Some(bound_params);
        }

        pub fn unbind_params(&mut self) {
            let mut frame = self.pop().unwrap();
            self.last_frame().stack.append(&mut frame.stack)
        }

        pub fn find_var(&mut self, ident: &Identifier) -> Option<Atom> {
            if let Some(f) = self.0.last() {
                if let Some(p) = &f.params {
                    if let Some(atom) = p.get(ident) {
                        return Some(atom.clone());
                    }
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
