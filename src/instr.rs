use core::convert::TryFrom;

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

#[allow(dead_code)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Instr {
    Illegal,

    Lb {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Lh {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Lw {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Ld {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Lbu {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Lhu {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
    },
    Lwu {
        rd: Reg,
        rs1: Reg,
        imm12: i32,
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
        imm12: i32,
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
        imm12: i32,
    },
    Sh {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: i32,
    },
    Sw {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: i32,
    },
    Sd {
        /// Base address
        rs1: Reg,
        /// Source register
        rs2: Reg,
        /// Offset
        imm12: i32,
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
    Mret {},

    Csrrw {
        rs1: Reg,
        imm12: u32,
    },

    Csrrs {
        rd: Reg,
        rs1: Reg,
        imm12: u32,
    },

    Csrrc {
        rs1: Reg,
    },

    Csrrwi {
        rd: Reg,
    },

    Csrrsi {
        imm5: u8,
        imm12: u32,
    },

    Csrrci {
        imm5: u8,
        imm12: u32,
    },

    Hint {
        /// TODO: Encode hint instructions
        /// Most of them use rd == x0 as a reserved space
        hint: (),
    },
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
