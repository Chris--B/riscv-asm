#![deny(unreachable_patterns)]

use core::convert::TryFrom;
use std::fmt;

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

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    Illegal,

    Lb {
        rd: Reg,
        rs1: Reg,
        imm: i32,
    },
    Lh {
        rd: Reg,
        rs1: Reg,
        imm: i32,
    },
    Lw {
        rd: Reg,
        rs1: Reg,
        imm: i32,
    },
    Ld {
        rd: Reg,
        rs1: Reg,
        imm: i32,
    },
    Lbu {
        rd: Reg,
        rs1: Reg,
        imm: u32,
    },
    Lhu {
        rd: Reg,
        rs1: Reg,
        imm: u32,
    },
    Lwu {
        rd: Reg,
        rs1: Reg,
        imm: u32,
    },
    Fence {
        rd: Reg,
        rs1: Reg,

        /// Successor Write/Read/Device Output/DeviceInput
        ///
        /// 4-bit value
        successor: u8,

        /// Predecessor Write/Read/Device Output/ƒDeviceInput
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
        imm12: i32,
    },
    Addi {
        rd: Reg,
        rs1: Reg,
        imm: i32,
    },
    Slli {
        rd: Reg,
        rs1: Reg,
        imm5: u8,
    },
    Slti {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Sltiu {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Xori {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
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
        imm12: i32,
    },
    Andi {
        rd: Reg,
        rs1: Reg,
        imm: i32,
    },
    /// AUIPC (add upper immediate to pc) is used to build pc-relative addresses
    /// and uses the U-type format.
    /// AUIPC forms a 32-bit offset from the 20-bit U-immediate, filling in the
    /// lowest 12 bits with zeros, adds this offset to the address of the AUIPC
    /// instruction, then places the result in register rd.
    Auipc {
        rd: Reg,
        imm: u32,
    },
    Sb {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm: i32,
    },
    Sh {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm: i32,
    },
    Sw {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm: i32,
    },
    Sd {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm: i32,
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
        imm: u32,
    },
    Beq {
        rs1: Reg,
        rs2: Reg,
        imm: i32,
    },
    Bne {
        rs1: Reg,
        rs2: Reg,
        imm: i32,
    },
    Blt {
        rs1: Reg,
        rs2: Reg,
        imm: i32,
    },
    Bge {
        rs1: Reg,
        rs2: Reg,
        imm: i32,
    },
    Bltu {
        rs1: Reg,
        rs2: Reg,
        imm: i32,
    },
    Bgeu {
        rs1: Reg,
        rs2: Reg,
        imm: i32,
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
        imm: i32,
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
        imm: i32,
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
    Uret {},
    Sret {},
    Mret {},

    Csrrw {
        rd: Reg,
        rs1: Reg,
        csr: u16,
    },

    Csrrs {
        rd: Reg,
        rs1: Reg,
        csr: u16,
    },

    Csrrc {
        rd: Reg,
        rs1: Reg,
        csr: u16,
    },

    Csrrwi {
        rd: Reg,
        src: u8,
        csr: u16,
    },

    Csrrsi {
        rd: Reg,
        src: u8,
        csr: u16,
    },

    Csrrci {
        rd: Reg,
        src: u8,
        csr: u16,
    },

    Hint {
        /// TODO: Encode hint instructions
        /// Most of them use rd == x0 as a reserved space
        hint: (),
    },
}

/// Instructions have arguments that specify the data that they used when executed.
///
/// Instructions can be:
///     1. A register (`a0`, `zero`, ...)
///     2. A signed or unsigned immediate (`0`, `123`, `0xff`, ...)
///         The signedness typically changes per instruction and is otherwise fixed.
///     3. A pair that forms a base (stored in a register) and offset (as an immediate) (`0(ra)`, `4(sp)`, ...)
///
/// This is used when enumerating over the instructions arguments. offset + base pairs are treated as a single argument
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Arg {
    /// A value read from a register before executing the instruction, or written to one afterwards
    Register(Reg),

    /// An unsigned value that is supplied as a literal in the assembly
    UnsignedImm(u32),

    /// A signed value that is supplied as a literal in the assembly
    SignedImm(i32),

    /// Some (usually `SYSTEM`) instructions take special, named values that correspond to immediates
    ///
    /// It's more useful to represent them as strings than immediates, so there's a dedicated Arg type for them.
    Special(String),

    /// A pair of a register and a signed offset that is added to get an address that acts as a single argument
    Address { base: Reg, offset: i32 },
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Reg::*;

        let reg = match *self {
            Zero => "zero",
            Ra => "ra",
            Sp => "sp",
            Gp => "gp",
            Tp => "tp",
            T0 => "t0",
            T1 => "t1",
            T2 => "t2",
            S0 => "s0",
            S1 => "s1",
            A0 => "a0",
            A1 => "a1",
            A2 => "a2",
            A3 => "a3",
            A4 => "a4",
            A5 => "a5",
            A6 => "a6",
            A7 => "a7",
            S2 => "s2",
            S3 => "s3",
            S4 => "s4",
            S5 => "s5",
            S6 => "s6",
            S7 => "s7",
            S8 => "s8",
            S9 => "s9",
            S10 => "s10",
            S11 => "s11",
            T3 => "t3",
            T4 => "t4",
            T5 => "t5",
            T6 => "t6",
        };

        write!(f, "{}", reg)
    }
}

impl fmt::Display for Arg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Arg::*;

        match self {
            Register(reg) => write!(f, "{}", reg),
            UnsignedImm(imm) => write!(f, "{}", imm),
            SignedImm(imm) => write!(f, "{}", imm),
            Special(special) => write!(f, "{}", special),
            Address { base, offset } => write!(f, "{offset}({base})", base = base, offset = offset),
        }
    }
}

