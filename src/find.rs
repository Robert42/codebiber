use super::*;

mod line;

use line::{parse as parse_line, Line};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section<'a>
{
  HANDWRITTEN(&'a str),
  CODEGEN{indentation: usize, identifier: &'a str, checksum: Option<blake3::Hash>, begin: Marker<'a>, end: Marker<'a>},
}
use Section::*;

#[derive(Parser)]
#[grammar = "find/section_grammar.pest"]
struct Section_Parser
{
}

pub type Section_List<'a> = SmallVec<[Section<'a> ; 8]>;

fn find(code: &str) -> Result<Section_List>
{
  let mut lines = smallvec![];

  use Rule::*;

  let result = Section_Parser::parse(file, code)?;
  for r in result
  {
    match r.as_rule()
    {
      line => match parse_line(r.into_inner().next().unwrap())?
      {
        Line::CODE(content) => lines.push(HANDWRITTEN(content)),
        Line::BEGIN_CODEGEN{..} => todo!(),
        Line::END_CODEGEN{..} => todo!(),
      },
      EOI => (),
      _ => unimplemented!("{:?}", r.as_rule()),
    }
  }
  Ok(lines)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Marker<'a>
{
  pub indentation: usize,
  pub before_marker: &'a str,
  pub after_marker: &'a str,
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
    /*
    assert_eq!(find("// << codegen foo >>\n// << /codegen >>\n")?, smallvec![CODEGEN{indentation: 0, ident: "foo", begin:"// << codegen foo >>\n", generated:"", end:"// << /codegen >>\n"}] as Section_List);
    {
      /* NOCEHCKIN
      let code = "xyz\n  // << codegen blub >>\n  uvw\n  // << /codegen >>\nabc";
      assert_eq!(
        finder.find(code)?,
        &[
          HANDWRITTEN(0..loc(code, 1, 2)),
          BEGIN_CODEGEN(2, loc(code, 1, 2)..loc(code, 2, 0)),
          GENERATED(loc(code, 2, 0)..loc(code, 3, 2)),
          END_CODEGEN(2, loc(code, 3, 2)..loc(code, 4, 0)),
          HANDWRITTEN(loc(code, 4, 0)..loc(code, 4, 3)),
        ]);
        */
    }
    */

    Ok(())
  }
}

use crate::pest::Parser;