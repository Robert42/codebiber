use super::*;

mod section;
pub use section::{Section, Marker, Section_List};

mod parser;
pub use parser::parse as find;

pub type Result<T=(), E=Parse_Error> = std::result::Result<T, E>;

pub use parser::Syntax_Error;

#[derive(Clone, Debug, Error)]
pub enum Parse_Error
{
  #[error("syntax error: {0}")]
  SYNTAX(#[from] parser::Syntax_Error),
  #[error("invalid crc32 checksum: {0}")]
  INVALID_CHECKSUM(&'static str),
}

impl PartialEq for Parse_Error
{
  fn eq(&self, other: &Self) -> bool
  {
    use Parse_Error::*;
    match (self, other)
    {
      (SYNTAX(a), SYNTAX(b)) => a == b,
      (INVALID_CHECKSUM(a), INVALID_CHECKSUM(b)) => format!("{a}") == format!("{b}"),
      (SYNTAX(_), _) | (INVALID_CHECKSUM(_), _) => false,
    }
  }
}
impl Eq for Parse_Error {}
