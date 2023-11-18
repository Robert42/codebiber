use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section<'a>
{
  HANDWRITTEN(&'a str),
}
use Section::*;

#[derive(Parser)]
#[grammar = "section_grammar.pest"]
struct Section_Parser
{
}

type Section_List<'a> = SmallVec<[Section<'a> ; 8]>;

fn find(code: &str) -> Result<Section_List>
{
  let mut lines = smallvec![];

  use Rule::*;

  let result = Section_Parser::parse(file, code)?;
  for r in result
  {
    match r.as_rule()
    {
      line => match parse_line(r)?
      {
        Line::HANDWRITTEN(content) => lines.push(HANDWRITTEN(content)),
      },
      EOI => (),
      _ => unimplemented!("{:?}", r.as_rule()),
    }
  }
  Ok(lines)
}

enum Line<'a>
{
  HANDWRITTEN(&'a str),
}

fn parse_line(node: crate::pest::iterators::Pair<Rule>) -> Result<Line>
{
  use Line::*;

  return Ok(HANDWRITTEN(node.as_str()))
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Error)]
pub enum Error
{
  #[error("syntax error: {0}")]
  SYNTAX(#[from] crate::pest::error::Error<Rule>),
}

#[cfg(test)]
mod test
{
  use super::*;
  
  #[test]
  fn trivial() -> Result
  {
    assert_eq!(find("")?, smallvec![HANDWRITTEN("")] as Section_List);
    assert_eq!(find("xyz")?, smallvec![HANDWRITTEN("xyz")] as Section_List);
    assert_eq!(find("xyz\nuvw")?, smallvec![HANDWRITTEN("xyz"), HANDWRITTEN("uvw")] as Section_List);

    Ok(())
  }
}

use crate::pest::Parser;