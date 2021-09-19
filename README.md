
# xxbf

Compiler and interpreter for the
[Brainfuck](https://esolangs.org/wiki/Brainfuck) programming language.

## TODO

- Optimize
  - Merge `+` `-` `<` `>` sequences into blocks
  - Propagates compile-time knowledge to the right of the program
  - Propagates information about which cells are used to the left
  - Bruteforce `,` possibilities when it feels doable
    - Multithreading ?
  - Study brainfuck idioms to see what to optimize
    - Read some [Daniel Cristofani's programs](http://brainfuck.org/)
    - See the `ideas` file
- Support different cell sizes and signed cells
- Support ifferent options such as
  - EOF is 0, EOF is -1, EOF does nothing
  - Fixed-size of dynamic tape
  - Behavior when underflowing to the left of the tape
  - Behavior when setting a cell to a value too small/big
  - etc.
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