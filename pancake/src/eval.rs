use crate::parse::*;
use crate::types::{Atom, Env, Identifier, Stack};

pub fn eval_call(q: Vec<Atom>, env: &mut Env) {
    for atom in q {
        eval_atom(atom, env);
    }
}

pub fn eval_call_quotation(callee: Atom, env: &mut Env) {
    if let Atom::Quotation(q) = callee {
        eval_call(q, env);
    } else {
        panic!("Tried to call a non-quotation.");
    }
}

pub fn eval_call_function(params: Vec<Identifier>, body: Stack, env: &mut Env) {
    if params.is_empty() {
        eval_call(body, env);
    } else {
        env.bind_params(params);
        eval_call(body, env);
        env.unbind_params();
    }
}

/// Take an Atom and evaluate its effect on the stack. For basic primitives,
/// this simply pushes them onto the stack.
pub fn eval_atom(atom: Atom, env: &mut Env) {
    use Atom::*;

    if env.lazy_mode() {
        match atom {
            QuotationStart | QuotationEnd => (),
            Plain(ident) => {
                match env.find_var(&ident) {
                    Some(Function(params, body)) => {
                        env.push_atom(Function(params, body));
                        env.push_atom(Call);
                    }
                    Some(found_atom) => env.push_atom(found_atom),
                    _ => {
                        env.push_atom(Plain(ident));
                    } // free variable
                };
                return;
            }
            _ => {
                env.push_atom(atom);
                return;
            }
        }
    }

    if !env.lazy_mode() {
        if let Quotation(_) | Function(_, _) = atom {
        } else {
            use crate::arity::arity_atom;
            let arity = arity_atom(&atom, env, &mut Vec::new());
            if let Some((num_in, _)) = arity {
                let stack_len = env.last_frame().stack.len();
                if stack_len < num_in as usize {
                    panic!(
                        "{:#?} expected {} arguments but {} were given",
                        atom, num_in, stack_len
                    );
                }
            }
        }
    }

    match atom {
        Bool(_) | Num(_) | Quotation(_) | Symbol(_) | Function(_, _) => env.push_atom(atom),
        Op(op) => {
            (op.f)(env);
        }
        QuotationStart => {
            env.push_blank(true);
        }
        QuotationEnd => {
            let stack: Stack = env.pop().unwrap().stack;
            let quotation = Quotation(stack);
            env.push_atom(quotation);
        }
        DefVar => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let (Symbol(ident), Quotation(expr)) = (a, b) {
                let result_of_expr = eval_with_new_scope(expr, env);
                env.bind_var(&ident, result_of_expr);
            } else {
                unreachable!();
            }
        }
        DefVarLiteral => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let Symbol(ident) = a {
                env.bind_var(&ident, b);
            } else {
                panic!("Expected '<value> <ident> let'.")
            }
        }
        DefFnLiteral => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if let (Symbol(ident), Quotation(q)) = (a, b) {
                env.bind_var(&ident, Function(Vec::new(), q));
            } else {
                panic!("Expected '<quotation> <ident> fn'.")
            }
        }
        Call => {
            match env.pop_atom() {
                Quotation(q) => eval_call(q, env),
                Function(p, b) => eval_call_function(p, b, env),
                _ => {
                    panic!("Tried to call non-quotation.");
                }
            };
        }
        Plain(ident) => match env.find_var(&ident) {
            Some(Function(p, b)) => eval_call_function(p, b, env),
            Some(atom) => env.push_atom(atom),
            _ => panic!("Unrecognized identifier: {}", ident),
        },
        _ => {
            panic!("Unexpected atom type {:#?}", atom);
        }
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
