use std::fs::File;

use gumdrop::Options;

#[derive(Debug, Options)]
struct MyOptions {
    /// Print the help message and exit
    #[options()]
    help: bool,

    /// "Use equivalent pseudo instructions when possible"
    #[options(default = "true")]
    allow_pseudo: bool,

    /// Path to a RISC-V elf to disassemble
    #[options(free)]
    input: String,

    /// Path to write disassembled output into
    ///
    /// If unspecified, this is derived from the input file
    #[options()]
    output: Option<String>,
}

impl MyOptions {
    /// Parse args from argv, resolve extra steps, or exit trying.
    fn new() -> Self {
        let mut opts = MyOptions::parse_args_default_or_exit();
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
    let opts = MyOptions::new();
    dbg!(&opts);

    use riscv_asm::dis::Disassembly;

    let dis = Disassembly::parse_from_elf_path(&opts.input)?;

    // Emulate LLVM disassembly output and write to file.
    use std::io::Write;
    let mut file = File::create(opts.output.unwrap())?;

    // We cannot write an escaped character (\t) in a string literal, so we must use the format string.
    // Generally this is wasteful, but this is a raw string so escaped strings aren't processed.
    #[allow(clippy::write_literal)]
    writeln!(
        file,
        r#"
{input}:{tab}file format ELF32-riscv


Disassembly of section .text:"#,
        tab = "\t",
        input = &opts.input
    )?;

    for entry in dis.disassembly() {
        if !entry.labels.is_empty() {
            // Add a blank line to separate sections with a label
            writeln!(file)?;

            for label in &entry.labels {
                // Print the entire address, zero-padding included
                writeln!(
                    file,
                    "{addr:08x} {label}:",
                    addr = entry.addr,
                    label = label
                )?;
            }
        }

        // Address of instruction
        write!(file, "{:8x}: ", entry.addr)?;

        // Raw bytes of instruction
        for byte in entry.bytes.iter() {
            write!(file, "{:02x} ", byte)?;
        }

        // Spacing
        write!(file, "{:17}\t", "")?;

        // Instruction as text
        if let Some(instr) = entry.o_instr {
            write!(file, "{:?}", instr)?;
        } else {
            write!(file, "???")?;
        }

        // Always end the entry with a newline
        writeln!(file)?;
    }

    Ok(())
}
