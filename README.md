# Lox

Implementation of a Lox interpreter as described by the book [Crafting Interpreters](https://craftinginterpreters.com/). (for fun)

The code was inspired by the original Java-based interpreter that's implemented in the book, but with changes made to make it fit into Rust's memory managed world.

For example, instead of adding a variable resolution pass as described in Chapter 11 of the book, I decided to implement closures with a form of [persistent environments](https://craftinginterpreters.com/resolving-and-binding.html#persistent-environments), where variable values are stored in an arena and referenced by generational indices. Not crazy elegant, but it seems to get the job done.

> Technically this is my [second time](https://github.com/Chriscbr/scheme-to-wasm) writing a compiler-ish thing in Rust. By coding this, I've learned a bit more about the kinds of tools available in Rust like Rc/RefCell's, "take"/"replace" functions, and [arenas](https://en.wikipedia.org/wiki/Region-based_memory_management).

## TODO

- [ ] implement classes