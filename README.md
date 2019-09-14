# Pancake

A simple dynamic, concatenative language made for fun, in the course of learning
Rust.

The name was supposed to be some kind of joke about stacks.

Apparently somebody made a stack-based esolang with this name in 2013 but it
seemed like a meme. The name is up for change if somebody ever complains to me
about it.

Fun fact: `pancake` uses the parser library `nom`. No, this joke has not gone
too far.

## Motivation

I want to learn Rust and design a simple programming language. Why not both at
the same time?

Concatenative languages are interesting, since, in some sense, they exist at an
intersection of functional and imperative programming. We can reason about their
semantics as a transformation of a single, global state, with actual changes
restricted. However, in implementation, we just push and pop from a stack. Nice
and simple.

With that in mind, I have a few basic goals (in rough order of priority):

- **Make the language ergonomic.** It should be easy to read and write code in
  Pancake, given experience corresponding to the complexity of the code.
- **Keep the core language relatively simple and small.** Note that this only
  applies semantically. Concatenative languages require many combinators to be
  properly ergonomic, but it should be a goal to use a flexible, simple set of
  them, which can be composed, and it should *usually* be the case that complex
  primitives are semantically equivalent to some combination of simple
  primitives.
- **Make good use of Rust's features and ecosystem for a safe and performant
  implementation.** This is partially a personal goal, as mentioned above, of
  course. However, a clean implementation is also obviously ideal, and I
  wouldn't be using Rust if it didn't lend itself to that.

On the other hand, there are explicit non-goals at present.

- **Static compilation / safety guarantees.** These are just outside the scope
  of my interest at the moment, especially for a language which is mostly being
  designed ad-hoc.
- **Maximizing performance.** Performance needs to be kept at a reasonable
  level, of course. However, making a JIT compiler and/or performing
  micro-optimizations is difficult, and again, outside the scope of my interest.
  Wherever the Rust ecosystem makes it relatively *easy* to get performance
  wins, I will try to do so.

## See Also

- [Reference](reference.md)
- [Roadmap](roadmap.md)