impl From<Reg> for Arg {
    fn from(reg: Reg) -> Arg {
        Arg::Register(reg)
    }
}

impl From<i32> for Arg {
    fn from(value: i32) -> Arg {
        Arg::SignedImm(value)
    }
}

impl From<u32> for Arg {
    fn from(value: u32) -> Arg {
        Arg::UnsignedImm(value)
    }
}

impl Instr {
    /// The all-lowercase neumonic for this instruction
    pub fn name(&self) -> String {
        use Instr::*;

        match *self {
            Illegal => "illegal",
            Hint { .. } => "hint",

            Add { .. } => "add",
            Addi { .. } => "addi",
            And { .. } => "and",
            Andi { .. } => "andi",
            Auipc { .. } => "auipc",
            Beq { .. } => "beq",
            Bge { .. } => "bge",
            Bgeu { .. } => "bgeu",
            Blt { .. } => "blt",
            Bltu { .. } => "bltu",
            Bne { .. } => "bne",
            Csrrc { .. } => "csrrc",
            Csrrci { .. } => "csrrci",
            Csrrs { .. } => "csrrs",
            Csrrsi { .. } => "csrrsi",
            Csrrw { .. } => "csrrw",
            Csrrwi { .. } => "csrrwi",
            Ebreak { .. } => "ebreak",
            Ecall { .. } => "ecall",
            Fence { .. } => "fence",
            FenceI { .. } => "fencei",
            Jal { .. } => "jal",
            Jalr { .. } => "jalr",
            Lb { .. } => "lb",
            Lbu { .. } => "lbu",
            Ld { .. } => "ld",
            Lh { .. } => "lh",
            Lhu { .. } => "lhu",
            Lui { .. } => "lui",
            Lw { .. } => "lw",
            Lwu { .. } => "lwu",
            Mret { .. } => "mret",
            Or { .. } => "or",
            Ori { .. } => "ori",
            Sb { .. } => "sb",
            Sd { .. } => "sd",
            Sh { .. } => "sh",
            Sll { .. } => "sll",
            Slli { .. } => "slli",
            Slt { .. } => "slt",
            Slti { .. } => "slti",
            Sltiu { .. } => "sltiu",
            Sltu { .. } => "sltu",
            Sra { .. } => "sra",
            Srai { .. } => "srai",
            Sret { .. } => "sret",
            Srl { .. } => "srl",
            Srli { .. } => "srli",
            Sub { .. } => "sub",
            Sw { .. } => "sw",
            Uret { .. } => "uret",
            Wfi { .. } => "wfi",
            Xor { .. } => "xor",
            Xori { .. } => "xori",
        }
        .into()
    }

