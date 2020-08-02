#![deny(unreachable_patterns)]

use crate::instr::{Instr, Reg};
use Instr::*;

use std::convert::TryInto;

/// Internal trait to simplify bit operations
trait Bits {
    type Signed;

    /// Extract the bit at index `idx`
    fn bit(&self, idx: u8) -> Self;

    /// Extract the bits `lo` through `hi`, inclusive, and then shift them to the 0 position.
    fn bits(&self, hi: u8, lo: u8) -> Self;

    /// Sign extend using the bit at index `hi` as the most significant bit.
    ///
    /// All bits in locations >= `hi` are replaced with the bit at `hi`
    fn sign_ext(&self, hi: u8) -> Self::Signed;
}

impl Bits for u32 {
    type Signed = i32;

    fn bit(&self, idx: u8) -> Self {
        let idx = idx as u32;
        (self >> idx) & 0x1
    }

    fn bits(&self, hi: u8, lo: u8) -> Self {
        let hi: u32 = hi as u32;
        let mask = u32::MAX >> (31 - hi);

        (self & mask) >> lo
    }

    fn sign_ext(&self, hi: u8) -> Self::Signed {
        let mask = if self.bit(hi) == 0 {
            0
        } else {
            u32::MAX << (hi + 1)
        };
        (self | mask) as Self::Signed
    }
}

