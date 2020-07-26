#![allow(clippy::inconsistent_digit_grouping)]
#![deny(unreachable_patterns)]

use std::fmt;
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

struct Word(u32);

impl Word {
    /// Extract the bits `lo` through `hi`, inclusive, and then shift them to the 0 position.
    fn bits(&self, hi: u8, lo: u8) -> u32 {
        let hi: u32 = hi as u32;
        let mask = u32::MAX >> (31 - hi);

        (self.0 & mask) >> lo
    }
}

#[test]
fn check_word_bits_1() {
    let w = Word(0xdead_beef);
    // To help visualize:
    assert_eq!(w.0, 0b_11011110101011011011111011101111);

    // Sanity check:
    const AWKWARD: u32 = 0b_0110_1101_1111_0111;
    assert_eq!(w.0 & (AWKWARD << 5), AWKWARD << 5);

    for (hi, lo, expect) in [
        (0_u8, 0_u8, 1_u32), // Individual Bits
        (0, 0, 1),           // ┌ 0xf
        (1, 1, 1),           // │
        (2, 2, 1),           // │
        (3, 3, 1),           // └
        (4, 4, 0),           // ┌ 0xe
        (5, 5, 1),           // │
        (6, 6, 1),           // │
        (7, 7, 1),           // └
        (8, 8, 0),           // ┌ 0xe
        (9, 9, 1),           // │
        (10, 10, 1),         // │
        (11, 11, 1),         // └
        (31, 16, 0xdead),    // High 2 bytes
        (16, 31, 0x0),       // High 2 bytes backwards
        (15, 0, 0xbeef),     // Low 2 bytes
        (31, 24, 0xde),      // High byte
        (23, 16, 0xad),      // 2nd high byte
        (15, 8, 0xbe),       // 2nd low byte
        (7, 0, 0xef),        // Low byte
        (31, 0, w.0),        // Full range
        (20, 5, AWKWARD),    // "Awkward" range that crosses bytes
    ]
    .iter()
    .cloned()
    {
        let actual = w.bits(hi, lo);

        let label_actual = format!("bits({hi}, {lo})", hi = hi, lo = lo,);
        let label_expect = format!("0x{expect:x}", expect = expect,);

        assert_eq!(
            actual,
            expect,
            concat!(
                "\n",
                "   bits({hi}, {lo}) != 0x{expect:x}\n",
                "       {label_actual:<12} == 0x{actual:08x} == 0b{actual:032b}\n",
                "       {label_expect:<12} == 0x{expect:08x} == 0b{expect:032b}\n"
            ),
            hi = hi,
            lo = lo,
            label_expect = label_expect,
            expect = expect,
            label_actual = label_actual,
            actual = actual
        );
    }
}

#[derive(Copy, Clone)]
struct UnknownInstr {
    word: u32,
    opcode: u32,
    rd: u32,
    funct3: u32,
    rs1: u32,
    rs2: u32,
    funct7: u32,
    funct12: u32,
}

fn fmt_hex_bin(x: u32) -> String {
    format!("(0x{:x}, 0b_{:b})", x, x)
}

impl fmt::Debug for UnknownInstr {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("UnknownInstr")
            .field("word", &fmt_hex_bin(self.word))
            .field("opcode", &fmt_hex_bin(self.opcode))
            .field("rd", &fmt_hex_bin(self.rd))
            .field("funct3", &fmt_hex_bin(self.funct3))
            .field("rs1", &fmt_hex_bin(self.rs1))
            .field("rs2", &fmt_hex_bin(self.rs2))
            .field("funct7", &fmt_hex_bin(self.funct7))
            .field("funct12", &fmt_hex_bin(self.funct12))
            .finish()
    }
}

