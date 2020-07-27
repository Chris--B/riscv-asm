#![allow(clippy::inconsistent_digit_grouping)]
#![deny(unreachable_patterns)]

use std::convert::TryFrom;
use std::convert::TryInto;
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

/// Register mnemonics for the standard ABI
///
/// See: https://github.com/riscv/riscv-elf-psabi-doc/blob/master/riscv-elf.md#integer-register-convention-
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Reg {
    Zero = 0,
    Ra = 1,
    Sp = 2,
    Gp = 3,
    Tp = 4,
    T0 = 5,
    T1 = 6,
    T2 = 7,
    S0 = 8,
    S1 = 9,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
    A6 = 16,
    A7 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    S8 = 24,
    S9 = 25,
    S10 = 26,
    S11 = 27,
    T3 = 28,
    T4 = 29,
    T5 = 30,
    T6 = 31,
}

impl Default for Reg {
    fn default() -> Reg {
        Reg::Zero
    }
}

/// An error when a register is referenced out of bounds
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RegIndexError {
    idx: u32,
}

impl TryFrom<u32> for Reg {
    type Error = RegIndexError;
    fn try_from(idx: u32) -> Result<Reg, Self::Error> {
        let o_reg = match idx {
            0 => Some(Reg::Zero),
            1 => Some(Reg::Ra),
            2 => Some(Reg::Sp),
            3 => Some(Reg::Gp),
            4 => Some(Reg::Tp),
            5 => Some(Reg::T0),
            6 => Some(Reg::T1),
            7 => Some(Reg::T2),
            8 => Some(Reg::S0),
            9 => Some(Reg::S1),
            10 => Some(Reg::A0),
            11 => Some(Reg::A1),
            12 => Some(Reg::A2),
            13 => Some(Reg::A3),
            14 => Some(Reg::A4),
            15 => Some(Reg::A5),
            16 => Some(Reg::A6),
            17 => Some(Reg::A7),
            18 => Some(Reg::S2),
            19 => Some(Reg::S3),
            20 => Some(Reg::S4),
            21 => Some(Reg::S5),
            22 => Some(Reg::S6),
            23 => Some(Reg::S7),
            24 => Some(Reg::S8),
            25 => Some(Reg::S9),
            26 => Some(Reg::S10),
            27 => Some(Reg::S11),
            28 => Some(Reg::T3),
            29 => Some(Reg::T4),
            30 => Some(Reg::T5),
            31 => Some(Reg::T6),
            _ => None,
        };

        if let Some(reg) = o_reg {
            Ok(reg)
        } else {
            Err(RegIndexError { idx })
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
enum Instr {
    Lb {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Lh {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Lw {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Ld {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Lbu {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Lhu {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Lwu {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Fence {
        rd: Reg,
        rs1: Reg,

        /// Successor Write/Read/Device Output/DeviceInput
        ///
        /// 4-bit value
        successor: u8,

        /// Predecessor Write/Read/Device Output/DeviceInput
        ///
        /// 4-bit value
        predecessor: u8,

        /// Fence Mode
        ///
        /// 4-bit value
        fm: u8,
    },
    FenceI {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Addi {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Slli {
        rd: Reg,
        rs1: Reg,
        imm5: u8,
    },
    Slti {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Sltiu {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Xori {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Srli {
        rd: Reg,
        rs1: Reg,
        imm5: u8,
    },
    Srai {
        rd: Reg,
        rs1: Reg,
        imm5: u8,
    },
    Ori {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Andi {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },
    Auipc {
        rd: Reg,
        imm20: u32,
    },
    Sb {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: u32,
    },
    Sh {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: u32,
    },
    Sw {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: u32,
    },
    Sd {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: u32,
    },
    Add {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Sub {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Sll {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Slt {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Sltu {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Xor {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Srl {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Sra {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Or {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    And {
        rd: Reg,
        rs1: Reg,
        rs2: Reg,
    },
    Lui {
        rd: Reg,
        imm20: u32,
    },
    Beq {
        rs1: Reg,
        rs2: Reg,
        imm12: u32,
    },
    Bne {
        rs1: Reg,
        rs2: Reg,
        imm12: u32,
    },
    Blt {
        rs1: Reg,
        rs2: Reg,
        imm12: u32,
    },
    Bge {
        rs1: Reg,
        rs2: Reg,
        imm12: u32,
    },
    Bltu {
        rs1: Reg,
        rs2: Reg,
        imm12: u32,
    },
    Bgeu {
        rs1: Reg,
        rs2: Reg,
        imm12: u32,
    },

    /// Jumps to a target address and saves the return address
    ///
    /// The target address is obtained by adding the sign-extended `imm12`
    /// to the register `rs1` then setting the LSB to 0.
    /// The instruction following the jump (pc + 4) is written to register `rd`
    /// The standard software calling convention uses `x1` as the return address
    /// register and `x5` as an alternative link register.
    Jalr {
        rd: Reg,
        rs1: Reg,
        /// Encoded as a multiple of 2-bytes
        imm12: u32,
    },

    /// Jumps to a relative address
    ///
    /// `imm20` is sign-extended and added to the address of the jump
    /// instruction to form the jump target address.
    /// The standard software calling convention uses `x1` as the return address
    /// register and `x5` as an alternative link register.
    Jal {
        rd: Reg,
        /// Encoded as a multiple of 2-bytes
        imm20: u32,
    },

    /// Make a service request to the execution environment
    ///
    /// The EEI will define how parameters for the service request are passed,
    /// but usually these will be in define locations in the integer register
    /// file.
    Ecall {
        rd: Reg,
        rs1: Reg,
    },

    /// Return control to a debugging environment
    Ebreak {
        rd: Reg,
        rs1: Reg,
    },

    // TODO: System Instructions
    Wfi {},
    Mret {},
    Csrrw {},
    Csrrs {},
    Csrrc {},
    Csrrwi {},
    Csrrsi {},
    Csrrci {},

    Hint {
        /// TODO: Encode hint instructions
        /// Most of them use rd == x0 as a reserved space
        hint: (),
    },
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
    // larger match block below.
    let opcode = w.bits(6, 0);
    let rd: Reg = w.bits(11, 7).try_into().unwrap_or_default();
    let funct3 = w.bits(14, 12);
    let rs1: Reg = w.bits(19, 15).try_into().unwrap_or_default();
    let rs2: Reg = w.bits(24, 20).try_into().unwrap_or_default();
    let funct7 = w.bits(31, 25);
    let funct12 = w.bits(31, 20);

    // TODO: Not sure if these are always loaded the same way
    let imm5 = 0;
    let imm12 = 0;
    let imm20 = 0;

    let o_instr: Option<Instr> = match (opcode, funct3) {
        // Load Instructions
        (0x03, 0x0) => Some(Instr::Lb { rd, rs1, imm12 }),
        (0x03, 0x1) => Some(Instr::Lh { rd, rs1, imm12 }),
        (0x03, 0x2) => Some(Instr::Lw { rd, rs1, imm12 }),
        (0x03, 0x3) => Some(Instr::Ld { rd, rs1, imm12 }),
        (0x03, 0x4) => Some(Instr::Lbu { rd, rs1, imm12 }),
        (0x03, 0x5) => Some(Instr::Lhu { rd, rs1, imm12 }),
        (0x03, 0x6) => Some(Instr::Lwu { rd, rs1, imm12 }),

        // Fences
        (0x0f, 0x0) => Some(Instr::Fence {
            rd,
            rs1,
            successor: 0,
            predecessor: 0,
            fm: 0,
        }),
        (0x0f, 0x1) => Some(Instr::FenceI { rd, rs1, imm12 }),

        (0x13, 0x0) => Some(Instr::Addi { rd, rs1, imm12 }),
        (0x13, 0x1) if funct7 == 0x00 => Some(Instr::Slli { rd, rs1, imm5 }),
        (0x13, 0x2) => Some(Instr::Slti { rd, rs1, imm12 }),
        (0x13, 0x3) => Some(Instr::Sltiu { rd, rs1, imm12 }),
        (0x13, 0x4) => Some(Instr::Xori { rd, rs1, imm12 }),
        (0x13, 0x5) if funct7 == 0x00 => Some(Instr::Srli { rd, rs1, imm5 }),
        (0x13, 0x5) if funct7 == 0x20 => Some(Instr::Srai { rd, rs1, imm5 }),
        (0x13, 0x6) => Some(Instr::Ori { rd, rs1, imm12 }),
        (0x13, 0x7) => Some(Instr::Andi { rd, rs1, imm12 }),

        (0x17, _) => Some(Instr::Auipc { rd, imm20 }),

        // // Store Instructions
        (0x23, 0x0) => Some(Instr::Sb { rs1, rs2, imm12 }),
        (0x23, 0x1) => Some(Instr::Sh { rs1, rs2, imm12 }),
        (0x23, 0x2) => Some(Instr::Sw { rs1, rs2, imm12 }),
        (0x23, 0x3) => Some(Instr::Sd { rs1, rs2, imm12 }),

        (0x33, 0x0) if funct7 == 0x00 => Some(Instr::Add { rd, rs1, rs2 }),
        (0x33, 0x0) if funct7 == 0x20 => Some(Instr::Sub { rd, rs1, rs2 }),
        (0x33, 0x1) => Some(Instr::Sll { rd, rs1, rs2 }),
        (0x33, 0x2) => Some(Instr::Slt { rd, rs1, rs2 }),
        (0x33, 0x3) => Some(Instr::Sltu { rd, rs1, rs2 }),
        (0x33, 0x4) => Some(Instr::Xor { rd, rs1, rs2 }),
        (0x33, 0x5) if funct7 == 0x00 => Some(Instr::Srl { rd, rs1, rs2 }),
        (0x33, 0x5) if funct7 == 0x20 => Some(Instr::Sra { rd, rs1, rs2 }),
        (0x33, 0x6) => Some(Instr::Or { rd, rs1, rs2 }),
        (0x33, 0x7) => Some(Instr::And { rd, rs1, rs2 }),

        (0x37, _) => Some(Instr::Lui { rd, imm20 }),

        (0x63, 0x0) => Some(Instr::Beq { rs1, rs2, imm12 }),
        (0x63, 0x1) => Some(Instr::Bne { rs1, rs2, imm12 }),
        (0x63, 0x4) => Some(Instr::Blt { rs1, rs2, imm12 }),
        (0x63, 0x5) => Some(Instr::Bge { rs1, rs2, imm12 }),
        (0x63, 0x6) => Some(Instr::Bltu { rs1, rs2, imm12 }),
        (0x63, 0x7) => Some(Instr::Bgeu { rs1, rs2, imm12 }),

        (0x67, 0x0) => Some(Instr::Jalr { rd, rs1, imm12 }),

        (0x6f, _) => Some(Instr::Jal { rd, imm20 }),

        (0x73, 0x0) if funct7 == 0x0 => Some(Instr::Ecall { rd, rs1 }),
        (0x73, 0x0) if funct7 == 0x1 => Some(Instr::Ebreak { rd, rs1 }),
        (0x73, 0x0) if funct12 == 0x302 => Some(Instr::Wfi {}),
        (0x73, 0x0) if funct12 == 0x105 => Some(Instr::Mret {}),

        (0x73, 0x1) => Some(Instr::Csrrw {}),
        (0x73, 0x2) => Some(Instr::Csrrs {}),
        (0x73, 0x3) => Some(Instr::Csrrc {}),
        (0x73, 0x5) => Some(Instr::Csrrwi {}),
        (0x73, 0x6) => Some(Instr::Csrrsi {}),
        (0x73, 0x7) => Some(Instr::Csrrci {}),
        _ => None,
    };

    if let Some(instr) = o_instr {
        format!("{:?}", o_instr)
    } else {
        format!("opcode= 0x{:x}, 0b_{:b}", opcode, opcode)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let opts = MyOptions::new();
    dbg!(&opts);
    dbg!(std::mem::size_of::<Instr>());

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
