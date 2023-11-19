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
  #[error("invalid blake3 checksum: {0}")]
  INVALID_CHECKSUM(#[from] blake3::HexError),
}

impl PartialEq for Error
{
  fn eq(&self, other: &Self) -> bool
  {
    use Error::*;
    match (self, other)
    {
      (SYNTAX(a), SYNTAX(b)) => a.eq(b),
      (INVALID_CHECKSUM(a), INVALID_CHECKSUM(b)) => format!("{a}") == format!("{b}"),
      (SYNTAX(_), _) | (INVALID_CHECKSUM(_), _) => false,
    }
  }
}
impl Eq for Error {}