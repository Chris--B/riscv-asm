use gumdrop::Options;

#[derive(Debug, Options)]
struct MyOptions {
    /// Print the help message and exit
    #[options()]
    help: bool,

    /// "Use equivilent psuedo instructions when possible"
    #[options(default = "true")]
    allow_psuedo: bool,

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

    let code: Vec<u32> = riscv_asm::parse_elf_from_path(&opts.input)?;

    // Text
    println!("ASM:");

    for (idx, word) in code.iter().cloned().enumerate() {
        // Address of instruction
        print!("  0x{:>03x}:    ", std::mem::size_of::<u32>() * idx);

        // Raw bytes of instruction
        for byte in word.to_le_bytes().iter() {
            print!("{:02x} ", byte);
        }

        // Instruction as text
        let o_instr = riscv_asm::decode_opcode(word);
        let instr_text = if let Some(instr) = o_instr {
            format!("{:?}", instr)
        } else {
            format!("0x{:08}, 0b_{:b}", word, word)
        };

        print!("   {:<25}", instr_text);

        println!();
    }
    println!();

    Ok(())
}
