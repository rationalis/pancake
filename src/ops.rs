use crate::eval::{eval_call, eval_function};
use crate::types::{Atom, Env};

macro_rules! eval_op {
    ( $op:tt, $wrap:path) => {
        {
            |env: &mut Env| {
                let b = env.pop_atom();
                let a = env.pop_atom();
                if let ($wrap(unwrapped_a), $wrap(unwrapped_b)) = (a,b) {
                    env.push_atom($wrap(unwrapped_a $op unwrapped_b));
                } else {
                    panic!("Wrong number or types of arguments for operation.");
                };
            }
        }
    };
    ( $op:tt, $wrap:path, $wrap2:path) => {
        {
            |env: &mut Env| {
                let b = env.pop_atom();
                let a = env.pop_atom();
                if let ($wrap(unwrapped_a), $wrap(unwrapped_b)) = (a,b) {
                    env.push_atom($wrap2(unwrapped_a $op unwrapped_b));
                } else {
                    panic!("Wrong number or types of arguments for operation.");
                };
            }
        }
    };
}

pub fn get_arithmetic_op(op: &str) -> Option<fn(&mut Env)> {
    Some(match op {
        "+" => eval_op!(+, Atom::Num),
        "-" => eval_op!(-, Atom::Num),
        "*" => eval_op!(*, Atom::Num),
        "/" => eval_op!(/, Atom::Num),
        "%" => eval_op!(%, Atom::Num),
        "<" => eval_op!(<, Atom::Num, Atom::Bool),
        ">" => eval_op!(>, Atom::Num, Atom::Bool),
        "<=" => eval_op!(<=, Atom::Num, Atom::Bool),
        ">=" => eval_op!(>=, Atom::Num, Atom::Bool),
        "=" => eval_op!(==, Atom::Num, Atom::Bool),
        _ => {
            return None;
        }
    })
}

pub fn get_boolean_op(op: &str) -> Option<fn(&mut Env)> {
    Some(match op {
        "and" => eval_op!(&&, Atom::Bool),
        "or" => eval_op!(||, Atom::Bool),
        "cond" => |env| {
            let else_branch = env.pop_atom();
            let if_branch = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(else_q), Atom::Quotation(if_q), Atom::Bool(cond)) =
                (else_branch, if_branch, condition)
            {
                if cond {
                    eval_function(Vec::new(), if_q, env);
                } else {
                    eval_function(Vec::new(), else_q, env);
                }
            }
        },
        "if" => |env| {
            let body = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(body_q), Atom::Bool(cond)) = (body, condition) {
                if cond {
                    eval_function(Vec::new(), body_q, env);
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
        "list" => |env| {
            let q = env.pop_atom();
            env.push_blank(false);
            eval_call(q, env);
            let stack = env.pop().unwrap().stack;
            env.push_atom(Atom::List(stack));
        },
        "map" => |env| {
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
                panic!("Expected list for map.");
            }
        },
        "splat" => |env| {
            let l = env.pop_atom();
            if let Atom::List(list) = l {
                env.append_atoms(list)
            }
        },
        "repeat" => |env| {
            let n = env.pop_atom();
            let q = env.pop_atom();
            if let Atom::Num(times) = n {
                for _ in 0..times {
                    eval_call(q.clone(), env);
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
