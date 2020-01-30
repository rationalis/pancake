use crate::arity::arity_fn;
use crate::eval::{eval_call, eval_call_function};
use crate::types::Op as O;
use crate::types::Atom;
use crate::vm::Env;

use Atom::*;

use pancake_macro::{atomify, binops, shuffle};

pub fn get_arithmetic_op(op: &str) -> Option<O> {
    binops!(a"+" a"-" a"*" a"/" a"%" c"<" c">" c"<=" c">=" c"==" c"!=")
}

pub fn get_boolean_op(op: &str) -> Option<O> {
    Some(match op {
        "and" => atomify!("and" ((a:Bool,b:Bool)->Bool) {a && b}),
        "or" => atomify!("or" ((a:Bool,b:Bool)->Bool) {a || b}),
        "cond" => atomify!("cond" ((cond:Bool, if_q:Quotation, else_q:Quotation)) {
            let q = if cond { if_q } else { else_q };
            eval_call(q, env);
        }),
        "not" => atomify!("not" ((a:Bool)->Bool) {!a}),
        "if" => atomify!("if" ((cond:Bool, body_q:Quotation)) {
            if env.loop_like {
                env.using_for_else = true;
            }
            if cond {
                eval_call(body_q, env);
                if env.loop_like {
                    env.for_else = false;
                }
            }
        }),
        _ => {
            return None;
        }
    })
}

pub fn get_stack_op(op: &str) -> Option<O> {
    Some(match op {
        "drop" => shuffle!(_a -- ),
        "swap" => shuffle!(a b -- b a),
        "rot3" => shuffle!(a b c -- b c a),
        "dup" => O::new(
            |env| {
                let a = env.pop_atom();
                env.push_atom(a.clone());
                env.push_atom(a);
            },
            Some((1, 2)),
        ),
        "list" => atomify!("list" ((q:Quotation)->List) {
            {
                env.push_blank(false);
                eval_call(q, env);
                let stack = env.pop().unwrap().stack;
                stack
            }
        }),
        "map" => atomify!("map" ((list:List, q:Quotation)->List) {
            {
                env.for_else = true;
                env.loop_like = true;
                let new_list = list
                    .iter()
                    .map(|atom| {
                        env.push_blank(false);
                        env.push_atom(atom.clone());
                        eval_call(q.clone(), env);
                        env.pop().unwrap().stack.pop().unwrap()
                    })
                    .collect();
                env.loop_like = false;
                new_list
            }
        }),
        "reduce_inner" => atomify!("reduce_inner" ((list:List, q:Quotation)->List) {
            {
                env.for_else = true;
                env.loop_like = true;
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
                    env.loop_like = false;
                    res
                } else {
                    env.loop_like = false;
                    Vec::new()
                }
            }
        }),
        "splat" => atomify!("splat" ((list:List)) {env.append_atoms(list)}),
        "repeat" => O::new(
            |env| {
                env.for_else = true;
                env.loop_like = true;
                let n = env.pop_atom();
                let q = env.pop_atom();
                let (p, b) = {
                    if let Quotation(q) = q {
                        (Vec::new(), q)
                    } else if let Function(p, b) = q {
                        (p, b)
                    } else {
                        panic!("Tried to call a non-quotation.");
                    }
                };
                if let Atom::Num(times) = n {
                    for _ in 0..times {
                        eval_call_function(&p, b.clone(), env);
                    }
                }
                env.loop_like = false;
            },
            None,
        ),
        "for_else" => O::new(
            |env| {
                if !env.using_for_else {
                    panic!("No conditionals used by loop-like combinator.")
                }
                let body = env.pop_atom();
                if let Atom::Quotation(body_q) = body {
                    if env.for_else {
                        eval_call(body_q, env);
                    }
                }
            },
            None,
        ),
        "for_if" => O::new(
            |env| {
                if !env.using_for_else {
                    panic!("No conditionals used by loop-like combinator.")
                }
                let body = env.pop_atom();
                if let Atom::Quotation(body_q) = body {
                    if !env.for_else {
                        eval_call(body_q, env);
                    }
                }
            },
            None,
        ),
        "print" => O::new(
            |env| {
                println!("{:#?}", env.pop_atom());
            },
            Some((1, 0)),
        ),
        "debug" => O::new(
            |env| {
                println!("{:#?}", env);
            },
            Some((0, 0)),
        ),
        // TODO: probably want a different syntax for getting functions literally
        "get" => O::new(
            |env| {
                if let Atom::Symbol(ident) = env.pop_atom() {
                    match env.find_var(&ident) {
                        Some(a) => env.push_atom(a),
                        _ => panic!("Unrecognized identifier: {}", ident),
                    }
                }
            },
            None,
        ),
        "keep" => O::new(
            |env| {
                let q = env.pop_atom();
                let arity = arity_fn(&q, env);
                if let Some((num_in, _)) = arity {
                    let last_n: Vec<Atom>;
                    {
                        let stack = &env.last_frame().stack;
                        last_n = stack[stack.len() - num_in as usize..].to_vec();
                    }
                    env.push_blank(false);
                    env.last_frame().stack = last_n;
                    if let Function(p, q) = q {
                        eval_call_function(&p, q, env);
                    } else if let Quotation(q) = q {
                        eval_call(q, env);
                    } else {
                        panic!("Tried to call a non-quotation.")
                    }
                    let a = env.pop_atom();
                    env.pop();
                    env.push_atom(a);
                } else {
                    panic!("Called keep on a quotation of unknown arity.");
                }
            },
            None,
        ),
        "at" => atomify!("at" ((list:List,n:Num)->Any) {
            list[n as usize].clone()
        }),
        "append" => unimplemented!(),
        "curry" => unimplemented!(),
        "cat" => unimplemented!(),
        _ => {
            return None;
        }
    })
}
