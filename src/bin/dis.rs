use std::fs::File;
use std::io::Write;

use clap::Clap;

#[derive(Debug, Clap)]
#[clap(version)]
struct DisOpts {
    /// Path to a RISC-V elf to disassemble
    input: String,

    /// Path to write disassembled output into
    ///
    /// If unspecified, this is derived from the input file.
    /// If "-" is specified, the output is directed to stdout.
    #[clap(short, long)]
    output: Option<String>,
}

impl DisOpts {
    /// Parse args from argv, resolve extra steps, or exit trying.
    fn new() -> Self {
        let mut opts = DisOpts::parse();

        // Some options have extra rules so we resolve them in a second pass.
        opts.resolve_extras();

        opts
    }

    /// Resolves extra options
    fn resolve_extras(&mut self) {
        use std::path::Path;

        // This path may optionally be specified directly.
        // When it's not, we need use the input file to derive an output.
        if self.output.is_none() {
            let input_path: &Path = &Path::new(&self.input);
            let file_stem: &str = input_path
                .file_stem()
                .expect("Failed to find file stem of input file")
                .to_str()
                .expect("file stem of input file wasn't valid utf8");

            self.output = Some(format!("./{}.s", file_stem));
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = DisOpts::new();
    let dis = riscv_asm::dis::Disassembly::parse_from_elf_path(&opts.input)?;

    // Emulate LLVM disassembly output and write to file.

    // Manage two objects - stdout or a file. We'll only initialize one of these, and reference it through `out`.
    // This is safe because rustc won't let us reference `file` or `stdout` outside of the match block. :)
    // TODO: Hmmmm is this a crate yet?
    let out: &mut dyn Write;
    let mut file: File;
    let mut stdout: std::io::Stdout;

    match opts.output.unwrap().as_str() {
        "-" => {
            stdout = std::io::stdout();
            out = &mut stdout;
        }
        filename => {
            file = File::create(filename)?;
            out = &mut file;
        }
    }

    write!(out, "\n{}:\tfile format ELF32-riscv\n\n\n", &opts.input)?;
    writeln!(out, "Disassembly of section .text:")?;

    for entry in dis.disassembly() {
        if !entry.labels.is_empty() {
            // Add a blank line to separate sections with a label
            writeln!(out)?;

            for label in &entry.labels {
                // Print the entire address, zero-padding included
                writeln!(out, "{addr:08x} {label}:", addr = entry.addr, label = label)?;
            }
        }

        // Address of instruction
        write!(out, "{:8x}: ", entry.addr)?;

        // Raw bytes of instruction
        for byte in entry.bytes.iter() {
            write!(out, "{:02x} ", byte)?;
        }

        // Spacing
        write!(out, "{:17}\t", "")?;

        // Instruction as text
        if let Some(instr) = entry.o_instr {
            // First, print the instruction name
            write!(out, "{}", instr.name())?;

            // Then, follow with any args, comma separated.
            let args = instr.args();

            let mut iter = args.iter();
            if let Some(arg) = iter.next() {
                // No comma preceeding the first item
                write!(out, "\t{}", arg)?;

                for arg in iter {
                    write!(out, ", {}", arg)?;
                }
            }
        } else {
            write!(out, "???")?;
        }

        // Always end the entry with a newline
        writeln!(out)?;
    }

    Ok(())
}
