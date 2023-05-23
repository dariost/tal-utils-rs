pub mod parser;
pub mod tc;

pub use parser::{Parser, ParserError};
pub use tc::{gen_data, run_tc, RunOptions, Verdict};
