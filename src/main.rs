#[macro_use] extern crate lazy_static;
//#[macro_use] extern crate im_rc;

use std::io;
use std::collections::HashMap;
use regex::Regex;
//use im_rc::vector::Vector;


type NumType = i32;
type IsFunction = bool;

type Stack = Vec<Atom>;
type Context = HashMap<String, Atom>;
type Frame = (Stack, Context, bool);
//type Env = Vec<Frame>;


const SPECIAL_CHARS : &str = "+-*/%[]";
const ARITHMETIC_OPS : &str = "+-*/%";

#[derive(Debug, Clone)]
enum Atom {
    Num(NumType),
    ArithmeticOp(char),
    QuotationStart, // [
    QuotationEnd, // ]
    FunctionEnd, // implicit ]
    Quotation(Vec<Atom>, IsFunction),
    Plain(String)
}

#[derive(Debug)]
struct Env(Vec<Frame>);

impl Env {
    fn new() -> Env {
        Env(vec![(Stack::new(), Context::new(), false)])
    }

    fn last_frame(&mut self) -> &mut Frame {
        if let Some(frame) = self.0.last_mut() {
            return frame;
        } else {
            panic!("Tried to get a frame from an empty stack.")
        }
    }

    fn push_blank(&mut self, lazy: bool) {
        let frame = (Stack::new(), Context::new(), lazy);
        self.0.push(frame)
    }

    fn pop(&mut self) -> Option<Frame> {
        self.0.pop()
    }

    fn find_var(&mut self, ident: &String) -> Option<Atom> {
        for frame in self.0.iter().rev() {
            let context = &frame.1;
            if let Some(atom) = context.get(ident) {
                return Some(atom.clone());
            }
        }
        None
    }
}

fn eval_token(token: &str) -> Option<Atom> {
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
fn eval_binary_op<F>(op: F, stack: &mut Stack) where
    F: Fn(NumType, NumType) -> NumType {

    let b = stack.pop();
    let a = stack.pop();
    if let (Some(Atom::Num(num_a)), Some(Atom::Num(num_b))) = (a,b) {
        stack.push(Atom::Num(op(num_a, num_b)));
    } else {
        panic!("Insufficient arguments for operation.");
    };
}

/// Take an Atom and evaluate its effect on the stack. For basic primitives, this
/// simply pushes them onto the stack.
fn eval_atom(atom: Atom, env: &mut Env) {
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
        Atom::Plain(ident) => {
            if let Some(atom) = env.find_var(&ident) {
                to_push = Some(atom);
            } else {
                panic!("Unrecognized identifier: {}", ident);
            };
        },
        _ => { to_push = Some(atom); }
    }

    let ref mut stack = env.last_frame().0;
    if let Some(atom) = to_push {
        stack.push(atom);
    }
}

fn eval_let(line: &String, env: &mut Env) -> bool {
    lazy_static! {
        static ref RE: Regex =
            Regex::new("let (?P<ident>[a-z]+?) = (?P<expr>.*)").unwrap();
    }
    let captures = RE.captures(line);
    if let Some(caps) = captures {
        // TODO: handle forbidden identifiers
        let ident = caps["ident"].to_string();

        let context : &Context = &env.last_frame().1;
        if context.contains_key(&ident) {
            panic!("Attempted to rebind existing variable.");
        }

        let expr = caps["expr"].to_string();

        env.push_blank(false);
        eval_line(&expr, env);
        let mut stack : Stack = env.pop().unwrap().0;
        let result_of_expr : Atom = stack.pop().unwrap();

        let ref mut context : Context = env.last_frame().1;
        context.insert(ident, result_of_expr);

        true
    } else { false }
}

fn eval_line(line: &String, env: &mut Env) {
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

fn eval_program(program: String) -> Env {
    let mut env = Env::new();
    let lines = program.split('\n');
    for line in lines {
        eval_line(&line.to_string(), &mut env);
        println!("Env: {:?}", env);
    }
    env
}

fn main() {
    let mut env = Env::new();
    loop {
        let mut line = String::new();

        io::stdin().read_line(&mut line)
            .expect("Failed to read line");

        eval_line(&line, &mut env);

        println!("Env: {:?}", env);
    }
}
