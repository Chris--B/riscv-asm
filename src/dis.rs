use std::fs;
use std::path::Path;

use goblin::elf::Elf;
use goblin::Object;
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

fn extract_code<'a>(elf: &'a Elf, buffer: &'a [u8]) -> Vec<u32> {
    use goblin::elf::program_header::*;

    let load_headers: Vec<&ProgramHeader> = elf
        .program_headers
        .iter()
        .filter(|hdr| hdr.p_type == PT_LOAD)
        .collect();

    assert_eq!(load_headers.len(), 1);
    let header = load_headers[0];
    assert_eq!(header.p_memsz % 4, 0);

    let start = header.p_offset as usize;
    let end = start + header.p_memsz as usize;
    let bytes = &buffer[start..end];

    bytes
        .chunks_exact(4)
        .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
        .collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = MyOptions::new();
    dbg!(&opts);

    let buffer: Vec<u8> = fs::read(&opts.input)?;
    let elf: Elf = match Object::parse(&buffer)? {
        Object::Elf(elf) => elf,
        Object::PE(_pe) => {
            eprintln!("{}: Expected ELF, found PE", opts.input);
            return Ok(());
        }
        Object::Mach(_mach) => {
            eprintln!("{}: Expected ELF, found MACH", opts.input);
            return Ok(());
        }
        Object::Archive(_archive) => {
            eprintln!("{}: Expected ELF, found ARCHIVE", opts.input);
            return Ok(());
        }
        Object::Unknown(magic) => {
            eprintln!(
                "{}: Expected ELF, found unknown format (magic: {:#x}",
                opts.input, magic
            );
            return Ok(());
        }
    };

    // Dump the entire Object struct to disk to debug it
    let debug_path = format!("{}.debug", opts.output.unwrap());
    fs::write(debug_path, &format!("{:#?}", elf))?;

    let code: Vec<u32> = extract_code(&elf, &buffer);

    // Hex Dump
    const WORDS_PER_LINE: usize = 4;
    for (idx, four_words) in code.as_slice().chunks(WORDS_PER_LINE).enumerate() {
        print!("  0x{:>03x}: ", WORDS_PER_LINE * idx);
        for word in four_words {
            print!("0x{:08x} ", word);
        }
        println!();
    }

    Ok(())
}
