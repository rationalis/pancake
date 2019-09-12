use crate::types::{Atom, Stack, Env, Identifier};
use crate::parse::*;

pub fn eval_call(quotation: Atom, env: &mut Env) {
    if let Atom::Quotation(q) = quotation {
        for atom in q {
            eval_atom(atom, env);
        }
    } else {
        panic!("Tried to call a non-quotation.");
    }
}

pub fn eval_function(params: Vec<Identifier>, body: Stack, env: &mut Env) {
    if params.is_empty() {
        eval_call(Atom::Quotation(body), env);
    } else {
        env.bind_params(params);
        eval_call(Atom::Quotation(body), env);
        env.unbind_params();
    }
}

/// Take an Atom and evaluate its effect on the stack. For basic primitives,
/// this simply pushes them onto the stack.
pub fn eval_atom(atom: Atom, env: &mut Env) {
    // Currently Atom::QuotationStart enters lazy mode and Atom::QuotationEnd
    // closes it. If/when it gets more complex there should be a more complex
    // guard here.
    if env.lazy_mode()
        && atom != Atom::QuotationStart
        && atom != Atom::QuotationEnd {

        env.push_atom(atom);
        return;
    } 

    match atom {
        Atom::Op(op) => {op.f()(env);},
        Atom::NotOp => {
            if let Atom::Bool(b) = env.pop_atom() {
                env.push_atom(Atom::Bool(!b));
            } else {
                panic!("Tried to negate a non-boolean.")
            }
        },
        Atom::QuotationStart => {
            env.push_blank(true);
        },
        Atom::QuotationEnd => {
            let stack: Stack = env.pop().unwrap().stack;
            let quotation = Atom::Quotation(stack);
            env.push_atom(quotation);
        },
        Atom::Function(params, body) => {
            eval_function(params, body, env);
        },
        Atom::DefVar => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let (Atom::Symbol(ident), Atom::Quotation(expr)) = (a, b) {
                let result_of_expr = eval_with_new_scope(expr, env);
                env.bind_var(&ident, result_of_expr);
            } else {
                unreachable!();
            }
        },
        Atom::DefFn(params) => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let (Atom::Symbol(ident), Atom::Quotation(expr)) = (a, b) {
                if let Atom::Quotation(q) = make_fn_q(expr, env) {
                    env.bind_var(&ident, Atom::Function(params, q));
                } else {
                    unreachable!();
                }
            } else {
                unreachable!();
            }
        },
        Atom::DefOp(is_function) => {
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
        },
        Atom::Call => eval_call(env.pop_atom(), env),
        Atom::Plain(ident) => {
            match env.find_var(&ident) {
                Some(Atom::Function(params, body)) =>
                    eval_function(params, body, env),
                Some(atom) => env.push_atom(atom),
                _ => panic!("Unrecognized identifier: {}", ident)
            }
        },
        _ => env.push_atom(atom)
    }
}

// TODO: Figure out what's happening here
fn make_fn_q(expr: Vec<Atom>, env: &mut Env) -> Atom {
    // Implementation 1
    // return Atom::Quotation(expr);

    // For some reason, this is the fastest of these three implementations.
    // Except, as far as I can tell, this one should be doing the most work,
    // since it should perform at least as many allocations, but with multiple
    // additional calls to functions that do more stuff.
    eval_atom(Atom::QuotationStart, env);
    for atom in expr {
        eval_atom(atom, env);
    }
    eval_atom(Atom::QuotationEnd, env);
    env.pop_atom()

    // Implementation 3
    // let v = Vec::new();
    // for atom in expr {
    //     v.push(atom);
    // }
    // Atom::Quotation(v)
}

pub fn eval_with_new_scope(expr: Vec<Atom>, env: &mut Env) -> Atom {
    env.push_blank(false);

    for atom in expr {
        eval_atom(atom, env);
    }

    let mut stack : Stack = env.pop().unwrap().stack;
    if let Some(atom) = stack.pop() {
        atom
    } else {
        panic!("Expected result but stack was empty.");
    }
}

pub fn eval_line(line: &str, env: &mut Env) {
    for atom in parse_line(line) {
        eval_atom(atom, env);
    }
}

pub fn eval_program(program: &str) -> Env {
    let mut env = Env::new();
    let lines = program.split('\n');
    for line in lines {
        eval_line(line, &mut env);
    }
    env
}
