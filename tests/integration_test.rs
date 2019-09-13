use pancake::eval::eval_program;
use pancake::types::{Atom, Atom::Bool, Atom::Num};

fn assert_prog_output(expected_out: Vec<Atom>, prog: &str) {
    let mut env = eval_program(prog);
    assert_eq!(expected_out, env.pop().unwrap().stack)
}

fn ntoa(v: Vec<i32>) -> Vec<Atom> {
    v.into_iter().map(|n| Num(n)).collect()
}

fn btoa(v: Vec<bool>) -> Vec<Atom> {
    v.into_iter().map(|n| Bool(n)).collect()
}

#[test]
fn one_plus_one() {
    assert_prog_output(ntoa(vec![2]), "1 1 +");
}

#[test]
fn more_complicated_arithmetic() {
    assert_prog_output(ntoa(vec![18]), "1 2 + 3 * 4 * 2 /");
}

#[test]
fn basic_boolean_logic() {
    for &a in [true, false].iter() {
        assert_prog_output(vec![Bool(!a)], format!("{} not", a).as_str());
        for &b in [true, false].iter() {
            assert_prog_output(vec![Bool(a && b)], format!("{} {} and", a, b).as_str());
            assert_prog_output(vec![Bool(a || b)], format!("{} {} or", a, b).as_str());
        }
    }
}

#[test]
fn basic_cond() {
    assert_prog_output(ntoa(vec![2]), "false [ 3 3 + ] [ 1 1 + ] cond");
}

#[test]
fn variable_and_function_definition() {
    assert_prog_output(
        ntoa(vec![35]),
        r"
let a = 17
let b = 18
a b +
",
    );
    assert_prog_output(
        ntoa(vec![3, 7, 6, 8]),
        r"
fn inc = 1 +
fn incinc = inc inc
1 incinc 3 incinc incinc 4 inc inc 5 inc inc inc
",
    );
}

#[test]
fn variable_and_function_definition2() {
    assert_prog_output(
        ntoa(vec![35]),
        r"
17 'a let 18 'b let a b +
",
    );
    assert_prog_output(
        ntoa(vec![3, 7, 6, 8]),
        r"
[ 1 + ] 'inc fn [ inc inc ] 'incinc fn
1 incinc 3 incinc incinc 4 inc inc 5 inc inc inc
",
    );
}

#[test]
fn recursive_fibonacci() {
    assert_prog_output(
        ntoa(vec![1, 1, 2, 3, 5, 8]),
        r"
fn fib = dup 2 <= [ drop 1 ] [ 1 - dup 1 - fib swap fib + ] cond
1 fib 2 fib 3 fib 4 fib 5 fib 6 fib
",
    );
}

#[test]
fn recursive_iterative_fibonacci() {
    assert_prog_output(
        ntoa(vec![1, 1, 2, 3, 5]),
        r"
fn fibn a b c = a 0 > [ a 1 - c b c + fibn ] [ c ] cond
fn fib = 2 - 1 1 fibn
1 fib 2 fib 3 fib 4 fib 5 fib
",
    );
}

#[test]
fn simple_named_param_functions() {
    assert_prog_output(
        ntoa(vec![1, 2, 3, 4, 5, 5]),
        r"
fn f a b c = a b c
fn g a = a a
fn h a b c = a b
1 2 3 4 5 f f g f h g f f
",
    );
}

#[test]
fn whitespace_insensitive() {
    assert_prog_output(
        ntoa(vec![2, 4, 6, 8]),
        r"
[1 1 +] call
[ 2 2 + ] call
[4 4 +][3 3 +] call swap call
",
    );
}

#[test]
fn map_inc() {
    assert_prog_output(
        ntoa(vec![2, 3, 4, 5, 6]),
        r"
[1 2 3 4 5] list [1 +] map splat
",
    );
}

#[test]
fn repeat() {
    assert_prog_output(
        ntoa(vec![10]),
        r"
fn inc = 1 +
0 'inc get 10 repeat
",
    );
}

#[test]
fn iterative_fibonacci() {
    assert_prog_output(
        ntoa(vec![1, 1, 2, 3, 5]),
        r"
fn fib n = 1 1 [ dup rot3 + ] n 2 - repeat swap drop
1 fib 2 fib 3 fib 4 fib 5 fib
",
    );
}

#[test]
fn bubblesort() {
    assert_prog_output(
        ntoa(vec![0, 1, 2, 3, 4, 5]),
        r"
fn fix a b = a b a b > [swap] if
fn bubblesort = [fix] reduce [bubblesort] for_if
[1 3 2 5 4 0]list bubblesort splat
",
    );
}
