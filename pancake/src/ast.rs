use crate::vm2::{Identifier};

use std::convert::From;

pub type NumType = i32;

struct Pgrm {
    defs: Vec<(bool, String, Term)>,
}


#[derive(Debug, Clone)]
pub enum Literal {
    //Bool(bool),
    Int(i32)
}

type BTerm = Box<Term>;

pub struct Let {
    pub rec: bool,
    pub name: Identifier,
    pub rhs: BTerm
}

pub enum Term {
    Lit(Literal),
    Var(Identifier),
    Lam(Identifier, BTerm),
    App(BTerm, BTerm),
    // Rcd(Vec<Identifier, BTerm>)
    // Sel(BTerm, Identifier)
    Let {
        binding: Let,
        body: BTerm
    }
}
