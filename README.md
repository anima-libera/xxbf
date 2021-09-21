
# xxbf

Compiler and interpreter for the
[Brainfuck](https://esolangs.org/wiki/Brainfuck) programming language.

## TODO

- Optimize
  - Propagates compile-time knowledge forward in the program
  - Propagates information about which cells are used backward in the program
  - Bruteforce `,` possibilities when it feels doable
    - Multithreading ?
  - Study brainfuck idioms to see what to optimize
    - Read some [Daniel Cristofani's programs](http://brainfuck.org/)
    - See the `ideas` file
- Support different cell sizes and signed cells
- Support different options such as
  - EOF is 0, EOF is -1, EOF does nothing
  - Fixed-size of dynamic tape
  - Behavior when underflowing to the left of the tape
  - Behavior when setting a cell to a value too small/big
  - Comma can receive any byte or only "typable" ascii character values and EOF
  - etc.
- Add test suite support
- Add per-program and per-directory config (cmd line config overwrite these)
- Support the `program!input` format
- Add a debugger
  - Support the `#` breakpoint debugging instruction
  - Add command line vm interaction (to use when hitting breakpoints)
  - Add a vm state visualizer
- Add warnings for code that could be shortened or removed
- Add warnings for compile-time known undefined behavior
- Support interoperability with target languages
- Add an LLVM backend
- Add more brainfuck programs (but no stealing)
