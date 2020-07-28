#![deny(unreachable_patterns)]

use crate::instr::{Instr, Reg};

use std::convert::TryInto;

/// Internal trait to simplify bit operations
trait Bits {
    /// Extract the bits `lo` through `hi`, inclusive, and then shift them to the 0 position.
    fn bits(&self, hi: u8, lo: u8) -> Self;
}

impl Bits for u32 {
    fn bits(&self, hi: u8, lo: u8) -> Self {
        let hi: u32 = hi as u32;
        let mask = u32::MAX >> (31 - hi);

        (self & mask) >> lo
    }
}

#[allow(unused_variables)]
pub fn decode_opcode(w: u32) -> String {
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
        format!("{:?}", instr)
    } else {
        format!("opcode= 0x{:x}, 0b_{:b}", opcode, opcode)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn check_bits() {
        const W: u32 = 0xdead_beef;

        // To help visualize:
        assert_eq!(W, 0b_11011110101011011011111011101111);

        // Sanity check:
        const AWKWARD: u32 = 0b_0110_1101_1111_0111;
        assert_eq!(W & (AWKWARD << 5), AWKWARD << 5);

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
            (31, 0, W),          // Full range
            (20, 5, AWKWARD),    // "Awkward" range that crosses bytes
        ]
        .iter()
        .cloned()
        {
            let actual = W.bits(hi, lo);

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
}
