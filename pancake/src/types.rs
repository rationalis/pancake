use crate::vm::{Env, Identifier};

pub type NumType = i32;
pub type IsFunction = bool;

pub type Arity = Option<(u8, u8)>;

#[derive(Clone)]
pub struct Op {
    pub f: fn(&mut Env),
    pub arity: Arity,
}

use std::fmt;
impl fmt::Debug for Op {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Op {}", self.f as usize)
    }
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        self.f as usize == other.f as usize
    }
}

impl PartialEq<&Op> for Op {
    fn eq(&self, other: &&Op) -> bool {
        self.f as usize == other.f as usize
    }
}

use std::hash::{Hash, Hasher};
impl Hash for Op {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (self.f as usize).hash(state);
    }
}

impl Eq for Op {}

impl Op {
    pub fn new(f: fn(&mut Env), arity: Arity) -> Self {
        Self { f, arity }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
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
