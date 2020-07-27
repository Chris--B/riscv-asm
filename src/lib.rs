use goblin::elf::Elf;

mod decode;
mod instr;

pub use decode::*;
pub use instr::*;

pub fn extract_code<'a>(elf: &'a Elf, buffer: &'a [u8]) -> Vec<u32> {
    for section in &elf.section_headers {
        let name = &elf.shdr_strtab[section.sh_name];

        if name == ".text" {
            let start = section.sh_offset as usize;
            let end = start + section.sh_size as usize;
            let bytes = &buffer[start..end];

            println!("Found {} bytes of code in \".text\"", bytes.len());

            return bytes
                .chunks_exact(4)
                .map(|w| u32::from_le_bytes([w[0], w[1], w[2], w[3]]))
                .collect();
        }
    }

    panic!("No '.text' section in elf")
}
