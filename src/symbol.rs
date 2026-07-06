pub use crate::types::Type;
use std::collections::HashMap;

#[cfg(feature = "codegen")]
mod backend;
#[cfg(feature = "semantic")]
mod frontend;

#[cfg(feature = "codegen")]
pub use backend::*;
#[cfg(feature = "semantic")]
pub use frontend::*;
