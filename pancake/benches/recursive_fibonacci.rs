#[macro_use]
extern crate criterion;

use criterion::Criterion;

use pancake::eval::eval_program;
use pancake::types::Atom;

fn fibonacci() {
    let expected_out = vec![Atom::Num(55)];
    let mut env = eval_program(
        r"
fn fib = dup 2 <= [ drop 1 ] [ 1 - dup 1 - fib swap fib + ] cond
10 fib",
    );
    assert_eq!(expected_out, env.pop().unwrap().stack);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fib 10", |b| b.iter(|| fibonacci()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
