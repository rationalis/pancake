use pancake::types::{Atom, Atom::Bool, Atom::Num};
use pancake::eval::eval_program;

fn assert_prog_output(expected_out: Vec<Atom>, prog: &str) {
    let mut env = eval_program(prog.to_string());
    assert_eq!(expected_out, env.pop().unwrap().0)
}

fn ntoa(v: Vec<i32>) -> Vec<Atom> {
    v.into_iter().map(|n| Num(n)).collect()
}

fn btoa(v: Vec<bool>) -> Vec<Atom> {
    v.into_iter().map(|n| Bool(n)).collect()
}

#[test]
fn one_plus_one() {
    assert_prog_output(ntoa(vec![2]),
                       "1 1 +");
}

#[test]
fn more_complicated_arithmetic() {
    assert_prog_output(ntoa(vec![18]),
                       "1 2 + 3 * 4 * 2 /");
}

#[test]
fn basic_boolean_logic() {
    for &a in [true, false].iter() {
        assert_prog_output(
            vec![Bool(!a)],
            format!("{} not", a).as_str()
        );
        for &b in [true, false].iter() {
            assert_prog_output(
                vec![Bool(a && b)],
                format!("{} {} and", a, b).as_str()
            );
            assert_prog_output(
                vec![Bool(a || b)],
                format!("{} {} or", a, b).as_str()
            );
        }
    }
}

#[test]
fn basic_cond() {
    assert_prog_output(ntoa(vec![2]),
                       "false [ 3 3 + ] [ 1 1 + ] cond");
}

#[test]
fn using_variables() {
    assert_prog_output(ntoa(vec![35]),
                       r"
let a = 17
let b = 18
a b +
");
}

#[test]
fn defining_functions() {
    assert_prog_output(ntoa(vec![3,7,6,8]),
                       r"
fn inc = 1 +
fn incinc = inc inc
1 incinc 3 incinc incinc 4 inc inc 5 inc inc inc
");
}

/*
TODO: dup, bools, cond

#[test]
fn recursive_fibonacci() {
    assert_prog_output(vec![3,7,6,8],
                       r"
fn fib = dup 2 <= [ 1 ] [ 1 - dup 1 - fib fib + ] cond
1 fib 2 fib 3 fib 4 fib 5 fib
");
}
*/

 
/*
#[test]
fn recursive_iterative_fibonacci() {
    assert_prog_output(vec![3,7,6,8],
                       r"
fn fibn a b c = a 0 <= [ a 1 - c b c + fib ] [ c ] cond
fn fib = 2 - 1 1 fibn
1 fib 2 fib 3 fib 4 fib 5 fib
");

}

#[test]
fn iterative_fibonacci() {
    assert_prog_output(vec![3,7,6,8],
                       r"
fn fibo n = [ dup rot3 + ] n repeat swap drop
fn fib n = 1 1 n 2 -
1 fib 2 fib 3 fib 4 fib 5 fib
");

}
*/

