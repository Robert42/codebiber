use super::*;

mod section;
pub use section::{Section, Marker, Section_List};

mod parser;
pub use parser::parse as find;

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Error)]
pub enum Error
{
  #[error("syntax error: {0}")]
  SYNTAX(#[from] parser::Syntax_Error),
}