use crate::parse::*;
use crate::types::{Atom, Env, Identifier, Stack};

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
    if env.lazy_mode() && atom != Atom::QuotationStart && atom != Atom::QuotationEnd {
        env.push_atom(atom);
        return;
    }

    match atom {
        Atom::Op(op) => {
            op.f()(env);
        }
        Atom::NotOp => {
            if let Atom::Bool(b) = env.pop_atom() {
                env.push_atom(Atom::Bool(!b));
            } else {
                panic!("Tried to negate a non-boolean.")
            }
        }
        Atom::QuotationStart => {
            env.push_blank(true);
        }
        Atom::QuotationEnd => {
            let stack: Stack = env.pop().unwrap().stack;
            let quotation = Atom::Quotation(stack);
            env.push_atom(quotation);
        }
        Atom::DefVar => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let (Atom::Symbol(ident), Atom::Quotation(expr)) = (a, b) {
                let result_of_expr = eval_with_new_scope(expr, env);
                env.bind_var(&ident, result_of_expr);
            } else {
                unreachable!();
            }
        }
        Atom::DefVarLiteral => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let Atom::Symbol(ident) = a {
                env.bind_var(&ident, b);
            } else {
                panic!("Expected '<value> <ident> let'.")
            }
        }
        Atom::DefFnLiteral => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let (Atom::Symbol(ident), Atom::Quotation(q)) = (a, b) {
                env.bind_var(&ident, Atom::Function(Vec::new(), q));
            } else {
                panic!("Expected '<quotation> <ident> fn'.")
            }
        }
        Atom::Call => eval_call(env.pop_atom(), env),
        Atom::Plain(ident) => match env.find_var(&ident) {
            Some(Atom::Function(params, body)) => eval_function(params, body, env),
            Some(atom) => env.push_atom(atom),
            _ => panic!("Unrecognized identifier: {}", ident),
        },
        _ => env.push_atom(atom),
    }
}

pub fn eval_with_new_scope(expr: Vec<Atom>, env: &mut Env) -> Atom {
    env.push_blank(false);

    for atom in expr {
        eval_atom(atom, env);
    }

    let mut stack: Stack = env.pop().unwrap().stack;
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
