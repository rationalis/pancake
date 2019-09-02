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

    if token == "call" {
        return Some(Atom::Call);
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
    if env.lazy_mode() && atom != Atom::QuotationEnd {
        env.push_atom(atom);
        return;
    } 

    let mut to_push : Option<Atom> = None;
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
        Atom::Quotation(q, true) => {
            env.push_atom(Atom::Quotation(q, true));
            eval_atom(Atom::Call, env);
        },
        Atom::Call => {
            if let Some(Atom::Quotation(q, _)) = env.last_frame().0.pop() {
                for atom in q {
                    eval_atom(atom, env);
                }
            } else {
                panic!("Tried to call a non-quotation / nothing.");
            }
        },
        Atom::Plain(ident) => {
            match env.find_var(&ident) {
                Some(Atom::Quotation(q, true)) =>
                    eval_atom(Atom::Quotation(q, true), env),
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

pub fn eval_def(line: &String, env: &mut Env) -> bool {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(
                r"^(?P<decl>let|fn) (?P<ident>[a-z]+?) = (?P<expr>.*)").unwrap();
    }
    let captures = RE.captures(line);
    if let Some(caps) = captures {
        // TODO: handle forbidden identifiers
        let decl = caps["decl"].to_string();
        let ident = caps["ident"].to_string();
        let mut expr = caps["expr"].to_string();

        let lazy = decl == "fn";
        if lazy { expr = format!("[ {} ]", expr); }
        let mut result_of_expr = env.eval_with_new_scope(&expr, lazy);
        if lazy {
            if let Atom::Quotation(q, false) = result_of_expr {
                result_of_expr = Atom::Quotation(q, true);
            } else {
                panic!("This should never happen.");
            }
        }
        env.bind_var(ident, result_of_expr);

        true
    } else { false }
}

pub fn eval_line(line: &String, env: &mut Env) {
    if eval_def(line, env) {
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
