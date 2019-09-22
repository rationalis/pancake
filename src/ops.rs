use crate::eval::{eval_call, eval_function};
use crate::types::{Atom, Env};

use Atom::*;

use pancake_macro::{atomify, binops};

pub fn get_arithmetic_op(op: &str) -> Option<fn(&mut Env)> {
    binops!(a"+" a"-" a"*" a"/" a"%" c"<" c">" c"<=" c">=" c"==" c"!=")
}

pub fn get_boolean_op(op: &str) -> Option<fn(&mut Env)> {
    Some(match op {
        "and" => atomify!("and" ((a:Bool,b:Bool)->Bool) {a && b}),
        "or" => atomify!("or" ((a:Bool,b:Bool)->Bool) {a || b}),
        "cond" => atomify!("cond" ((cond:Bool, if_q:Quotation, else_q:Quotation)) {
            let q = if cond { if_q } else { else_q };
            eval_function(Vec::new(), q, env);
        }),
        "if" => |env| {
            if env.loop_like {
                env.using_for_else = true;
            }
            let body = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(body_q), Atom::Bool(cond)) = (body, condition) {
                if cond {
                    eval_function(Vec::new(), body_q, env);
                    if env.loop_like {
                        env.for_else = false;
                    }
                }
            }
        },
        _ => {
            return None;
        }
    })
}

pub fn get_stack_op(op: &str) -> Option<fn(&mut Env)> {
    Some(match op {
        "dup" => |env| {
            let a = env.pop_atom();
            env.push_atom(a.clone());
            env.push_atom(a);
        },
        "drop" => |env| {
            env.pop_atom();
        },
        "swap" => |env| {
            let a = env.pop_atom();
            let b = env.pop_atom();
            env.push_atom(a);
            env.push_atom(b);
        },
        "rot3" => |env| {
            let c = env.pop_atom();
            let b = env.pop_atom();
            let a = env.pop_atom();
            // a b c -- b c a
            env.push_atom(b);
            env.push_atom(c);
            env.push_atom(a);
        },
        "list" => |env| {
            let q = env.pop_atom();
            env.push_blank(false);
            eval_call(q, env);
            let stack = env.pop().unwrap().stack;
            env.push_atom(Atom::List(stack));
        },
        "map" => |env| {
            env.for_else = true;
            env.loop_like = true;
            let q = env.pop_atom();
            let l = env.pop_atom();
            if let Atom::List(list) = l {
                let new_list = list
                    .iter()
                    .map(|atom| {
                        env.push_blank(false);
                        env.push_atom(atom.clone());
                        eval_call(q.clone(), env);
                        env.pop().unwrap().stack.pop().unwrap()
                    })
                    .collect();
                env.push_atom(Atom::List(new_list));
            } else {
                panic!("Expected list quotation map.");
            }
            env.loop_like = false;
        },
        "reduce" => |env| {
            env.for_else = true;
            env.loop_like = true;
            let q = env.pop_atom();
            let l = env.pop_atom();
            if let Atom::List(mut list) = l {
                env.push_blank(false);
                let mut list = list.drain(..);
                let first = list.next();
                if let Some(a) = first {
                    env.push_atom(a);
                    for atom in list {
                        env.push_atom(atom);
                        eval_call(q.clone(), env);
                    }
                    let res: Vec<Atom> = env.pop().unwrap().stack;
                    env.push_atom(Atom::List(res));
                }
            } else {
                panic!("Expected list quotation reduce.");
            }
            env.loop_like = false;
        },
        "splat" => |env| {
            let l = env.pop_atom();
            if let Atom::List(list) = l {
                env.append_atoms(list)
            } else {
                panic!("Expected list splat.")
            }
        },
        "repeat" => |env| {
            env.for_else = true;
            env.loop_like = true;
            let n = env.pop_atom();
            let q = env.pop_atom();
            if let Atom::Num(times) = n {
                for _ in 0..times {
                    eval_call(q.clone(), env);
                }
            }
            env.loop_like = false;
        },
        "for_else" => |env| {
            if !env.using_for_else {
                panic!("No conditionals used by loop-like combinator.")
            }
            let body = env.pop_atom();
            if let Atom::Quotation(body_q) = body {
                if env.for_else {
                    eval_function(Vec::new(), body_q, env);
                }
            }
        },
        "for_if" => |env| {
            if !env.using_for_else {
                panic!("No conditionals used by loop-like combinator.")
            }
            let body = env.pop_atom();
            if let Atom::Quotation(body_q) = body {
                if !env.for_else {
                    eval_function(Vec::new(), body_q, env);
                }
            }
        },
        "get" => |env| {
            if let Atom::Symbol(ident) = env.pop_atom() {
                match env.find_var(&ident) {
                    Some(Atom::Function(_, body)) => env.push_atom(Atom::Quotation(body)),
                    Some(_) => panic!("Tried to get a non-function."),
                    _ => panic!("Unrecognized identifier: {}", ident),
                }
            }
        },
        _ => {
            return None;
        }
    })
}
