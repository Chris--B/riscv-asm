#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Privilage {
    /// User read and write
    Urw,
    /// User read-only
    Uro,

    /// Supervisor read and write
    Srw,

    /// Machine read and write
    Mrw,
    /// Machine read-only
    Mro,
}
pub use Privilage::*;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Csr(pub u16, pub Privilage);

impl Csr {
    pub const fn num(&self) -> u16 {
        self.0
    }

    pub const fn privilage(&self) -> Privilage {
        self.1
    }
}

// ====== User Trap Setup ======================================================

/// User status register
pub const USTATUS: Csr = Csr(0x000, Urw);

/// User interrupt-enable register
pub const UIE: Csr = Csr(0x004, Urw);

/// User trap handler base address
pub const UTVEC: Csr = Csr(0x005, Urw);

// ====== User Trap Handling ===================================================

/// Scratch register for user trap handlers
pub const USCRATCH: Csr = Csr(0x040, Urw);

/// User exception program counter
pub const UEPC: Csr = Csr(0x041, Urw);

/// User trap cause
pub const UCAUSE: Csr = Csr(0x042, Urw);

/// User bad address or instruction
pub const UTVAL: Csr = Csr(0x043, Urw);

/// User interrupt pending
pub const UIP: Csr = Csr(0x044, Urw);

// ====== User Floating-Point CSRs =============================================

/// Floating-Point Accrued Exceptions
pub const FFLAGS: Csr = Csr(0x001, Urw);

/// Floating-Point Dynamic Rounding Mode
pub const FRM: Csr = Csr(0x002, Urw);

/// Floating-Point Control and Status Register (`frm` + `fflags`)
pub const FCSR: Csr = Csr(0x003, Urw);

// ====== User Counter/Timers ==================================================

/// Cycle counter for `RDCYCLE` instruction
pub const CYCLE: Csr = Csr(0xC00, Uro);

/// Timer for `RDTIME` instruction
pub const TIME: Csr = Csr(0xC01, Uro);

