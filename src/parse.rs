use crate::ops::*;
use crate::types::{Atom, NumType};
use regex::Regex;

use inlinable_string::InlinableString;

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::char as nomchar,
    character::complete::*,
    combinator::{all_consuming, map, map_res, not, opt, recognize},
    multi::{many0, many1, separated_list},
    sequence::{delimited, preceded, terminated, tuple},
    IResult,
};

fn parse_num_nom_(token: &str) -> IResult<&str, Atom> {
    map(
        map_res(recognize(tuple((opt(nomchar('-')), digit1))), |s: &str| {
            s.parse::<NumType>()
        }),
        Atom::Num,
    )(&token)
}

fn parse_op_(token: &str) -> Atom {
    use crate::types::Op;
    if let Some(op) = get_arithmetic_op(token) {
        Atom::Op(Op::new(op))
    } else {
        panic!("Unrecognized operator '{}'", token);
    }
}

fn parse_special_ident_(token: &str) -> Option<Atom> {
    use crate::types::Op;
    Some(match token {
        "call" => Atom::Call,
        "let" => Atom::DefOp(false),
        "fn" => Atom::DefOp(true),
        "true" => Atom::Bool(true),
        "false" => Atom::Bool(false),
        "not" => Atom::NotOp,
        s => {
            if let Some(op) = get_boolean_op(s) {
                Atom::Op(Op::new(op))
            } else if let Some(op) = get_stack_op(s) {
                Atom::Op(Op::new(op))
            } else {
                return None;
            }
        }
    })
}

fn parse_op_nom_(token: &str) -> IResult<&str, Atom> {
    map(
        recognize(many1(one_of("+!@#$%^&*()<>,-=?/.|"))),
        |s: &str| parse_op_(s),
    )(token)
}

fn recognize_ident_nom_(token: &str) -> IResult<&str, &str> {
    let valid_char = alt((alphanumeric1, tag("_")));
    let valid_chars = many0(valid_char);
    let valid_starting = tuple((alpha1, valid_chars));
    recognize(valid_starting)(token)
}

fn parse_special_ident_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(recognize_ident_nom_, |s: &str| {
        if let Some(a) = parse_special_ident_(s) {
            Ok(a)
        } else {
            Err("This should never surface")
        }
    })(token)
}

fn parse_ident_nom_(token: &str) -> IResult<&str, Atom> {
    map_res(recognize_ident_nom_, |s: &str| {
        let special = not(parse_special_ident_nom_)(s);
        if special.is_ok() {
            Ok(Atom::Plain(InlinableString::from(s)))
        } else {
            Err("Unexpected reserved identifier.")
        }
    })(token)
}

fn parse_symbol_nom_(token: &str) -> IResult<&str, Atom> {
    map(preceded(nomchar('\''), parse_ident_nom_), |a: Atom| {
        if let Atom::Plain(s) = a {
            Atom::Symbol(s)
        } else {
            unreachable!()
        }
    })(token)
}

fn parse_bracket_nom_(token: &str) -> IResult<&str, Atom> {
    map(alt((nomchar('['), nomchar(']'))), |c: char| match c {
        '[' => Atom::QuotationStart,
        ']' => Atom::QuotationEnd,
        _ => unreachable!(),
    })(token)
}

fn parse_token_nom_(token: &str) -> IResult<&str, Atom> {
    alt((
        parse_bracket_nom_,
        parse_num_nom_,
        parse_symbol_nom_,
        parse_op_nom_,
        parse_special_ident_nom_,
        parse_ident_nom_,
    ))(token)
}

pub fn parse_token(token: &str) -> Atom {
    parse_token_nom_(token).unwrap().1
}

/// Parse a line definition of a variable like `let a = 100` or `fn inc = 1 +`.
pub fn parse_def(line: &str) -> Option<Vec<Atom>> {
    if let Some(a) = parse_let(line) {
        Some(a)
    } else if let Some(a) = parse_fn(line) {
        Some(a)
    } else {
        None
    }
}

fn parse_expr(expr: &str) -> Vec<Atom> {
    let parser = all_consuming(delimited(
        multispace0,
        many0(terminated(parse_token_nom_, opt(multispace1))),
        multispace0,
    ));

    let result = parser(expr);
    let (_, v) = result.unwrap();
    v
}

fn parse_let(line: &str) -> Option<Vec<Atom>> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r"^let (?P<ident>[a-z_]+?) = (?P<expr>.*)").unwrap();
    }
    let captures = RE.captures(line);
    if let Some(caps) = captures {
        // TODO: handle forbidden identifiers
        let ident = InlinableString::from(&caps["ident"]);
        let expr = &caps["expr"];

        return Some(vec![
            Atom::Quotation(parse_expr(expr)),
            Atom::Symbol(ident),
            Atom::DefVar,
        ]);
    }
    None
}

fn parse_fn(line: &str) -> Option<Vec<Atom>> {
    let parser = preceded(
        tag("fn"),
        terminated(
            delimited(
                multispace1,
                separated_list(multispace1, parse_ident_nom_),
                multispace1,
            ),
            tag("="),
        ),
    );

    let result = parser(line);

    if let Ok((s, mut v)) = result {
        let expr: &str = s;
        let mut d = v.drain(..);
        let v0: Atom = d.next().unwrap();

        if let Atom::Plain(ident) = v0 {
            return Some(vec![
                Atom::Quotation(parse_expr(expr)),
                Atom::Symbol(ident),
                Atom::DefFn(
                    d.map(|a| {
                        if let Atom::Plain(name) = a {
                            name
                        } else {
                            unreachable!()
                        }
                    })
                    .collect(),
                ),
            ]);
        }
    }

    None
}

pub fn parse_line(line: &str) -> Vec<Atom> {
    // Early exit for comments
    if let Some('#') = line.chars().next() {
        return Vec::new();
    }

    if let Some(v) = parse_def(line) {
        return v;
    }

    parse_expr(line)
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
    assert_eq!(
        Ok(("", Atom::Plain(InlinableString::from("a")))),
        parse_ident_nom_("a")
    );
}

#[test]
fn test_parse_var_fail() {
    let test_val = parse_ident_nom_("=");
    if let Ok(_) = test_val {
        panic!("Unexpected Ok.")
    }
}
