use crate::parse::*;
use crate::types::{Arity, Atom, Env, Identifier, Stack};

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

pub fn arity(f: &Atom, env: &mut Env) -> Arity {
    use crate::types::Op;
    let cond: Op = Op::new(get_boolean_op("cond").unwrap());

    use crate::ops::get_boolean_op;
    use Atom::*;

    let num_in: u8;
    let quot: &Vec<Atom>;
    if let Function(p, q) = f {
        num_in = p.len() as u8;
        quot = q;
    } else if let Quotation(q) = f {
        num_in = 0;
        quot = q;
    } else {
        panic!("arity called on non-function");
    }

    let mut arities: Vec<Arity> = Vec::new();

    for atom in quot {
        let a: Arity = match atom {
            Bool(_) | Num(_) | Symbol(_) => Some((0, 1)),
            Quotation(_) | Function(_, _) => arity(atom, env),
            // TODO: Handle arities of other control flow combinators
            Op(op) => {
                if cond != op {
                    op.arity
                } else {
                    let a = arities.pop();
                    let b = arities.pop();
                    // We just assume that, if one branch has undefined arity,
                    // the branches must agree.
                    let arity = if let (Some(a), Some(b)) = (a, b) {
                        if a.is_some() && b.is_some() {
                            assert_eq!(a, b);
                            a
                        } else if a.is_some() {
                            a
                        } else if b.is_some() {
                            b
                        } else {
                            None
                        }
                    } else {
                        None
                    };
                    if let Some((num_in, num_out)) = arity {
                        Some((num_in + 1, num_out))
                    } else {
                        None
                    }
                }
            }
            _ => None,
        };
        arities.push(a);
    }

    let mut num_in = num_in;
    let mut num_out: u8 = 0;

    for arity in arities {
        if let Some((in1, out1)) = arity {
            if in1 > num_out {
                num_in += in1 - num_out;
                num_out = out1;
            } else if in1 == num_out {
                num_out = out1;
            } else if in1 < num_out {
                num_out -= in1;
                num_out += out1;
            }
        } else {
            return None;
        }
    }

    Some((num_in, num_out))
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

    use Atom::*;
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
        Call => eval_call(env.pop_atom(), env),
        Plain(ident) => match env.find_var(&ident) {
            Some(Function(params, body)) => eval_function(params, body, env),
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