    /// Values provided to an instruction that change its behavior
    ///
    /// Note: The number of arguments may change depending on the value of some arguments (e.g. omitting a trivial argument)
    // TODO: Clarify when this happens.
    pub fn args(&self) -> Vec<Arg> {
        use Arg::*;
        use Instr::*;
        use Reg::*;

        match *self {
            Illegal => vec![],
            Hint { .. } => vec![],

            Add { rd, rs1, rs2 } => vec![rd.into(), rs1.into(), rs2.into()],
            Addi { rd, rs1, imm } => vec![rd.into(), rs1.into(), imm.into()],

            And { rd, rs1, rs2 } => vec![rd.into(), rs1.into(), rs2.into()],
            Andi { rd, rs1, imm } => vec![rd.into(), rs1.into(), imm.into()],

            Auipc { rd, imm } => vec![rd.into(), imm.into()],

            Beq { rs1, rs2, imm } => vec![rs1.into(), rs2.into(), imm.into()],
            Bge { rs1, rs2, imm } => vec![rs1.into(), rs2.into(), imm.into()],
            Bgeu { rs1, rs2, imm } => vec![rs1.into(), rs2.into(), imm.into()],
            Blt { rs1, rs2, imm } => vec![rs1.into(), rs2.into(), imm.into()],
            Bltu { rs1, rs2, imm } => vec![rs1.into(), rs2.into(), imm.into()],
            Bne { rs1, rs2, imm } => vec![rs1.into(), rs2.into(), imm.into()],

            Csrrc { .. } => vec![],
            Csrrci { .. } => vec![],
            Csrrs { .. } => vec![],
            Csrrsi { .. } => vec![],
            Csrrw { .. } => vec![],
            Csrrwi { .. } => vec![],
            Ebreak { .. } => vec![],
            Ecall { .. } => vec![],

            Fence {
                rd,
                rs1,
                successor,
                predecessor,
                fm,
            } => vec![
                Register(rd),
                Register(rs1),
                Special(format!(
                    "succ: 0b{:b}, pred: 0b{:b}, fm: 0b{:b}",
                    successor, predecessor, fm
                )),
            ],
            FenceI { rd, rs1, imm12 } => vec![rd.into(), rs1.into(), imm12.into()],

            Jal { rd, imm } => vec![rd.into(), imm.into()],
            Jalr { rd: Ra, rs1, imm } => vec![Address {
                base: rs1,
                offset: imm,
            }],
            Jalr { rd, rs1, imm } => vec![
                rd.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],

            Lb { rd, rs1, imm } => vec![
                rd.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],
            Ld { rd, rs1, imm } => vec![
                rd.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],
            Lh { rd, rs1, imm } => vec![
                rd.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],
            Lw { rd, rs1, imm } => vec![
                rd.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],

            Lbu { rd, rs1: _, imm: _ } => vec![rd.into()],
            Lhu { rd, rs1: _, imm: _ } => vec![rd.into()],
            Lwu { rd, rs1: _, imm: _ } => vec![rd.into()],

            Lui { rd, imm } => vec![rd.into(), imm.into()],

            Mret { .. } => vec![],

            Or { rd, rs1, rs2 } => vec![Register(rd), Register(rs1), Register(rs2)],
            Ori { rd, rs1, imm12 } => vec![Register(rd), Register(rs1), SignedImm(imm12)],

            Sb { rs1, rs2, imm } => vec![
                rs2.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],
            Sd { rs1, rs2, imm } => vec![
                rs2.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],
            Sh { rs1, rs2, imm } => vec![
                rs2.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],
            Sw { rs1, rs2, imm } => vec![
                rs2.into(),
                Address {
                    base: rs1,
                    offset: imm,
                },
            ],

            Sll { .. } => vec![],
            Slli { .. } => vec![],
            Slt { .. } => vec![],
            Slti { .. } => vec![],
            Sltiu { .. } => vec![],
            Sltu { .. } => vec![],
            Sra { .. } => vec![],
            Srai { .. } => vec![],
            Sret { .. } => vec![],
            Srl { .. } => vec![],
            Srli { .. } => vec![],

            Sub { rd, rs1, rs2 } => vec![Register(rd), Register(rs1), Register(rs2)],

            Uret { .. } => vec![],
            Wfi { .. } => vec![],

            Xor { rd, rs1, rs2 } => vec![rd.into(), rs1.into(), rs2.into()],
            Xori { rd, rs1, imm12 } => vec![rd.into(), rs1.into(), imm12.into()],
        }
    }
}

/// An error when a register is referenced out of bounds
///
/// A `Reg` value can be constructed from a `u8` or a `u32`, whichever is more
/// convenient to use. Notice though, that all valid `Reg` values are derived
/// from valid `u8`.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct RegIndexError {
    idx: u32,
}

impl TryFrom<u32> for Reg {
    type Error = RegIndexError;
    fn try_from(idx: u32) -> Result<Reg, Self::Error> {
        match idx {
            0 => Ok(Reg::Zero),
            1 => Ok(Reg::Ra),
            2 => Ok(Reg::Sp),
            3 => Ok(Reg::Gp),
            4 => Ok(Reg::Tp),
            5 => Ok(Reg::T0),
            6 => Ok(Reg::T1),
            7 => Ok(Reg::T2),
            8 => Ok(Reg::S0),
            9 => Ok(Reg::S1),
            10 => Ok(Reg::A0),
            11 => Ok(Reg::A1),
            12 => Ok(Reg::A2),
            13 => Ok(Reg::A3),
            14 => Ok(Reg::A4),
            15 => Ok(Reg::A5),
            16 => Ok(Reg::A6),
            17 => Ok(Reg::A7),
            18 => Ok(Reg::S2),
            19 => Ok(Reg::S3),
            20 => Ok(Reg::S4),
            21 => Ok(Reg::S5),
            22 => Ok(Reg::S6),
            23 => Ok(Reg::S7),
            24 => Ok(Reg::S8),
            25 => Ok(Reg::S9),
            26 => Ok(Reg::S10),
            27 => Ok(Reg::S11),
            28 => Ok(Reg::T3),
            29 => Ok(Reg::T4),
            30 => Ok(Reg::T5),
            31 => Ok(Reg::T6),
            _ => Err(RegIndexError { idx }),
        }
    }
}

impl TryFrom<u8> for Reg {
    type Error = RegIndexError;
    fn try_from(idx: u8) -> Result<Reg, Self::Error> {
        Reg::try_from(idx as u32)
    }
}
