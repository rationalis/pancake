# Pancake Language Reference

## Introduction

This is a high-level overview of the language. It's meant to be relatively
comprehensive on the semantics of the language, mainly as a way for me to keep
track of everything at a glance. Therefore, it should be approachable for
anybody with at least intermediate programming experience.

TODO: An advanced reference?

## Notation

`a b c c` == `a b c dup` means `a b c c` results in the same state as `a b c
dup`.

## Basics

Pancake is an interpreted stack-based language. In other words, the memory model
is a simple stack data structure, and if you start up the REPL and input some
values, those values get pushed onto a stack in the order that you input them.
Operations consume elements by popping from the stack, in other words, `7 1 1 +`
== `7 2`.

Currently, the only primitive value types are integers and booleans.

Operations on integers: +, -, *, /, %, <, >, <=, >=, =
Operations on booleans: and, or, not

## Stack Shuffling Combinators

There are some basic operations which can be used to manipulate the stack.

`dup` (duplicate); `1 dup` == `1 1`\
`drop`; `1 2 3 drop` == `1 2`\
`swap`; `1 2 swap` == `2 1`
`rot3`; `1 2 3 rot3` == `2 3 1`

## Quotations

Quotations are a form of lazy evaluation. A quotation begins with `[` and ends
with `]`. A quotation can be `called`. For example:

`1 [1 +] call` == `2`

## Variables and Functions

Sometimes, it can be excessively complicated to keep track of the stack. Because
we can only manipulate the elements at the top of the stack, it quickly becomes
unmanageable to handle even a handful of values. Thus, Pancake supports basic
named variables as follows:

`let a = 1 1 +` (== `let a = 2`)\
`fn increment = 1 +`\
`a increment` == `3`

A few notes:
- Variable definitions must be self-contained expressions, i.e. they cannot
  manipulate the existing stack, and they must consist of a *single* value.
- Function definitions are equivalent to an assignment of a quotation to a
  variable, with the essential difference being that function references
  immediately invoke `call`.
- Variable and function definitions never mutate the stack.
- Variables and functions can never be mutated.

### Functions with Named Parameters

We can define functions with named parameters as follows: `fn add a b = a b +`
== `fn add = +`. Of course, this example is fairly redundant, but the utility of
named parameters is similar to named variables. When we want to handle more than
a few values at a time, it can quickly become unwieldy to do so.

Notes:
- Named arguments are consumed.
- Functions with named parameters cannot manipulate stack elements not captured
  by the named parameters. A simple workaround is adding more named parameters,
  then immediately pushing those parameters to replicate the stack state, e.g.
  `fn drop a b c = a b`.

### Scoping

Because Pancake has named variables, it also supports limited lexical scoping:
functions with named parameters can define variables which do not outlive the
function call, and shadow variables which share the same name. This is also true
of quotations which are fed into operators.

It is *not*, however, true of all quotations. In fact, lexical scoping is the
exception rather than the rule. In general, a quotation is executed with lexical
scoping only when it is not expected to be `call`ed to manipulate the stack
directly. Functions with named parameters can be seen as one exception, because
they cannot manipulate the stack outside of the captured named arguments.

TODO: Document the exact semantics of scoping.

## Control Flow

`cond`: `true q1 q2 cond` == `q1 call`, `false q1 q2 cond` == `q2 call`\
`if`: `true q if` == `q call` (`false q if` is a no-op)\
`repeat`: `0 [1 +] 10 repeat` == `10`

## Lists

`list` takes a quotation, evaluates it within a new scope, and constructs a
heterogeneous list containing the contents of the stack within this new scope.
Example: `[1 1 + 2 2 +] list` == `[2 4] list`

`l q map`: The usual higher-order function `map`.

`l splat`: *Splats* the elements of the list onto the stack, i.e. `[1 2] list
splat` == `1 2`
