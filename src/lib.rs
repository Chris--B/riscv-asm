pub mod csr;
pub mod dis;
pub mod instr;

mod decode;

// TODO: Add an error type
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub mod prelude {
    pub use crate::decode::*;
    pub use crate::instr::*;

    pub use crate::Result;
}
