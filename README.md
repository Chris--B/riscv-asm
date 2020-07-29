
## RISC-V Assembler and Disassembler

A WIP, simple-to-use assembler and disassembler for `riscv32i-unknown-none-elf` targets.

[![Build Status Badge][badge-img]][badge-url]

[badge-img]: https://github.com/Chris--b/riscv-asm/workflows/CI/badge.svg?branch=main
[badge-url]: https://github.com/Chris--B/riscv-asm/actions

### Building

The binary target `dis` will disassemble an Elf file passed to it.
```bash
$ cargo run --bin dis ./path/to/riscv32i/bin.elf
```

See the full `--help` output for more options
```bash
$ cargo run --bin dis -- --help
Usage: target/debug/dis [OPTIONS]

Positional arguments:
  input                Path to a RISC-V elf to disassemble

Optional arguments:
  -h, --help           Print the help message and exit
  -a, --allow-pseudo   "Use equivalent pseudo instructions when possible" (default: true)
  -o, --output OUTPUT  Path to write disassembled output into
```