#[allow(unused_variables)]
pub fn decode_opcode(w: u32) -> Option<Instr> {
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
    let funct3 = w.bits(14, 12);
    let rd_idx = w.bits(11, 7) as u8;
    let rs2_idx = w.bits(24, 20) as u8;
    let rs1_idx = w.bits(19, 15) as u8;
    let funct7 = w.bits(31, 25);
    let funct12 = w.bits(31, 20);

    // csr is a lot like funct12/I-type immediates, but it is zero-extended
    let csr = funct12 as u16;

    let rd: Reg = rd_idx.try_into().unwrap_or(Reg::Zero);
    let rs2: Reg = rs2_idx.try_into().unwrap_or(Reg::Zero);
    let rs1: Reg = rs1_idx.try_into().unwrap_or(Reg::Zero);

    // TODO: Not sure if these are always loaded the same way
    let imm5: u8 = 0;
    let imm12: i32 = 0;
    let imm20: i32 = 0;

    // R-type instructions do not have an immediate encoded
    let _r_imm: ();

    let i_imm: i32 = w.bits(31, 20).sign_ext(11);

    let s_imm: i32 = ((w.bits(31, 25) << 5) | w.bits(11, 7)).sign_ext(11);

    #[rustfmt::skip]
    let b_imm: i32 = (
        (w.bit(31) << 12)                   // ┌ Note: lsb is always 0!
        | (w.bit(7) << 11)                  // │
        | (w.bits(30, 25) << 5)             // │
        | (w.bits(11, 8) << 1)              // ┘
    )
    .sign_ext(12);

    // U-type encodings may shift this by 12, or may keep it as-is but
    // reinterpret the bits as the upper 20-bits of a word.
    let u_imm: u32 = w.bits(31, 12);

    #[rustfmt::skip]
    let j_imm: i32 = (
        (w.bit(31) << 20)                   // ┌ Note: lsb is always 0!
        | (w.bits(19, 12) << 12)            // │
        | (w.bit(20) << 11)                 // │
        | (w.bits(30, 21) << 1)             // ┘
    )
    .sign_ext(20);

    // Print some useful state so that it's visible when a unit test fails.
    if cfg!(test) {
        println!("=== DECODE STATE ===");
        println!("word    0x{bits:08x} 0b{bits:032b} {:>12}", bits = w);
        println!("opcode  0x{bits:08x} 0b{bits:032b} {:>12}", bits = opcode);
        println!("funct3  0x{bits:08x} 0b{bits:032b} {:>12}", bits = funct3);
        println!("funct7  0x{bits:08x} 0b{bits:032b} {:>12}", bits = funct7);
        println!("funct12 0x{bits:08x} 0b{bits:032b} {:>12}", bits = funct12);
        println!();

        println!("i_imm   0x{bits:08x} 0b{bits:032b} {:>12}", bits = i_imm);
        println!("s_imm   0x{bits:08x} 0b{bits:032b} {:>12}", bits = s_imm);
        println!("b_imm   0x{bits:08x} 0b{bits:032b} {:>12}", bits = b_imm);
        println!("u_imm   0x{bits:08x} 0b{bits:032b} {:>12}", bits = u_imm);
        println!("j_imm   0x{bits:08x} 0b{bits:032b} {:>12}", bits = j_imm);

        println!();
        println!("rd      {:?}", rd);
        println!("rd_idx  0x{bits:08x} 0b{bits:032b} {:>5}", bits = rd_idx);
        println!("rs2     {:?}", rs2);
        println!("rs2_idx 0x{bits:08x} 0b{bits:032b} {:>5}", bits = rs2_idx);
        println!("rs1     {:?}", rs1);
        println!("rs1_idx 0x{bits:08x} 0b{bits:032b} {:>5}", bits = rs1_idx);
        println!();
    }

    match (opcode, funct3) {
        // Special values
        _ if w == 0x0 => {
            // The all-zero instruction is special-cased as illegal, so we handle
            // it here like an instruction. For the rest of our decoding, we'll handle
            // invalid instructions like an error.
            Some(Illegal)
        }

        // Load Instructions
        (0x03, 0x0) => Some(Lb {
            rd,
            rs1,
            imm: i_imm,
        }),
        (0x03, 0x1) => Some(Lh {
            rd,
            rs1,
            imm: i_imm,
        }),
        (0x03, 0x2) => Some(Lw {
            rd,
            rs1,
            imm: i_imm,
        }),
        (0x03, 0x3) => Some(Ld {
            rd,
            rs1,
            imm: i_imm,
        }),
        (0x03, 0x4) => Some(Lbu {
            rd,
            rs1,
            imm: i_imm as u32,
        }),
        (0x03, 0x5) => Some(Lhu {
            rd,
            rs1,
            imm: i_imm as u32,
        }),
        (0x03, 0x6) => Some(Lwu {
            rd,
            rs1,
            imm: i_imm as u32,
        }),

        // Fences
        (0x0f, 0x0) => Some(Fence {
            rd,
            rs1,
            successor: w.bits(27, 24) as u8,
            predecessor: w.bits(23, 20) as u8,
            fm: w.bits(31, 28) as u8,
        }),
        (0x0f, 0x1) => Some(FenceI { rd, rs1, imm12 }),

        (0x13, 0x0) => Some(Addi {
            rd,
            rs1,
            imm: i_imm,
        }),
        (0x13, 0x1) if funct7 == 0x00 => Some(Slli {
            rd,
            rs1,
            imm5: i_imm as u8,
        }),
        (0x13, 0x2) => Some(Slti { rd, rs1, imm12 }),
        (0x13, 0x3) => Some(Sltiu { rd, rs1, imm12 }),
        (0x13, 0x4) => Some(Xori { rd, rs1, imm12 }),
        (0x13, 0x5) if funct7 == 0x00 => Some(Srli { rd, rs1, imm5 }),
        (0x13, 0x5) if funct7 == 0x20 => Some(Srai { rd, rs1, imm5 }),
        (0x13, 0x6) => Some(Ori { rd, rs1, imm12 }),
        (0x13, 0x7) => Some(Andi {
            rd,
            rs1,
            imm: i_imm,
        }),

        (0x17, _) => Some(Auipc { rd, imm: u_imm }),

        // // Store Instructions
        (0x23, 0x0) => Some(Sb {
            rs1,
            rs2,
            imm: s_imm,
        }),
        (0x23, 0x1) => Some(Sh {
            rs1,
            rs2,
            imm: s_imm,
        }),
        (0x23, 0x2) => Some(Sw {
            rs1,
            rs2,
            imm: s_imm,
        }),
        (0x23, 0x3) => Some(Sd {
            rs1,
            rs2,
            imm: s_imm,
        }),

        (0x33, 0x0) if funct7 == 0x00 => Some(Add { rd, rs1, rs2 }),
        (0x33, 0x0) if funct7 == 0x20 => Some(Sub { rd, rs1, rs2 }),
        (0x33, 0x1) => Some(Sll { rd, rs1, rs2 }),
        (0x33, 0x2) => Some(Slt { rd, rs1, rs2 }),
        (0x33, 0x3) => Some(Sltu { rd, rs1, rs2 }),
        (0x33, 0x4) => Some(Xor { rd, rs1, rs2 }),
        (0x33, 0x5) if funct7 == 0x00 => Some(Srl { rd, rs1, rs2 }),
        (0x33, 0x5) if funct7 == 0x20 => Some(Sra { rd, rs1, rs2 }),
        (0x33, 0x6) => Some(Or { rd, rs1, rs2 }),
        (0x33, 0x7) => Some(And { rd, rs1, rs2 }),

        (0x37, _) => Some(Lui { rd, imm: u_imm }),

        (0x63, 0x0) => Some(Beq {
            rs1,
            rs2,
            imm: b_imm,
        }),
        (0x63, 0x1) => Some(Bne {
            rs1,
            rs2,
            imm: b_imm,
        }),
        (0x63, 0x4) => Some(Blt {
            rs1,
            rs2,
            imm: b_imm,
        }),
        (0x63, 0x5) => Some(Bge {
            rs1,
            rs2,
            imm: b_imm,
        }),
        (0x63, 0x6) => Some(Bltu {
            rs1,
            rs2,
            imm: b_imm,
        }),
        (0x63, 0x7) => Some(Bgeu {
            rs1,
            rs2,
            imm: b_imm,
        }),

        // Note: Jal uses J-type encoding, but Jalr uses I-type encoding
        (0x67, 0x0) => Some(Jalr {
            rd,
            rs1,
            imm: i_imm,
        }),
        (0x6f, _) => Some(Jal { rd, imm: j_imm }),

        (0x73, 0x0) if funct7 == 0x0 => Some(Ecall { rd, rs1 }),
        (0x73, 0x0) if funct7 == 0x1 => Some(Ebreak { rd, rs1 }),
        (0x73, 0x0) if funct12 == 0x302 => Some(Wfi {}),
        (0x73, 0x0) if funct12 == 0x105 => Some(Mret {}),

        (0x73, 0x1) => Some(Csrrw {
            rs1,
            imm12: imm12 as u32,
        }),
        (0x73, 0x2) => Some(Csrrs {
            rd,
            rs1,
            imm12: imm12 as u32,
        }),
        (0x73, 0x3) => Some(Csrrc { rs1 }),
        (0x73, 0x5) => Some(Csrrwi { rd }),
        (0x73, 0x6) => Some(Csrrsi {
            imm5,
            imm12: imm12 as u32,
        }),
        (0x73, 0x7) => Some(Csrrci {
            imm5,
            imm12: imm12 as u32,
        }),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use Reg::*;

    // These asserts give us diffs when they fail.
    // Import both `assert_eq` and `assert_ne`, even though we don't `assert_ne`
    // yet, so that future tests don't accidentally miss this import and use
    // std's macros instead.
    #[allow(unused_imports)]
    use pretty_assertions::{assert_eq, assert_ne};

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

            if hi == lo {
                assert_eq!(actual, W.bit(hi));
            }
        }
    }

    #[test]
    fn check_bits_sign_ext() {
        assert_eq!(
            0b_0000_0000_0000_0000_0000_0000_0000_0001.sign_ext(0),
            //                                       ^~~~~~~~~~~^
            0b_1111_1111_1111_1111_1111_1111_1111_1111_u32 as i32, // Expected
        );
        assert_eq!(
            0b_0000_0000_0000_0000_0100_0000_0000_1111_u32.sign_ext(14),
            //                      ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^
            0b_1111_1111_1111_1111_1100_0000_0000_1111_u32 as i32, // Expected
        );

        assert_eq!(
            0b_0.sign_ext(0),
            // ^~~~~~~~~~~^
            0b_0_u32 as i32, // Expected
        );
        assert_eq!(
            0b_0100_0000_0000_1111_u32.sign_ext(15),
            // ^~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~^^
            0b_0100_0000_0000_1111_u32 as i32, // Expected
        );
    }

    // This test takes takes too long, while the rest are instant.
    /// (~15 seconds on release and ~1238 seconds (yes really) on debug)
    // Until this is sped up, mark it as ignored. We can run it manually:
    //      time cargo test --release brute -- --ignored
    #[test]
    #[ignore]
    fn brute_force_decode() {
        // Test every word to make sure that we don't panic :)
        for word in 0..=u32::MAX {
            let _ = decode_opcode(word);
        }
    }

    macro_rules! make_instr_test {
        ( $( $test_name:ident : $le_bytes:expr => $expected:expr ),+ ) => {
            $(
                #[test]
                fn $test_name() {
                    let word = u32::from_le_bytes($le_bytes);
                    assert_eq!(decode_opcode(word), Some($expected));
                }
            )+
        };
    }

    make_instr_test! {
        // The zero-word is an illegal instruction by design.
        check_zero_word:                [0x00, 0x00, 0x00, 0x00] => Illegal,
        check_unimp:                    [0x73, 0x10, 0x00, 0xc0] => Illegal,

        // TODO: Check
        //      add a, b, c
        // making sure to use each of the 31 registers at least twice in different spots.
        // Note: add zero, X, X is a "HINT" opcode

        check_add_s0_sp_zero:           [0x33, 0x04, 0x01, 0x00] => Add { rd: S0, rs1: Sp, rs2: Zero, },
        check_add_a2_a5_a1:             [0x33, 0x86, 0xb7, 0x00] => Add { rd: A2, rs1: A5, rs2: A1, },
        check_add_t0_t0_t2:             [0xb3, 0x82, 0x72, 0x00] => Add { rd: T0, rs1: T0, rs2: T2, },

        check_addi_sp_sp_64:            [0x13, 0x01, 0x01, 0x04] => Addi { rd: Sp, rs1: Sp, imm: 64, },
        check_addi_t1_t1_neg_1:         [0x13, 0x03, 0xf3, 0xff] => Addi { rd: T1, rs1: T1, imm: -1, },
        check_addi_a0_sp_32:            [0x13, 0x05, 0x01, 0x02] => Addi { rd: A0, rs1: Sp, imm: 32, },
        check_addi_a7_a0_neg_273:       [0x93, 0x08, 0xf5, 0xee] => Addi { rd: A7, rs1: A0, imm: -273, },
        check_addi_t0_t0_neg_2048:      [0x93, 0x82, 0x02, 0x80] => Addi { rd: T0, rs1: T0, imm: -2048, },

        check_and_a0_a0_a1:             [0x33, 0x75, 0xb5, 0x00] => And { rd: A0, rs1: A0, rs2: A1 },

        check_andi_a2_a2_1:             [0x13, 0x76, 0x16, 0x00] => Andi { rd: A2, rs1: A2, imm: 1 },

        check_auipc_sp_4:               [0x17, 0x41, 0x00, 0x00] => Auipc { rd: Sp, imm: 4 },
        check_auipc_gp_1:               [0x97, 0x11, 0x00, 0x00] => Auipc { rd: Gp, imm: 1 },

        check_beq_a0_zero_12:           [0x63, 0x06, 0x05, 0x00] => Beq { rs1: A0, rs2: Zero, imm: 12 },
        check_beq_a1_a0_20:             [0x63, 0xda, 0xa5, 0x00] => Bge { rs1: A1, rs2: A0, imm: 20 },

        check_bgeu_a0_a1_36:            [0x63, 0x72, 0xb5, 0x02] => Bgeu { rs1: A0, rs2: A1, imm: 36 },

        check_bltu_a1_a0_neg_16:        [0xe3, 0xe8, 0xa5, 0xfe] => Bltu { rs1: A1, rs2: A0, imm: -16 },

        check_bne_t3_t1_neg_64:          [0xe3, 0x10, 0x6e, 0xfc] => Bne { rs1: T3, rs2: T1, imm: -64 },

        // ==== TODO: All of the Csrr tests and decoding is incomplete
        // Csrr a0, mcause
        check_csrr_a0_mcause:           [0x73, 0x25, 0x20, 0x34] => Csrrc { rs1: Zero },

        // Csrr a0, mhartid
        check_cssr_a0_mhartid:          [0x73, 0x25, 0x40, 0xf1] => Csrrc { rs1: Zero },

        // Csrw mtvec, t0
        check_csrw_mtvec_t0:            [0x73, 0x90, 0x52, 0x30] => Csrrw { rs1: T0, imm12: 0 },

        // Csrwi  mie, 0
        check_csrwi_mie_0:              [0x73, 0x50, 0x40, 0x30] => Csrrwi { rd: Zero },

        // Csrwi  mip, 0
        check_csrwi_mip_0:              [0x73, 0x50, 0x40, 0x34] => Csrrwi { rd: Zero },

        // Fence  rw, rw
        check_fence_rw_rw:              [0x0f, 0x00, 0x30, 0x03] => Fence {
            rd: Zero, rs1: Zero,
            successor: 0b_0011, predecessor: 0b_0011,
            fm: 0
        },

        check_j_0:                      [0x6f, 0x00, 0x00, 0x00] => Jal { rd: Zero, imm: 0 },
        check_j_900:                    [0x6f, 0x00, 0x40, 0x38] => Jal { rd: Zero, imm: 900 },

        check_j_neg_96:                 [0x6f, 0xf0, 0x1f, 0xfa] => Jal { rd: Zero, imm: -96 },

        check_jal_76:                   [0xef, 0x00, 0xc0, 0x04] => Jal { rd: Ra, imm: 76 },

        check_jalr_a0:                  [0xe7, 0x00, 0x05, 0x00] => Jalr { rd: Ra, rs1: A0, imm: 0 },
        check_jalr_728_ra:              [0xe7, 0x80, 0x80, 0x2d] => Jalr { rd: Ra, rs1: Ra, imm: 728 },

        check_lui_a0_0:                 [0x37, 0x05, 0x00, 0x00] => Lui { rd: A0, imm: 0 },
        check_lui_a0_1:                 [0x37, 0x15, 0x00, 0x00] => Lui { rd: A0, imm: 1 },
        check_lui_a0_2:                 [0x37, 0x25, 0x00, 0x00] => Lui { rd: A0, imm: 2 },
        check_lui_a0_912092:            [0x37, 0xc5, 0xad, 0xde] => Lui { rd: A0, imm: 912092 },
        check_lui_ra_0:                 [0xb7, 0x00, 0x00, 0x00] => Lui { rd: Ra, imm: 0 },
        check_lui_t0_0:                 [0xb7, 0x02, 0x00, 0x00] => Lui { rd: T0, imm: 0 },
        check_lui_a1_0:                 [0xb7, 0x05, 0x00, 0x00] => Lui { rd: A1, imm: 0 },
        check_lui_a1_674490:            [0xb7, 0xa5, 0xab, 0xa4] => Lui { rd: A1, imm: 674490 },

        check_lw_t1_8_sp:               [0x03, 0x23, 0x81, 0x00] => Lw { rd: T1, rs1: Sp, imm: 8},
        check_lw_a6_56_sp:              [0x03, 0x28, 0x81, 0x03] => Lw { rd: A6, rs1: Sp, imm: 56},
        check_lw_t6_28_sp:              [0x83, 0x2f, 0xc1, 0x01] => Lw { rd: T6, rs1: Sp, imm: 28},

        // Mret
        check_mret:                     [0x73, 0x00, 0x20, 0x30] => Mret {},

        // Ret
        // check_ret:                      [0x67, 0x80, 0x00, 0x00] => Ret {},

        check_sb_a2_a1_0:               [0x23, 0x80, 0xc5, 0x00] => Sb { rs1: A1, rs2: A2, imm: 0 },
        check_sw_a3_sp_44:              [0x23, 0x26, 0xd1, 0x02] => Sw { rs1: Sp, rs2: A3, imm: 44},

        check_slli_a0_a0_2:             [0x13, 0x15, 0x25, 0x00] => Slli { rd: A0, rs1: A0, imm5: 2 },

        check_sub_sp_sp_t0:             [0x33, 0x01, 0x51, 0x40] => Sub { rd: Sp, rs1: Sp, rs2: T0 },

        // Wfi
        check_wfi:                      [0x73, 0x00, 0x50, 0x10] => Wfi {},

        // Xor  a2, a1, a3
        check_xor_a2_a1_a3:             [0x33, 0xc6, 0xd5, 0x00] => Xor { rd: A2, rs1: A1, rs2: A3 }
    }
}
