use crate::types::{Atom, Env};
use crate::eval::eval_atom;

macro_rules! eval_op {
    ( $op:tt, $wrap:path, $env:ident ) => {
        {
            let b = $env.pop_atom();
            let a = $env.pop_atom();
            if let ($wrap(unwrapped_a), $wrap(unwrapped_b)) = (a,b) {
                $env.push_atom($wrap(unwrapped_a $op unwrapped_b));
            } else {
                panic!("Wrong number or types of arguments for operation.");
            };
        }
    };
    ( $op:tt, $wrap:path, $wrap2:path, $env:ident ) => {
        {
            let b = $env.pop_atom();
            let a = $env.pop_atom();
            if let ($wrap(unwrapped_a), $wrap(unwrapped_b)) = (a,b) {
                $env.push_atom($wrap2(unwrapped_a $op unwrapped_b));
            } else {
                panic!("Wrong number or types of arguments for operation.");
            };
        }
    };
}

pub fn eval_arithmetic_op(s: &str, env: &mut Env) {
    match s {
        "+" => eval_op!(+, Atom::Num, env),
        "-" => eval_op!(-, Atom::Num, env),
        "*" => eval_op!(*, Atom::Num, env),
        "/" => eval_op!(/, Atom::Num, env),
        "%" => eval_op!(%, Atom::Num, env),
        "<" => eval_op!(<, Atom::Num, Atom::Bool, env),
        ">" => eval_op!(>, Atom::Num, Atom::Bool, env),
        "<=" => eval_op!(<=, Atom::Num, Atom::Bool, env),
        ">=" => eval_op!(>=, Atom::Num, Atom::Bool, env),
        "=" => eval_op!(==, Atom::Num, Atom::Bool, env),
        _ => panic!("This should never happen.")
    }
}

pub fn eval_boolean_op(op: &str, env: &mut Env) {
    match op {
        "and" => eval_op!(&&, Atom::Bool, env),
        "or" => eval_op!(||, Atom::Bool, env),
        "cond" => {
            let else_branch = env.pop_atom();
            let if_branch = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(else_q),
                    Atom::Quotation(if_q),
                    Atom::Bool(cond)) = (else_branch, if_branch, condition) {
                if cond {
                    eval_atom(Atom::Function(Vec::new(), if_q), env);
                } else {
                    eval_atom(Atom::Function(Vec::new(), else_q), env);
                }
            }
        },
        _ => panic!("This should never happen.")
    }
}

pub fn eval_stack_op(op: &str, env: &mut Env) {
    match op {
        "dup" => {
            let a = env.pop_atom();
            env.push_atom(a.clone());
            env.push_atom(a);
        },
        "drop" => { env.pop_atom(); },
        "swap" => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            env.push_atom(a);
            env.push_atom(b);
        },
        _ => panic!("This should never happen.")
    }
}
