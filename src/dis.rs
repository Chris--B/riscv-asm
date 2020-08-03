use std::collections::HashMap;
use std::fs;
use std::path::Path;

use goblin::{elf::Elf, Object};

use crate::prelude::*;

/// Object that contains a full disassembly of a riscv program
///
/// This object can be obtained from a binary or elf file ("disassembled"),
/// and can then be written back to a file ("assembled").
/// Assembly can fail with link errors if symbols are referenced without a
/// definition.
pub struct Disassembly {
    entries: HashMap<u32, Entry>,
}

#[derive(Clone, Debug)]
pub struct Entry {
    pub addr: u32,
    pub word: u32,
    pub bytes: [u8; 4],
    pub o_instr: Option<Instr>,
    pub labels: Vec<String>,
}

impl Disassembly {
    /// Parse a disassembly from an elf file on disk
    pub fn parse_from_elf_path<P: AsRef<Path>>(path: P) -> Result<Self> {
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

        Self::parse_from_elf(&elf, &buffer)
    }

    /// Parse a disassembly from an elf object
    ///
    /// `elf` must have been parsed from `buffer`.
    fn parse_from_elf<'a>(elf: &'a Elf, buffer: &'a [u8]) -> Result<Self> {
        // First off, we need to isolate the '.text' section.
        //
        // We expect to find exactly one ".text" section.
        // If there are multiple (is that allowed?), we will ignore them.
        // We'll use this section index to correlate symbols to the
        // .text section, later.
        let (text_shndx, section) =
            match elf
                .section_headers
                .iter()
                .enumerate()
                .find(|(_idx, section)| {
                    let name = &elf.shdr_strtab[section.sh_name];
                    name == ".text"
                }) {
                Some(pair) => pair,
                None => {
                    // TODO: return an error
                    panic!("No '.text' section in elf")
                }
            };

        // The '.text' section contains the executable code that we will load
        // into the Disassembly object, so we need to extract and parse the
        // bytes into instructions.
        let start = section.sh_offset as usize;
        let end = start + section.sh_size as usize;
        let bytes = &buffer[start..=end];

        // riscv32i instructions are always exactly 32-bits, stored in little
        // Endian regardless of the endianness of the target machine.
        let words: Vec<u32> = bytes
            .chunks_exact(4)
            .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
            .collect();

        let instrs: Vec<Option<Instr>> = words
            .iter()
            .cloned()
            .map(crate::decode::decode_opcode)
            .collect();

        assert_eq!(words.len(), instrs.len());

        // TODO: Check the elf - I'm pretty sure the elf can specify that
        // the binary loads at a non-zero v/paddr...
        let addr_base = 0;

        let mut entries = HashMap::new();

        for (i, (word, o_instr)) in words.into_iter().zip(instrs).enumerate() {
            let addr = (core::mem::size_of::<u32>() * i) as u32 + addr_base;

            let entry = Entry {
                addr,
                word,
                bytes: word.to_le_bytes(),
                o_instr,
                labels: vec![],
            };
            entries.insert(addr, entry);
        }

        // Find the symbols (labels) that we need to disassamble from
        // the symbols table in the elf.
        // dbg!(elf.strtab.to_vec());
        for sym in &elf.syms {
            let name = &elf.strtab[sym.st_name];

            // Skip empty symbols (what are they even doing here?)
            if name.trim().is_empty() {
                continue;
            }

            // Limit our symbols to those referencing the text section
            if sym.st_shndx != text_shndx {
                continue;
            }

            // This is the address (within the .text section) that the symbol
            // references.
            let addr = sym.st_value as u32;

            if let Some(entry) = entries.get_mut(&addr) {
                entry.labels.push(name.to_string());
            }
        }

        Ok(Disassembly { entries })
    }

    pub fn disassembly(&self) -> impl Iterator<Item = &Entry> {
        #![allow(unreachable_code)]

        // The addresses stored are expected to be contiguous
        let addr_min: u32 = *self.entries.keys().min().unwrap_or(&0);
        let addr_max: u32 = *self.entries.keys().max().unwrap_or(&0);

        // So step 4 at a time - all instructions are 4 byte-aligned.
        // (Are labels?)
        (addr_min..=addr_max)
            .step_by(4)
            .map(move |addr| &self.entries[&addr])
    }
}
