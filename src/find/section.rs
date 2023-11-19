use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section<'a>
{
  HANDWRITTEN(&'a str),
  CODEGEN{identifier: &'a str, code: &'a str, checksum: Option<blake3::Hash>, begin: Marker<'a>, end: Marker<'a>},
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marker<'a>
{
  pub indentation: Indentation,
  pub before_marker: &'a str,
  pub after_marker: &'a str,
}

pub type Section_List<'a> = SmallVec<[Section<'a> ; 8]>;
use crate::indentation::Indentation;