use regex::Regex;
use crate::types::{ARITHMETIC_OPS, BOOLEAN_OPS, STACK_OPS, Atom, NumType};

use nom::{
    IResult,
    branch::alt,
    character::complete::*,
    character::complete::char as nomchar,
    combinator::{map_res,opt,recognize},
    multi::many1,
    sequence::tuple
};

fn parse_num_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        map_res(
            recognize(tuple(
                (opt(nomchar('-')),
                digit1))),
            |s: &str| s.parse::<NumType>()
        ),
        |n: NumType| {
            let result: Result<Atom, &str> = Ok(Atom::Num(n));
            result
        }
    ) (&token)
}

fn parse_opident_(token: String) -> Atom {
    match token.as_str() {
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
    }
}

fn parse_op_(token: &str) -> Atom {
    match token {
        s if ARITHMETIC_OPS.contains(&s) => Atom::ArithmeticOp(s.to_string()),
        _ => unreachable!()
    }
}

fn parse_ident_(token: &str) -> Atom {
    match token {
        "call" => Atom::Call,
        "let" => Atom::DefOp(false),
        "fn" => Atom::DefOp(true),
        "true" => Atom::Bool(true),
        "false" => Atom::Bool(false),
        "not" => Atom::NotOp,
        s if BOOLEAN_OPS.contains(&s) => Atom::BooleanOp(s.to_string()),
        s if STACK_OPS.contains(&s) => Atom::StackOp(s.to_string()),
        _ => Atom::Plain(token.to_string())
    }
}

fn parse_op_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        recognize(many1(one_of("+!@#$%^&*()<>,-=_?/.|"))),
        |s: &str| Ok(parse_op_(s)) as Result<Atom, &str>
    ) (token)
}

fn parse_ident_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        recognize(tuple((alpha1, alphanumeric0))),
        |s: &str| Ok(parse_ident_(s)) as Result<Atom, &str>
    ) (token)
}

fn parse_symbol_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        tuple((nomchar('\''),
               recognize(alphanumeric1))),
        |(_,s): (_, &str)|
        Ok(Atom::Symbol(s.to_string())) as Result<Atom, &str>
    ) (token)
}

fn parse_bracket_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        alt((nomchar('['), nomchar(']'))),
        |c: char| {
            let result: Result<Atom, &str> = Ok(
                match c {
                    '[' => Atom::QuotationStart,
                    ']' => Atom::QuotationEnd,
                    _ => unreachable!()
                });
            result
        }
    ) (token)
}

fn parse_token_nom_(token: String) -> Atom {
    alt((parse_bracket_nom_, parse_num_nom_, parse_symbol_nom_,
         parse_op_nom_, parse_ident_nom_)
    ) (token.as_str()).unwrap().1
}

fn parse_token_old_(token: String) -> Option<Atom> {
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

    Some(parse_opident_(token))
}

pub fn parse_token(token: String) -> Atom {
    // parse_token_old(token)
    parse_token_nom_(token)
}

/// Parse a line definition of a variable like `let a = 100` or `fn inc = 1 +`.
pub fn parse_def(line: &str) -> Option<Atom> {
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

