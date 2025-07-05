use super::*;

mod section;
pub use section::{Section, Marker, Section_List};

mod parser;
pub use parser::parse as find;

pub type Result<T=(), E=Parse_Error> = std::result::Result<T, E>;

pub use parser::Syntax_Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parse_Error
{
  SYNTAX(parser::Syntax_Error),
}

impl fmt::Display for Parse_Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    use Parse_Error::*;
    match self
    {
    SYNTAX(e) => write!(f, "syntax error: {e}"),
    }
  }
}

use std::fmt;
