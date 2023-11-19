use super::*;

mod parser;
pub use parser::parse as find;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section<'a>
{
  HANDWRITTEN(&'a str),
  CODEGEN{indentation: usize, identifier: &'a str, checksum: Option<blake3::Hash>, begin: Marker<'a>, end: Marker<'a>},
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker<'a>
{
  pub indentation: usize,
  pub before_marker: &'a str,
  pub after_marker: &'a str,
}

pub type Section_List<'a> = SmallVec<[Section<'a> ; 8]>;

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Error)]
pub enum Error
{
  #[error("syntax error: {0}")]
  SYNTAX(#[from] parser::Syntax_Error),
}