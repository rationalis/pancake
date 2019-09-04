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
}

pub fn eval_arithmetic_op(c: char, env: &mut Env) {
    match c {
        '+' => eval_op!(+, Atom::Num, env),
        '-' => eval_op!(-, Atom::Num, env),
        '*' => eval_op!(*, Atom::Num, env),
        '/' => eval_op!(/, Atom::Num, env),
        '%' => eval_op!(%, Atom::Num, env),
        _ => panic!("This should never happen.")
    }
}

pub fn eval_boolean_op(op: String, env: &mut Env) {
    match op.as_str() {
        "and" => eval_op!(&&, Atom::Bool, env),
        "or" => eval_op!(||, Atom::Bool, env),
        "cond" => {
            let else_branch = env.pop_atom();
            let if_branch = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(else_q, _),
                    Atom::Quotation(if_q, _),
                    Atom::Bool(cond)) = (else_branch, if_branch, condition) {
                if cond {
                    eval_atom(Atom::Quotation(if_q, true), env);
                } else {
                    eval_atom(Atom::Quotation(else_q, true), env);
                }
            }
        },
        _ => ()
    }
}
