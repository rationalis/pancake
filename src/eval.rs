use crate::ops;
use crate::types::{Atom, Stack, Env};
use crate::parse::*;

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

    let mut to_push: Option<Atom> = None;
    match atom {
        Atom::ArithmeticOp(c) => ops::eval_arithmetic_op(c, env),
        Atom::BooleanOp(s) => ops::eval_boolean_op(s, env),
        Atom::StackOp(s) => ops::eval_stack_op(s, env),
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
            to_push = Some(quotation);
        },
        Atom::Function(params, body) => {
            if !params.is_empty() {
                env.bind_params(params);
                env.push_atom(Atom::Quotation(body));
                eval_atom(Atom::Call, env);
                env.unbind_params();
            } else {
                env.push_atom(Atom::Quotation(body));
                eval_atom(Atom::Call, env);
            }
        },
        Atom::DefUnparsedVar(ident, expr) => {
            let result_of_expr = eval_with_new_scope(&expr, env);
            env.bind_var(ident, result_of_expr);
        },
        Atom::DefUnparsedFn(ident, params, mut expr,) => {
            expr = format!("[ {} ]", expr);
            let mut result_of_expr = eval_with_new_scope(&expr, env);
            if let Atom::Quotation(q) = result_of_expr {
                result_of_expr = Atom::Function(params, q);
            } else {
                unreachable!();
            }
            env.bind_var(ident, result_of_expr);
        },
        Atom::DefOp(is_function) => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if is_function {
                if let (Atom::Symbol(ident), Atom::Quotation(q)) = (a, b) {
                    env.bind_var(ident, Atom::Function(Vec::new(), q));
                } else {
                    panic!("Expected '<quotation> <ident> fn'.")
                }
            } else {
                if let Atom::Symbol(ident) = a {
                    env.bind_var(ident, b);
                } else {
                    panic!("Expected '<value> <ident> let'.")
                }
            }
        },
        Atom::Call => {
            if let Atom::Quotation(q) = env.pop_atom() {
                for atom in q {
                    eval_atom(atom, env);
                }
            } else {
                panic!("Tried to call a non-quotation.");
            }
        },
        Atom::Plain(ident) => {
            match env.find_var(&ident) {
                Some(Atom::Function(params, body)) =>
                    eval_atom(Atom::Function(params, body), env),
                Some(atom) => to_push = Some(atom),
                _ => panic!("Unrecognized identifier: {}", ident)
            }
        },
        _ => { to_push = Some(atom); }
    }

    if let Some(atom) = to_push {
        env.push_atom(atom)
    }
}

pub fn eval_with_new_scope(expr: &String, env: &mut Env) -> Atom {
    env.push_blank(false);
    eval_line(&expr, env);
    let mut stack : Stack = env.pop().unwrap().stack;
    if let Some(atom) = stack.pop() {
        return atom;
    } else {
        panic!("Expected result but stack was empty.");
    }
}

pub fn eval_line(line: &String, env: &mut Env) {
    // Early exit for comments
    if let Some('#') = line.chars().next() {
        return;
    }

    // Special handler for def syntax sugar
    if let Some(def_atom) = parse_def(line) {
        eval_atom(def_atom, env);
        return;
    }

    let iter = line.split_ascii_whitespace();
    for token in iter {
        eval_atom(parse_token(token.to_string()), env);
    }
}

pub fn eval_program(program: String) -> Env {
    let mut env = Env::new();
    let lines = program.split('\n');
    for line in lines {
        eval_line(&line.to_string(), &mut env);
        println!("Env: {:#?}", env);
    }
    env
}
