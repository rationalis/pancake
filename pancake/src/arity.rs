use crate::ops::get_boolean_op;
use crate::types::{Arity, Atom, Env};

pub fn arity_atom(atom: &Atom, env: &mut Env, arities: &mut Vec<Arity>) -> Arity {
    let cond = get_boolean_op("cond").unwrap();
    use Atom::*;
    match atom {
        Bool(_) | Num(_) | Symbol(_) => Some((0, 1)),
        Quotation(_) | Function(_, _) => arity_fn(atom, env),
        // TODO: Handle arities of other control flow combinators
        Op(op) => {
            if cond != op {
                op.arity
            } else {
                if arities.len() < 2 {
                    return None;
                }
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
    }
}

pub fn arity_fn(f: &Atom, env: &mut Env) -> Arity {
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
        panic!("arity_fn called on non-function");
    }

    let mut arities: Vec<Arity> = Vec::new();

    for atom in quot {
        let arity = arity_atom(atom, env, &mut arities);
        arities.push(arity);
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
