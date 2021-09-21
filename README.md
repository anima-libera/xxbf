
# xxbf

Compiler and interpreter for the
[Brainfuck](https://esolangs.org/wiki/Brainfuck) programming language.

## CLI

Cmdline arg | Parameter | Description
----------- | --------- | -----------
`-h` or `--help` | | Prints a help message.
`-v` or `--verbose` | | Prints information maybe useful to debug.
`-s` or `--src` | Brainfuck source code | Takes source code in the cmdline arguments.
`-f` or `--src-file` | Brainfuck file path | Takes source code from the given file.
`-O0` or `--no-optimizations` | | Disables optimizations.
`-c` or `--compile` | | Compile instead of interpreting.
`-i` or `--input` | String | When interpreting, read input from the given string instead of stdin.
`-o` or `--output-file` | File path | When compiling, writes generated code to the given file instead of stdout.

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
