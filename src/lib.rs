use std::fs;
use std::path::Path;

use goblin::{elf::Elf, Object};

mod decode;
mod instr;

pub use decode::*;
pub use instr::*;

// TODO: Add an error type
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Parse an elf file from disk and return the executable code as `u32` words
pub fn parse_elf_from_path<P: AsRef<Path>>(path: P) -> Result<Vec<u32>> {
    let path: &Path = path.as_ref();
    let buffer: Vec<u8> = fs::read(&path)?;
    let elf: Elf = match Object::parse(&buffer)? {
        Object::Elf(elf) => elf,
        Object::PE(_pe) => {
            // TODO: Return an error
            panic!("{}: Expected ELF, found PE", path.to_string_lossy());
        }
        Object::Mach(_mach) => {
            // TODO: Return an error
            panic!("{}: Expected ELF, found MACH", path.to_string_lossy());
        }
        Object::Archive(_archive) => {
            // TODO: Return an error
            panic!("{}: Expected ELF, found ARCHIVE", path.to_string_lossy());
        }
        Object::Unknown(magic) => {
            // TODO: Return an error
            panic!(
                "{}: Expected ELF, found unknown format (magic: {:#x}",
                path.to_string_lossy(),
                magic
            );
        }
    };

    parse_elf(&elf, &buffer)
}

/// Parse an elf file and return the executable code as `u32` words
pub fn parse_elf<'a>(elf: &'a Elf, buffer: &'a [u8]) -> Result<Vec<u32>> {
    for section in &elf.section_headers {
        let name = &elf.shdr_strtab[section.sh_name];

        if name == ".text" {
            let start = section.sh_offset as usize;
            let end = start + section.sh_size as usize;
            let bytes = &buffer[start..end];

            println!("Found {} bytes of code in \".text\"", bytes.len());

            return Ok(bytes
                .chunks_exact(4)
                .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
                .collect());
        }
    }

    // TODO: Error "no .tet section found - does this elf contain code?"
    panic!("No '.text' section in elf")
}