#[allow(unused_variables)]
fn decode_opcode(w: Word) -> String {
    /*
      Different instructions may use different named fields in the enoding,
    and not all fields are always used. Many fields overlap.
    However, if two instructions use the same field name, that field is
    located in the same location in the word for both instructions.

    RISC-V Instruction Encodings by type:
    (note: funct3 is abbreviated as f3)

        R-type
         0                   1                   2                   3
         0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
        |    opcode   |    rd   | f3  |   rs1   |   rs2   |    funct7   |
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

        I-type
         0                   1                   2                   3
         0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
        |    opcode   |    rd   | f3  |   rs1   |       imm[11;0]       |
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

        S-type
         0                   1                   2                   3
         0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
        |    opcode   | imm[4;0]| f3  |   rs1   |   rs2   |  imm[11;5]  |
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

        U-type
         0                   1                   2                   3
         0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
        |    opcode   |    rd   |               imm[31;12]              |
        +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+

    */

    // We extract each field value here, then reference them in the
    /// larger match block below.
    let opcode = w.bits(6, 0);
    let rd = w.bits(11, 7);
    let funct3 = w.bits(14, 12);
    let rs1 = w.bits(19, 15);
    let rs2 = w.bits(24, 20);
    let funct7 = w.bits(31, 25);
    let funct12 = w.bits(31, 20);

    assert!(funct3 < (1 << 3));
    assert!(funct7 < (1 << 7));

    match (opcode, funct3) {
        // Load Instructions
        (0x03, 0x0) => ("lb").into(),
        (0x03, 0x1) => ("lh").into(),
        (0x03, 0x2) => ("lw").into(),
        (0x03, 0x3) => ("ld").into(),
        (0x03, 0x4) => ("lbu").into(),
        (0x03, 0x5) => ("lhu").into(),
        (0x03, 0x6) => ("lwu").into(),

        // Fences
        (0x0f, 0x0) => ("fence").into(),
        (0x0f, 0x1) => ("fence.i").into(),

        (0x13, 0x0) => ("addi").into(),
        (0x13, 0x1) if funct7 == 0x00 => ("slli").into(),
        (0x13, 0x2) => ("slti").into(),
        (0x13, 0x3) => ("sltiu").into(),
        (0x13, 0x4) => ("xori").into(),
        (0x13, 0x5) if funct7 == 0x00 => ("srli").into(),
        (0x13, 0x5) if funct7 == 0x20 => ("srai").into(),
        (0x13, 0x6) => ("ori").into(),
        (0x13, 0x7) => ("andi").into(),

        (0x17, _) => ("auipc").into(),

        (0x1b, 0x0) => ("addiw").into(),
        (0x1b, 0x1) if funct7 == 0x00 => ("slliw").into(),
        (0x1b, 0x5) if funct7 == 0x00 => ("srliw").into(),
        (0x1b, 0x5) if funct7 == 0x20 => ("sraiw").into(),

        // Store Instructions
        (0x23, 0x0) => ("sb").into(),
        (0x23, 0x1) => ("sh").into(),
        (0x23, 0x2) => ("sw").into(),
        (0x23, 0x3) => ("sd").into(),

        (0x33, 0x0) if funct7 == 0x00 => ("add").into(),
        (0x33, 0x0) if funct7 == 0x20 => ("sub").into(),
        (0x33, 0x1) => ("sll").into(),
        (0x33, 0x2) => ("slt").into(),
        (0x33, 0x3) => ("sltu").into(),
        (0x33, 0x4) => ("xor").into(),
        (0x33, 0x5) if funct7 == 0x00 => ("srl").into(),
        (0x33, 0x5) if funct7 == 0x20 => ("sra").into(),
        (0x33, 0x6) => ("or").into(),
        (0x33, 0x7) => ("and").into(),

        (0x37, _) => ("lui").into(),

        (0x3b, 0x0) if funct7 == 0x00 => ("addw").into(),
        (0x3b, 0x0) if funct7 == 0x20 => ("subw").into(),
        (0x3b, 0x1) if funct7 == 0x00 => ("sllw").into(),
        (0x3b, 0x5) if funct7 == 0x00 => ("srlw").into(),
        (0x3b, 0x5) if funct7 == 0x20 => ("sraw").into(),

        (0x63, 0x0) => ("beq").into(),
        (0x63, 0x1) => ("bne").into(),
        (0x63, 0x4) => ("blt").into(),
        (0x63, 0x5) => ("bge").into(),
        (0x63, 0x6) => ("bltu").into(),
        (0x63, 0x7) => ("bgeu").into(),

        (0x67, 0x0) => ("jalr").into(),

        (0x6f, _) => ("jal").into(),

        (0x73, 0x0) if funct7 == 0x0 => ("ecall").into(),
        (0x73, 0x0) if funct7 == 0x1 => ("ebreak").into(),
        (0x73, 0x0) if funct12 == 0x302 => ("wfi").into(),
        (0x73, 0x0) if funct12 == 0x105 => ("mret").into(),

        (0x73, 0x1) => ("csrrw").into(),
        (0x73, 0x2) => ("csrrs").into(),
        (0x73, 0x3) => ("csrrc").into(),
        (0x73, 0x5) => ("csrrwi").into(),
        (0x73, 0x6) => ("csrrsi").into(),
        (0x73, 0x7) => ("csrrci").into(),

        _ => {
            if true {
                let info = UnknownInstr {
                    word: w.0,
                    opcode,
                    rd,
                    funct3,
                    rs1,
                    rs2,
                    funct7,
                    funct12,
                };
                format!("{:#?}", info)
            } else {
                format!("??? opcode=0x{:x} (0x{:08x})", opcode, w.0)
            }
        }
    }
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
    println!("HEX:");
    const WORDS_PER_LINE: usize = 4;

    for (idx, four_words) in code.as_slice().chunks(WORDS_PER_LINE).enumerate() {
        print!("  0x{:>03x}: ", WORDS_PER_LINE * idx);
        for word in four_words {
            print!("0x{:08x} ", word);
        }
        println!();
    }
    println!();

    // Text
    println!("ASM:");
    const INSTR_PER_LINE: usize = 1;

    for (idx, four_words) in code.as_slice().chunks(INSTR_PER_LINE).enumerate() {
        let addr = std::mem::size_of::<u32>() * INSTR_PER_LINE * idx;
        print!("  0x{:>03x}: ", addr);
        for word in four_words {
            let instr = decode_opcode(Word(*word));
            print!("{:<25}", instr);
        }
        println!();
    }
    println!();

    Ok(())
}
