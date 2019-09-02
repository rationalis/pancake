use regex::Regex;
use crate::types::{ARITHMETIC_OPS, NumType, Atom, Stack, Env};

pub fn eval_token(token: &str) -> Option<Atom> {
    if let Ok(num) = token.parse::<NumType>() {
        return Some(Atom::Num(num));
    }

    if let Ok(c) = token.parse::<char>() {
        if ARITHMETIC_OPS.contains(c) {
            return Some(Atom::ArithmeticOp(c));
        }
        match c {
            '[' => return Some(Atom::QuotationStart),
            ']' => return Some(Atom::QuotationEnd),
            _ => ()
        }
    }

    Some(Atom::Plain(token.to_string()))
}

// TODO: Generalize arity and types using macro (generics are not enough)
// TODO: Gracefully handle insufficient arguments
pub fn eval_binary_op<F>(op: F, stack: &mut Stack) where
    F: Fn(NumType, NumType) -> NumType {

    let b = stack.pop();
    let a = stack.pop();
    if let (Some(Atom::Num(num_a)), Some(Atom::Num(num_b))) = (a,b) {
        stack.push(Atom::Num(op(num_a, num_b)));
    } else {
        panic!("Insufficient arguments for operation.");
    };
}

/// Take an Atom and evaluate its effect on the stack. For basic primitives,
/// this simply pushes them onto the stack.
pub fn eval_atom(atom: Atom, env: &mut Env) {
    let mut to_push : Option<Atom> = None;
    if env.lazy_mode() && atom != Atom::QuotationEnd {
        to_push = Some(atom);
    } else {
        match atom {
            Atom::ArithmeticOp(c) => {
                let ref mut stack = env.last_frame().0;
                match c {
                    '+' => eval_binary_op(|a,b| a+b, stack),
                    '-' => eval_binary_op(|a,b| a-b, stack),
                    '*' => eval_binary_op(|a,b| a*b, stack),
                    '/' => eval_binary_op(|a,b| a/b, stack),
                    '%' => eval_binary_op(|a,b| a%b, stack),
                    _ => ()
                }
            },
            Atom::QuotationStart => {
                env.push_blank(true);
            },
            Atom::QuotationEnd => {
                let stack: Stack = env.pop().unwrap().0;
                let quotation = Atom::Quotation(stack, false);
                to_push = Some(quotation);
            },
            Atom::Plain(ref ident) if ident == "call" => {
                if let Some(Atom::Quotation(quoted_stack, _)) =
                    env.last_frame().0.pop()
                {
                    for atom in quoted_stack { eval_atom(atom, env); }
                } else {
                    panic!("Tried to call a non-quotation / nothing.");
                }
            },
            Atom::Plain(ident) => {
                if let Some(atom) = env.find_var(&ident) {
                    to_push = Some(atom);
                } else {
                    panic!("Unrecognized identifier: {}", ident);
                };
            },
            _ => { to_push = Some(atom); }
        }
    }

    if let Some(atom) = to_push {
        let ref mut stack = env.last_frame().0;
        stack.push(atom);
    }
}

pub fn eval_let(line: &String, env: &mut Env) -> bool {
    lazy_static! {
        static ref RE: Regex =
            Regex::new("let (?P<ident>[a-z]+?) = (?P<expr>.*)").unwrap();
    }
    let captures = RE.captures(line);
    if let Some(caps) = captures {
        // TODO: handle forbidden identifiers
        let ident = caps["ident"].to_string();
        let expr = caps["expr"].to_string();
        let result_of_expr = env.eval_with_new_scope(&expr);
        env.bind_var(ident, result_of_expr);

        true
    } else { false }
}

pub fn eval_line(line: &String, env: &mut Env) {
    if eval_let(line, env) {
        return;
    }

    let iter = line.split_ascii_whitespace();
    for token in iter {
        if let Some(atom) = eval_token(token) {
            eval_atom(atom, env);
        } else {
            panic!("Unrecognized token.");
        };
    }
}

pub fn eval_program(program: String) -> Env {
    let mut env = Env::new();
    let lines = program.split('\n');
    for line in lines {
        eval_line(&line.to_string(), &mut env);
        println!("Env: {:?}", env);
    }
    env
}
