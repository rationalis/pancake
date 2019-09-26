use crate::types::{Atom, Env, Stack, OpA::*, OpB::*, OpS::*};
use crate::eval::{eval_with_new_scope, make_fn_q};
use crate::ops;
use crate::eval_op;

#[derive(Debug, Clone)]
pub struct Transform(Box<dyn FnOnce(&mut Env) -> Vec<Transform>>);

pub fn get_transform(atom: Atom) -> Transform {
    Transform(
        match atom {
            Atom::ArithmeticOp(c) => 
                match c {
                    Add => Box::new(|env| {eval_op!(+, Atom::Num, env); Vec::new()}),
                    Sub => Box::new(|env| {eval_op!(-, Atom::Num, env); Vec::new()}),
                    Mult => Box::new(|env| {eval_op!(*, Atom::Num, env); Vec::new()}),
                    Div => Box::new(|env| {eval_op!(/, Atom::Num, env); Vec::new()}),
                    Mod => Box::new(|env| {eval_op!(%, Atom::Num, env); Vec::new()}),
                    Less => Box::new(|env| {eval_op!(<, Atom::Num, Atom::Bool, env); Vec::new()}),
                    Greater => Box::new(|env| {eval_op!(>, Atom::Num, Atom::Bool, env); Vec::new()}),
                    LEq => Box::new(|env| {eval_op!(<=, Atom::Num, Atom::Bool, env); Vec::new()}),
                    GEq => Box::new(|env| {eval_op!(>=, Atom::Num, Atom::Bool, env); Vec::new()}),
                    Eq => Box::new(|env| {eval_op!(==, Atom::Num, Atom::Bool, env); Vec::new()}),
                },
            Atom::BooleanOp(s) => Box::new(move |env| {
                ops::eval_boolean_op(s, env); Vec::new()}),
            Atom::StackOp(s) => Box::new(move |env| {
                ops::eval_stack_op(s, env); Vec::new()}),
            Atom::NotOp => Box::new(|env| {
                if let Atom::Bool(b) = env.pop_atom() {
                    env.push_atom(Atom::Bool(!b));
                    Vec::new()
                } else {
                    panic!("Tried to negate a non-boolean.")
                }
            }),
            Atom::QuotationStart => Box::new(|env| {
                env.push_blank(true);
                Vec::new()
            }),
            Atom::QuotationEnd => Box::new(|env| {
                let stack: Stack = env.pop().unwrap().stack;
                let quotation = Atom::Quotation(stack.iter().map(|a| get_transform(*a)).collect());
                env.push_atom(quotation);
                Vec::new()
            }),
            Atom::DefVar => Box::new(|env| {
                let a = env.pop_atom();
                let b = env.pop_atom();
                if let (Atom::Symbol(ident), Atom::Quotation(expr)) = (a, b) {
                    let result_of_expr = eval_with_new_scope(expr, env);
                    env.bind_var(&ident, result_of_expr);
                } else {
                    unreachable!();
                }
                Vec::new()
            }),
            Atom::DefFn(params) => Box::new(|env| {
                let a = env.pop_atom();
                let b = env.pop_atom();
                if let (Atom::Symbol(ident), Atom::Quotation(expr)) = (a, b) {
                    env.bind_var(&ident, Atom::Function(params, expr));
                } else {
                    unreachable!();
                }
                Vec::new()
            }),
            Atom::DefOp(is_function) => Box::new(move |env| {
                let a = env.pop_atom();
                let b = env.pop_atom();
                if is_function {
                    if let (Atom::Symbol(ident), Atom::Quotation(q)) = (a, b) {
                        env.bind_var(&ident,
                                     Atom::Function(Vec::new(), q));
                    } else {
                        panic!("Expected '<quotation> <ident> fn'.")
                    }
                } else if let Atom::Symbol(ident) = a {
                    env.bind_var(&ident, b);
                } else {
                    panic!("Expected '<value> <ident> let'.")
                }
                Vec::new()
            }),
            Atom::Call => Box::new(|env| {
                if let Atom::Quotation(q) = env.pop_atom() {
                    return q;
                } else {
                    panic!("Tried to call a non-quotation.");
                }
            }),
            Atom::Plain(ident) => Box::new(move |env| {
                match env.find_var(&ident) {
                    Some(Atom::Function(params, body)) => {
                        if params.is_empty() {
                            return vec![
                                Atom::Quotation(body),
                                Atom::Call,
                            ];
                        }
                        env.bind_params(params);
                        vec![
                            Atom::Quotation(body),
                            Atom::Call,
                            Atom::Unbind
                        ]
                    }
                    Some(atom) => {
                        env.push_atom(atom);
                        Vec::new()
                    },
                    _ => panic!("Unrecognized identifier: {}", ident)
                }
            }),
            Atom::Unbind => Box::new(|env| {
                env.unbind_params();
                Vec::new()
            }),
            _ => Box::new(|env| {
                env.push_atom(atom);
                Vec::new()
            })
        }
    )
}

