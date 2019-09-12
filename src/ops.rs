use crate::types::{Atom, Env, OpA, OpB, OpS};
use crate::eval::{eval_call, eval_function};

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

pub fn eval_arithmetic_op(op: OpA, env: &mut Env) {
    use OpA::*;
    match op {
        Add => eval_op!(+, Atom::Num, env),
        Sub => eval_op!(-, Atom::Num, env),
        Mult => eval_op!(*, Atom::Num, env),
        Div => eval_op!(/, Atom::Num, env),
        Mod => eval_op!(%, Atom::Num, env),
        Less => eval_op!(<, Atom::Num, Atom::Bool, env),
        Greater => eval_op!(>, Atom::Num, Atom::Bool, env),
        LEq => eval_op!(<=, Atom::Num, Atom::Bool, env),
        GEq => eval_op!(>=, Atom::Num, Atom::Bool, env),
        Eq => eval_op!(==, Atom::Num, Atom::Bool, env),
    }
}

pub fn eval_boolean_op(op: OpB, env: &mut Env) {
    use OpB::*;
    match op {
        And => eval_op!(&&, Atom::Bool, env),
        Or => eval_op!(||, Atom::Bool, env),
        Cond => {
            let else_branch = env.pop_atom();
            let if_branch = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(else_q),
                    Atom::Quotation(if_q),
                    Atom::Bool(cond)) = (else_branch, if_branch, condition) {
                if cond {
                    eval_function(Vec::new(), if_q, env);
                } else {
                    eval_function(Vec::new(), else_q, env);
                }
            }
        },
        If =>  {
            let body = env.pop_atom();
            let condition = env.pop_atom();
            if let (Atom::Quotation(body_q),
                    Atom::Bool(cond)) = (body, condition) {
                if cond {
                    eval_function(Vec::new(), body_q, env);
                }
            }
        }
    }
}

pub fn eval_stack_op(op: OpS, env: &mut Env) {
    use OpS::*;
    match op {
        Dup => {
            let a = env.pop_atom();
            env.push_atom(a.clone());
            env.push_atom(a);
        },
        Drop => { env.pop_atom(); },
        Swap => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            env.push_atom(a);
            env.push_atom(b);
        },
        List => {
            let q = env.pop_atom();
            env.push_blank(false);
            eval_call(q, env);
            let stack = env.pop().unwrap().stack;
            env.push_atom(Atom::List(stack));
        },
        Map => {
            let q = env.pop_atom();
            let l = env.pop_atom();
            if let Atom::List(list) = l {
                let new_list = list.iter().map(|atom| {
                    env.push_blank(false);
                    env.push_atom(atom.clone());
                    eval_call(q.clone(), env);
                    env.pop().unwrap().stack.pop().unwrap()
                }).collect();
                env.push_atom(Atom::List(new_list));
            } else {
                panic!("Expected list for map.");
            }
        },
        Splat => {
            let l = env.pop_atom();
            if let Atom::List(list) = l {
                env.append_atoms(list)
            }
        },
        Repeat => {
            let n = env.pop_atom();
            let q = env.pop_atom();
            if let Atom::Num(times) = n {
                for _ in 0..times {
                    eval_call(q.clone(), env);
                }
            }
        },
        Get => {
            if let Atom::Symbol(ident) = env.pop_atom() {
                match env.find_var(&ident) {
                    Some(Atom::Function(_, body)) =>
                        env.push_atom(Atom::Quotation(body)),
                    Some(_) => panic!("Tried to get a non-function."),
                    _ => panic!("Unrecognized identifier: {}", ident)
                }
            }
        }
    }
}
