use regex::Regex;
use crate::types::{ARITHMETIC_OPS, NumType, Atom, Stack, Env};

pub fn parse_token(token: String) -> Option<Atom> {
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

    if let Some('\'') = token.chars().next() {
        let (_, ident) = token.split_at(1);
        return Some(Atom::Symbol(ident.to_string()));
    }

    if token == "call" {
        return Some(Atom::Call);
    } else if token == "let" {
        return Some(Atom::DefOp(false));
    } else if token == "fn" {
        return Some(Atom::DefOp(true));
    }

    Some(Atom::Plain(token))
}

// TODO: Generalize arity and types using macro (generics are not enough)
// TODO: Gracefully handle insufficient arguments
pub fn eval_binary_op<F>(op: F, env: &mut Env) where
    F: Fn(NumType, NumType) -> NumType {

    let b = env.pop_atom();
    let a = env.pop_atom();
    if let (Atom::Num(num_a), Atom::Num(num_b)) = (a,b) {
        env.push_atom(Atom::Num(op(num_a, num_b)));
    } else {
        panic!("Insufficient arguments for operation.");
    };
}

/// Take an Atom and evaluate its effect on the stack. For basic primitives,
/// this simply pushes them onto the stack.
pub fn eval_atom(atom: Atom, env: &mut Env) {
    // Currently Atom::QuotationStart enters lazy mode and Atom::QuotationEnd
    // closes it. If/when it gets more complex there should be a more complex
    // guard here.
    if env.lazy_mode() && atom != Atom::QuotationEnd {
        env.push_atom(atom);
        return;
    } 

    let mut to_push : Option<Atom> = None;
    match atom {
        Atom::ArithmeticOp(c) => {
            match c {
                '+' => eval_binary_op(|a,b| a+b, env),
                '-' => eval_binary_op(|a,b| a-b, env),
                '*' => eval_binary_op(|a,b| a*b, env),
                '/' => eval_binary_op(|a,b| a/b, env),
                '%' => eval_binary_op(|a,b| a%b, env),
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
        Atom::DefUnparsed(ident, mut expr, lazy) => {
            if lazy { expr = format!("[ {} ]", expr); }
            let mut result_of_expr = eval_with_new_scope(&expr, env, lazy);
            if lazy {
                if let Atom::Quotation(q, false) = result_of_expr {
                    result_of_expr = Atom::Quotation(q, true);
                } else {
                    panic!("This should never happen.");
                }
            }
            env.bind_var(ident, result_of_expr);
        },
        Atom::DefOp(is_function) => {
            let a = env.pop_atom();
            let b = env.pop_atom();
            if is_function {
                if let (Atom::Symbol(ident), Atom::Quotation(q, _)) = (a, b) {
                    env.bind_var(ident, Atom::Quotation(q, true));
                } else {
                    panic!("Expected '<quotation> <ident> fn'.")
                }
            } else {
                if let (Atom::Symbol(ident), Atom::Num(num)) = (a, b) {
                    env.bind_var(ident, Atom::Num(num));
                } else {
                    panic!("Expected '<num> <ident> let'.")
                }
            }
        },
        Atom::Call => {
            if let Atom::Quotation(q, _) = env.pop_atom() {
                for atom in q {
                    eval_atom(atom, env);
                }
            } else {
                panic!("Tried to call a non-quotation.");
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

pub fn eval_with_new_scope(expr: &String, env: &mut Env, lazy: bool) -> Atom {
    env.push_blank(lazy);
    eval_line(&expr, env);
    let mut stack : Stack = env.pop().unwrap().0;
    if let Some(atom) = stack.pop() {
        return atom;
    } else {
        panic!("Expected result but stack was empty.");
    }
}

/// Parse a line definition of a variable like `let a = 100` or `fn inc = 1 +`.
pub fn parse_def(line: &String) -> Option<Atom> {
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
        let expr = caps["expr"].to_string();

        return Some(Atom::DefUnparsed(ident, expr, decl == "fn"));
    }
    None
}

pub fn eval_line(line: &String, env: &mut Env) {
    // Special handler for def syntax sugar
    if let Some(def_atom) = parse_def(line) {
        eval_atom(def_atom, env);
        return;
    }

    let iter = line.split_ascii_whitespace();
    for token in iter {
        if let Some(atom) = parse_token(token.to_string()) {
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
