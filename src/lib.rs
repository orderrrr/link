extern crate pest;

pub mod err;
pub use crate::err::LocatedError;

#[macro_use]
extern crate pest_derive;

#[derive(Parser)]
#[grammar = "gram.pest"]
pub struct LP;

pub type Result<T> = anyhow::Result<T>;

pub mod ast;
pub mod byte;
pub mod op;
pub mod parse;
pub mod vm;

pub use crate::ast::*;
pub use crate::byte::*;
pub use crate::err::*;
pub use crate::op::*;
pub use crate::parse::*;
pub use crate::vm::*;
