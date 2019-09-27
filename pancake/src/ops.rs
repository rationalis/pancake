use crate::eval::{eval_call, eval_function};
use crate::types::{Arity, Atom, Env};

use Atom::*;

use pancake_macro::{atomify, binops, shuffle};

type OpWithArity = (fn(&mut Env), Arity);

pub fn get_arithmetic_op(op: &str) -> Option<OpWithArity> {
    binops!(a"+" a"-" a"*" a"/" a"%" c"<" c">" c"<=" c">=" c"==" c"!=")
}

pub fn get_boolean_op(op: &str) -> Option<OpWithArity> {
    Some(match op {
        "and" => atomify!("and" ((a:Bool,b:Bool)->Bool) {a && b}),
        "or" => atomify!("or" ((a:Bool,b:Bool)->Bool) {a || b}),
        "cond" => atomify!("cond" ((cond:Bool, if_q:Quotation, else_q:Quotation)) {
            let q = if cond { if_q } else { else_q };
            eval_function(Vec::new(), q, env);
        }),
        "not" => atomify!("not" ((a:Bool)->Bool) {!a}),
        "if" => atomify!("if" ((cond:Bool, body_q:Quotation)) {
            if env.loop_like {
                env.using_for_else = true;
            }
            if cond {
                eval_function(Vec::new(), body_q, env);
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

pub fn get_stack_op(op: &str) -> Option<OpWithArity> {
    Some(match op {
        "drop" => shuffle!(_a -- ),
        "swap" => shuffle!(a b -- b a),
        "rot3" => shuffle!(a b c -- b c a),
        "dup" => (|env| {
            let a = env.pop_atom();
            env.push_atom(a.clone());
            env.push_atom(a);
        }, Some((1, 2))),
        "list" => (|env| {
            let q = env.pop_atom();
            env.push_blank(false);
            eval_call(q, env);
            let stack = env.pop().unwrap().stack;
            env.push_atom(Atom::List(stack));
        }, Some((1, 1))),
        "map" => atomify!("map" ((list:List, q:Quotation)->List) {
            {
                let q = Quotation(q);
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
                let q = Quotation(q);
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
        "repeat" => (|env| {
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
        }, None),
        "for_else" => (|env| {
            if !env.using_for_else {
                panic!("No conditionals used by loop-like combinator.")
            }
            let body = env.pop_atom();
            if let Atom::Quotation(body_q) = body {
                if env.for_else {
                    eval_function(Vec::new(), body_q, env);
                }
            }
        }, None),
        "for_if" => (|env| {
            if !env.using_for_else {
                panic!("No conditionals used by loop-like combinator.")
            }
            let body = env.pop_atom();
            if let Atom::Quotation(body_q) = body {
                if !env.for_else {
                    eval_function(Vec::new(), body_q, env);
                }
            }
        }, None),
        "print" => (|env| {
            println!("{:#?}", env.pop_atom());
        }, Some((1, 0))),
        "debug" => (|env| {
            println!("{:#?}", env);
        }, Some((0, 0))),
        "get" => (|env| {
            if let Atom::Symbol(ident) = env.pop_atom() {
                match env.find_var(&ident) {
                    Some(Atom::Function(_, body)) => env.push_atom(Atom::Quotation(body)),
                    Some(_) => panic!("Tried to get a non-function."),
                    _ => panic!("Unrecognized identifier: {}", ident),
                }
            }
        }, None),
        _ => {
            return None;
        }
    })
}
