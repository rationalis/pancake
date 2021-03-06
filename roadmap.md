# Pancake Roadmap

## Work Queue

- Delineate static and dynamic arity
  - Optimize arity checking a bit if possible
- Add more combinators as I go along
  - Hopefully, arity improvements will allow more powerful general combinators
- Write some docstrings with test examples.
  - I keep saying I should but I don't. ><
- Lists/composites
  - Dictionaries
  - `match`
  - Indexing
  - `any`, `all`
- Functions defined for singular values (ad hoc polymorphism more generally)
- String types / polymorphism, standard string operations (split, substring,
  index, concat, format)
  - Uses of traits like Add requires changing macros, some kind of basic type
    checking

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
  - [x] Named parameters: `fn f a b c = a` = `drop drop`

- [x] Logic / Booleans
  - [x] Booleans: `true` `false` `and` `or` `not`
  - [x] Comparators: `<` `>` `=` `<=` `>=`
  - [x] `A B C cond` where `A` is boolean, `B` and `C` are quotations, is
        semantically "if A then eval B else eval C".
    - [x] `A B if` where `A` is boolean, is semantically "if A then eval B".
    - Both of these consume the boolean before evaluating quotations.

- [ ] Composite Types
  - [x] Lists
    - [x] `[ a b c ] list` evaluates `a b c` and constructs a list from the
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
  - [x] `dup` `swap` `drop` etc. 
    - Not exhaustively implemented.
  - [ ] Function concatenation operator. See [TODO: find the
        link](http://google.com). `2 2 3 3 +,+` = `2 2 + 3 3 +`

- Misc
  - [x] Write some tests so I don't have to manually check things every time
  - [ ] Write some docs that are more organized than this haphazard roadmap
  - [x] Use macros to generalize a bunch of repetitive code
  - [x] Better pretty-print of Env/Stack/Context
  - [ ] Loops/Iteration (of some kind)
  - [x] Comment syntax
  - [ ] I/O that isn't just printing the whole state (maybe `print`?)
