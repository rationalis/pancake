use regex::Regex;
use crate::types::{ARITHMETIC_OPS, BOOLEAN_OPS, STACK_OPS, Atom, NumType};

pub fn parse_token(token: String) -> Option<Atom> {
    if let Ok(num) = token.parse::<NumType>() {
        return Some(Atom::Num(num));
    }

    if let Ok(c) = token.parse::<char>() {
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

    let atom = match token.as_str() {
        "call" => Atom::Call,
        "let" => Atom::DefOp(false),
        "fn" => Atom::DefOp(true),
        "true" => Atom::Bool(true),
        "false" => Atom::Bool(false),
        "not" => Atom::NotOp,
        s if ARITHMETIC_OPS.contains(&s) => Atom::ArithmeticOp(s.to_string()),
        s if BOOLEAN_OPS.contains(&s) => Atom::BooleanOp(s.to_string()),
        s if STACK_OPS.contains(&s) => Atom::StackOp(s.to_string()),
        _ => Atom::Plain(token)
    };

    Some(atom)
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
