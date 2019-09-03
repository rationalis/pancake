# Pancake

A simple dynamic, concatenative language made for fun, in the course of learning
Rust.

The name was supposed to be some kind of joke about stacks.

Apparently somebody made a stack-based esolang with this name in 2013 but it
seemed like a meme. The name is up for change if somebody ever complains to me
about it.

## Motivation

I want to learn Rust and design a simple programming language. Why not both at
the same time?

Concatenative languages are interesting, since, in some sense, they exist at an
intersection of functional and imperative programming. We can reason about their
semantics as a transformation of a single, global state, with actual changes
restricted. However, in implementation, we just push and pop from a stack. Nice
and simple.

In fact, it's such a simple model that there's no real need for any complex
parser - everything can be implemented from near-scratch. This allows for plenty
of flexibility in determining syntax and semantics. The fun is in experimenting!

On the other hand, there are plenty of explicit non-goals: compilation,
performance, safety. These are outside of my interest for a new hobby language.

## Work Queue

- Write some docstrings with test examples.
- Check for forbidden identifiers on definitions.
- Some booleans and ops (to allow nontrivial recursive functions)

## Roadmap

- [x] Basic integer arithmetic in a REPL (+,-,*,/)
  - [ ] Better numerics which handle arbitrary precision rationals/floats, and
        converts between them as necessary

- [x] Named variables
  - Assignment syntax: `let var = 100`
  - RHS of assignment must evaluate to a single constant.
  - A direct reference (`var`) immediately pushes the value onto the stack.
    - In reality the top value on the stack after evaluation is what gets
      assigned to the variable.
    - [ ] Actually enforce this restriction.
  - Variables cannot be re-assigned nor are their values mutable. Thus, they are
    only useful for reuse of a value, *not* a mutable shared state.

- [x] Quotations / Functions
  - [x] Quotation: `[ 2 2 + ]`
  - [x] Add stack-of-stacks (-> lexical scoping w/ variable shadowing)
    - [x] Allow nested quotations.
    - [x] Allow lazy `let` and lazy `fn` (-> local functions).
    - [x] ~~Default to eager capture but fall back to late binding (->
          recursion).~~
      - Caveat: This allows name collision in inner scopes which redefine a
        variable.
      - Deferred; since variables are *always* immutable, the above caveat is
        the only time late binding differs from eager capture, other than small
        potential performance gains
      - `[` makes all evaluation lazy. `]` consumes up to the nearest `[` to
        construct the quotation.
  - [x] Implement `call`: evaluates the quotation on the top of the stack
  - [x] Function: `fn sq = 2 ^`
    - Function definitions are implicit quotations. A function reference pushes
      the quotation then calls `call`.
    - Composition/currying for free, hopefully.
  - [ ] Can push the quotation definition without calling `apply`, e.g. `''fn`.

- [ ] Logic / Booleans
  - [ ] Booleans: `true` `false` `and` `or` `not`
  - [ ] Comparators: `<` `>` `=` `<=` `>=`
  - [ ] `A B C cond` where `A` is boolean, `B` and `C` are quotations, is
        semantically "if A then eval B else eval C".
        - [ ] `A B if` where `B` is boolean, is semantically "if A then eval B".
        - Both of these consume the boolean before evaluating quotations.

- [ ] Composite Types
  - [ ] Lists
    - [ ] `[ a b c ] list` evaluates `a b c` and constructs a list from the
          result.
    - [ ] `[ 1 2 3 ] list 4 append = [ 1 2 3 4 ] list`
    - [ ] `map` `fold` `reduce`
    - [ ] Indexing via `.` e.g. `.0`, `.a`
    - [ ] Boolean list convenience functions: `any` `all`
    - [ ] Ranges
  - [ ] Dictionaries
    - [ ] `[ a b c d ] dict` evaluates `a b c d`; if these are 4 atoms, then the
          result is a dictionary containing `(a,b),(c,d)`
    - [ ] A `case` statement which can be used for switch-case, if/else-if, and
          pattern matching.
      - `case` takes a quotation:quotation dict, where the key quotations
      evaluate to bools. It evaluates the conditions (sequentially) and
      evaluates the first value quotation corresponding to a true condition
      (short-circuiting).
      - [ ] `multicase` which does not short-circuit. There are no guarantees
            about evaluation order.
  
- [ ] Operators
  - [x] `let` form which is not on its own line, `1 'a let'` = `^let a = 1`
  - [x] `fn` form "
  - [ ] `dup` `swap` `drop` etc. 
  - [ ] Function concatenation operator. See [TODO: find the
        link](http://google.com). `2 2 3 3 +,+` = `2 2 + 3 3 +`

- Misc
  - [ ] Write some tests so I don't have to manually check things every time
  - [ ] Write some docs that are more organized than this haphazard roadmap
  - [ ] Use macros to generalize a bunch of repetitive code
  - [ ] Add transaction logging to Env to facilitate easier debugging
  - [ ] Loops/Iteration (of some kind)
  - [x] Comment syntax
  - [ ] I/O that isn't just printing the whole state (maybe `print`?)