/// Instructions-retired counter for `RDINSTRET` instruction
pub const INSTRET: Csr = Csr(0xC02, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER3: Csr = Csr(0xC03, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER4: Csr = Csr(0xC04, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER5: Csr = Csr(0xC05, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER6: Csr = Csr(0xC06, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER7: Csr = Csr(0xC07, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER8: Csr = Csr(0xC08, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER9: Csr = Csr(0xC09, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER10: Csr = Csr(0xC0A, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER11: Csr = Csr(0xC0B, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER12: Csr = Csr(0xC0C, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER13: Csr = Csr(0xC0D, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER14: Csr = Csr(0xC0E, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER15: Csr = Csr(0xC0F, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER16: Csr = Csr(0xC10, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER17: Csr = Csr(0xC11, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER18: Csr = Csr(0xC12, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER19: Csr = Csr(0xC13, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER20: Csr = Csr(0xC14, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER21: Csr = Csr(0xC15, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER22: Csr = Csr(0xC16, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER23: Csr = Csr(0xC17, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER24: Csr = Csr(0xC18, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER25: Csr = Csr(0xC19, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER26: Csr = Csr(0xC1A, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER27: Csr = Csr(0xC1B, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER28: Csr = Csr(0xC1C, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER29: Csr = Csr(0xC1D, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER30: Csr = Csr(0xC1E, Uro);

/// Performance-monitoring counter
pub const HPMCOUNTER31: Csr = Csr(0xC1F, Uro);

/// Upper bits of `cycle`, RV32I only
pub const CYCLE_H: Csr = Csr(0xC80, Uro);

/// Upper bits of `time`, RV32I only
pub const TIME_H: Csr = Csr(0xC81, Uro);

/// Upper bits of `instret`, RV32I only
pub const INSTRET_H: Csr = Csr(0xC82, Uro);

/// Upper bits of `hpmcounter3`, RV32I only
pub const HPMCOUNTER3_H: Csr = Csr(0xC83, Uro);

/// Upper bits of `hpmcounter4`, RV32I only
pub const HPMCOUNTER4_H: Csr = Csr(0xC84, Uro);

/// Upper bits of `hpmcounter5`, RV32I only
pub const HPMCOUNTER5_H: Csr = Csr(0xC85, Uro);

/// Upper bits of `hpmcounter6`, RV32I only
pub const HPMCOUNTER6_H: Csr = Csr(0xC86, Uro);

/// Upper bits of `hpmcounter7`, RV32I only
pub const HPMCOUNTER7_H: Csr = Csr(0xC87, Uro);

/// Upper bits of `hpmcounter8`, RV32I only
pub const HPMCOUNTER8_H: Csr = Csr(0xC88, Uro);

/// Upper bits of `hpmcounter9`, RV32I only
pub const HPMCOUNTER9_H: Csr = Csr(0xC89, Uro);

/// Upper bits of `hpmcounter10`, RV32I only
pub const HPMCOUNTER10_H: Csr = Csr(0xC8A, Uro);

/// Upper bits of `hpmcounter11`, RV32I only
pub const HPMCOUNTER11_H: Csr = Csr(0xC8B, Uro);

/// Upper bits of `hpmcounter12`, RV32I only
pub const HPMCOUNTER12_H: Csr = Csr(0xC8C, Uro);

/// Upper bits of `hpmcounter13`, RV32I only
pub const HPMCOUNTER13_H: Csr = Csr(0xC8D, Uro);

/// Upper bits of `hpmcounter14`, RV32I only
pub const HPMCOUNTER14_H: Csr = Csr(0xC8E, Uro);

/// Upper bits of `hpmcounter15`, RV32I only
pub const HPMCOUNTER15_H: Csr = Csr(0xC8F, Uro);

/// Upper bits of `hpmcounter16`, RV32I only
pub const HPMCOUNTER16_H: Csr = Csr(0xC90, Uro);

/// Upper bits of `hpmcounter17`, RV32I only
pub const HPMCOUNTER17_H: Csr = Csr(0xC91, Uro);

/// Upper bits of `hpmcounter18`, RV32I only
pub const HPMCOUNTER18_H: Csr = Csr(0xC92, Uro);

/// Upper bits of `hpmcounter19`, RV32I only
pub const HPMCOUNTER19_H: Csr = Csr(0xC93, Uro);

/// Upper bits of `hpmcounter20`, RV32I only
pub const HPMCOUNTER20_H: Csr = Csr(0xC94, Uro);

/// Upper bits of `hpmcounter21`, RV32I only
pub const HPMCOUNTER21_H: Csr = Csr(0xC95, Uro);

/// Upper bits of `hpmcounter22`, RV32I only
pub const HPMCOUNTER22_H: Csr = Csr(0xC96, Uro);

/// Upper bits of `hpmcounter23`, RV32I only
pub const HPMCOUNTER23_H: Csr = Csr(0xC97, Uro);

/// Upper bits of `hpmcounter24`, RV32I only
pub const HPMCOUNTER24_H: Csr = Csr(0xC98, Uro);

/// Upper bits of `hpmcounter25`, RV32I only
pub const HPMCOUNTER25_H: Csr = Csr(0xC99, Uro);

/// Upper bits of `hpmcounter26`, RV32I only
pub const HPMCOUNTER26_H: Csr = Csr(0xC9A, Uro);

/// Upper bits of `hpmcounter27`, RV32I only
pub const HPMCOUNTER27_H: Csr = Csr(0xC9B, Uro);

/// Upper bits of `hpmcounter28`, RV32I only
pub const HPMCOUNTER28_H: Csr = Csr(0xC9C, Uro);

/// Upper bits of `hpmcounter29`, RV32I only
pub const HPMCOUNTER29_H: Csr = Csr(0xC9D, Uro);

/// Upper bits of `hpmcounter30`, RV32I only
pub const HPMCOUNTER30_H: Csr = Csr(0xC9E, Uro);

/// Upper bits of `hpmcounter31`, RV32I only
pub const HPMCOUNTER31_H: Csr = Csr(0xC9F, Uro);

// ===== Supervisor Trap Setup =================================================
// TODO: Omitted for brevity at this time

// ===== Supervisor Trap Handling ==============================================
// TODO: Omitted for brevity at this time

// ===== Supervisor Protection and Translation =================================
// TODO: Omitted for brevity at this time

// ===== Machine Information Registers =========================================

/// Vendor ID
pub const MVENDORID: Csr = Csr(0xF11, Mro);

/// Architecture ID
pub const MARCHID: Csr = Csr(0xF12, Mro);

/// Implementation ID
pub const MIMPID: Csr = Csr(0xF13, Mro);

/// Hardware thread ID
pub const MHARTID: Csr = Csr(0xF14, Mro);

// ===== Machine Trap Setup ====================================================

/// Machine status register
pub const MSTATUS: Csr = Csr(0x300, Mrw);

/// ISA and extensions
pub const MISA: Csr = Csr(0x301, Mrw);

/// Machine exception delegation register
pub const MEDELEG: Csr = Csr(0x302, Mrw);

/// Machine interrupt delegation register
pub const MIDELEG: Csr = Csr(0x303, Mrw);

/// Machine interrupt-enable register
pub const MIE: Csr = Csr(0x304, Mrw);

/// Machine trap-handler base address
pub const MTVEC: Csr = Csr(0x305, Mrw);

/// Machine counter enable
pub const MCOUNTEREN: Csr = Csr(0x306, Mrw);

// ===== Machine Trap Handling =================================================

/// Scratch register for machine trap handlers
pub const MSCRATCH: Csr = Csr(0x340, Mrw);

/// Machine exception program counter
pub const MEPC: Csr = Csr(0x341, Mrw);

/// Machine trap cause
pub const MCAUSE: Csr = Csr(0x342, Mrw);

/// Machine bad address or instruction
pub const MTVAL: Csr = Csr(0x343, Mrw);

/// Machine interrupt pending
pub const MIP: Csr = Csr(0x344, Mrw);

// ===== Machine Memory Protection =============================================
// TODO: Omitted for brevity at this time
