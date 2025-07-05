use super::*;

mod section;
pub use section::{Section, Marker, Section_List};

mod parser;
pub use parser::parse as find;

pub type Result<T=(), E=Parse_Error> = std::result::Result<T, E>;

pub use parser::Syntax_Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error_Location
{
  pub path: Option<std::path::PathBuf>,
  pub line: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Parse_Error
{
  SYNTAX(Error_Location, parser::Syntax_Error),
}

impl fmt::Display for Parse_Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    use Parse_Error::*;
    match self
    {
    SYNTAX(loc, e) => write!(f, "syntax error: {e} at {loc}"),
    }
  }
}

impl fmt::Display for Error_Location
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    let line = self.line + 1;
    match &self.path
    {
    Some(path) => write!(f, "{path}:{line}", path=path.display()),
    None => write!(f, "line {line}"),
    }
  }
}

use std::fmt;
