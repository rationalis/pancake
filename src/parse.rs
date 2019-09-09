use regex::Regex;
use crate::types::{ARITHMETIC_OPS, BOOLEAN_OPS, STACK_OPS, Atom, NumType};

use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::*,
    character::complete::char as nomchar,
    combinator::{all_consuming, map, map_res, not, opt, recognize},
    multi::{many1, separated_list},
    sequence::{delimited, preceded, terminated, tuple}
};

fn parse_num_nom_(token: &str) -> IResult<&str, Atom> {
    map(
        map_res(
            recognize(tuple(
                (opt(nomchar('-')),
                digit1))),
            |s: &str| s.parse::<NumType>()
        ),
        |n: NumType| Atom::Num(n)
    ) (&token)
}

fn parse_op_(token: &str) -> Atom {
    match token {
        s if ARITHMETIC_OPS.contains(&s) => Atom::ArithmeticOp(s.to_string()),
        _ => unreachable!()
    }
}

fn parse_special_ident_(token: &str) -> Option<Atom> {
    Some(
        match token {
            "call" => Atom::Call,
            "let" => Atom::DefOp(false),
            "fn" => Atom::DefOp(true),
            "true" => Atom::Bool(true),
            "false" => Atom::Bool(false),
            "not" => Atom::NotOp,
            s if BOOLEAN_OPS.contains(&s) => Atom::BooleanOp(s.to_string()),
            s if STACK_OPS.contains(&s) => Atom::StackOp(s.to_string()),
            _ => {return None;}
    })
}

fn parse_op_nom_(token: &str) -> IResult<&str, Atom> {
    map(
        recognize(many1(one_of("+!@#$%^&*()<>,-=_?/.|"))),
        |s: &str| parse_op_(s)
    ) (token)
}

fn parse_special_ident_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        recognize(tuple((alpha1, alphanumeric0))),
        |s: &str|
        if let Some(a) = parse_special_ident_(s) {
            Ok(a)
        } else {
            Err("This should never surface")
        }
    ) (token)
}

fn parse_ident_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(
        recognize(tuple((alpha1, alphanumeric0))),
        |s: &str| {
            let special = not(parse_special_ident_nom_) (s);
            if special.is_ok() {
                Ok(Atom::Plain(s.to_string()))
            } else {
                Err("Unexpected reserved identifier.")
            }
        }
    ) (token)
}

fn parse_symbol_nom_(token: &str) -> IResult<&str, Atom> {
    map(
        preceded(nomchar('\''),
                 parse_ident_nom_),
        |a: Atom|
        if let Atom::Plain(s) = a {
            Atom::Symbol(s)
        } else {
            unreachable!()
        }
    ) (token)
}

fn parse_bracket_nom_(token: &str) -> IResult<&str, Atom> {
    map(
        alt((nomchar('['), nomchar(']'))),
        |c: char| match c {
            '[' => Atom::QuotationStart,
            ']' => Atom::QuotationEnd,
            _ => unreachable!()
        }
    ) (token)
}

fn parse_token_nom_(token: &str) -> IResult<&str, Atom> {
    alt((parse_bracket_nom_, parse_num_nom_, parse_symbol_nom_,
         parse_op_nom_, parse_special_ident_nom_, parse_ident_nom_)
    ) (token)
}

pub fn parse_token(token: String) -> Atom {
    parse_token_nom_(token.as_str()).unwrap().1
}

/// Parse a line definition of a variable like `let a = 100` or `fn inc = 1 +`.
pub fn parse_def(line: &str) -> Option<Atom> {
    if let Some(a) = parse_let(line) {
        Some(a)
    } else if let Some(a) = parse_fn(line) {
        Some(a)
    } else {
        None
    }
}

fn parse_let(line: &str) -> Option<Atom> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(
                r"^let (?P<ident>[a-z]+?) = (?P<expr>.*)").unwrap();
    }
    let captures = RE.captures(line);
    if let Some(caps) = captures {
        // TODO: handle forbidden identifiers
        let ident = caps["ident"].to_string();
        let expr = caps["expr"].to_string();

        return Some(Atom::DefUnparsedVar(ident, expr));
    }
    None

}

fn parse_fn(line: &str) -> Option<Atom> {
    let parser =
        preceded(tag("fn"),
                 terminated(
                     delimited(multispace1,
                               separated_list(multispace1, parse_ident_nom_),
                               multispace1),
                     tag("="))
    );

    let result = parser(line);

    if let Ok((s, v)) = result {
        let expr: &str = s;
        let _: Vec<Atom> = v;
        if let Atom::Plain(ident) = &v[0] {
            return Some(Atom::DefUnparsedFn(
                ident.to_string(),
                v.split_at(1).1.iter().map(
                    |a| if let Atom::Plain(name) = a {
                        name.clone()
                    } else {
                        unreachable!()
                    }).collect(),
                expr.to_string()));
        }
    }

    None
}

pub fn parse_line(line: &str) -> Vec<Atom> {
    // Early exit for comments
    if let Some('#') = line.chars().next() {
        return Vec::new();
    }

    if let Some(atom) = parse_def(line) {
        return vec![atom];
    }

    let parser =
        all_consuming(
            delimited(multispace0,
                      separated_list(opt(multispace1), parse_token_nom_),
                      multispace0));

    let result = parser(line);
    let (_, v) = result.unwrap();
    v
}

#[test]
fn test_parse_fn() {
    assert_ne!(None, parse_fn("fn f a b c = 1 2 3"));
}

#[test]
fn test_special_ident_fail() {
    let test_val = parse_special_ident_nom_("a");
    if let Ok(_) = test_val {
        panic!("Unexpected Ok.")
    }
}

#[test]
fn test_parse_var_name() {
    assert_eq!(Ok(("", Atom::Plain("a".to_string()))), parse_ident_nom_("a"));
}

#[test]
fn test_parse_var_fail() {
    let test_val = parse_ident_nom_("=");
    if let Ok(_) = test_val {
        panic!("Unexpected Ok.")
    }
}
