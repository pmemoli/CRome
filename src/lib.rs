#[cfg(feature = "codegen")]
pub mod codegen;
#[cfg(feature = "emission")]
pub mod emission;
#[cfg(feature = "lex")]
pub mod lexer;
#[cfg(feature = "parser")]
pub mod parser;
#[cfg(feature = "semantic")]
pub mod semantic;
#[cfg(feature = "semantic")]
pub mod symbol;
#[cfg(feature = "tacky")]
pub mod tacky;
#[cfg(feature = "parser")]
pub mod types;

pub mod driver;
