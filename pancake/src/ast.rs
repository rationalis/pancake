use crate::vm2::{Identifier};

use std::convert::From;

pub type NumType = i32;
// pub type IsFunction = bool;

// pub type Arity = Option<(u8, u8)>;

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

// type BExpr = Box<Expr>;
// type VarDefinition = (Identifier, BExpr);
// type CaseMatchPattern = (Identifier, Identifier);

// #[derive(Debug)]
// pub enum Expr {
//     Call(BExpr, Vec<BExpr>),
//     // Case(Identifier, BExpr),
//     // FieldAccess(BExpr, Identifier),
//     FuncDef(Vec<Identifier>, BExpr),
//     // If(BExpr, BExpr, BExpr),
//     Let(VarDefinition, BExpr),
//     LetRec(Vec<VarDefinition>, BExpr),
//     Literal(Literal),
//     // Match(BExpr, Vec<(CaseMatchPattern, BExpr)>),
//     // Record(Vec<(Identifier, BExpr)>),
//     Variable(Identifier),
//     QuotationStart,
//     QuotationEnd,
// }

// impl From<NumType> for Expr {
//     fn from(item: i32) -> Self {
//         Expr::Literal(Literal::Int(item))
//     }
// }

// impl From<bool> for Expr {
//     fn from(item: bool) -> Self {
//         Expr::Literal(Literal::Bool(item))
//     }
// }
