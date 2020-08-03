
## RISC-V Assembler and Disassembler

A WIP, simple-to-use assembler and disassembler for `riscv32i-unknown-none-elf` targets.

[![Build Status Badge][badge-build-img]][actions-url] [![Test Status Badge][badge-test-img]][actions-url]

[badge-build-img]: https://github.com/Chris--b/riscv-asm/workflows/Build/badge.svg?branch=main
[badge-test-img]: https://github.com/Chris--b/riscv-asm/workflows/Test/badge.svg?branch=main
[actions-url]: https://github.com/Chris--B/riscv-asm/actions

### Building

The binary target `dis` will disassemble an Elf file passed to it.
```bash
$ cargo run --bin dis ./path/to/riscv32i/bin.elf
```

See the full `--help` output for more options
```bash
$ cargo run --bin dis -- --help
riscv-asm 0.0.3-wip

USAGE:
    dis [OPTIONS] <input>

ARGS:
    <input>    
            Path to a RISC-V elf to disassemble

FLAGS:
    -h, --help       
            Prints help information

    -V, --version    
            Prints version information


OPTIONS:
    -o, --output <output>    
            Path to write disassembled output into
            
            If unspecified, this is derived from the input file. If "-" is specified, the output is directed to stdout.
```
