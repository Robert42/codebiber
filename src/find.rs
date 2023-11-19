use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section<'a>
{
  HANDWRITTEN(&'a str),
  CODEGEN{indentation: usize, ident: &'a str, begin: &'a str, generated: &'a str, end: &'a str},
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
      line => match parse_line(r.into_inner().next().unwrap())?
      {
        Line::CODE(content) => lines.push(HANDWRITTEN(content)),
        Line::BEGIN_CODEGEN{..} => todo!(),
      },
      EOI => (),
      _ => unimplemented!("{:?}", r.as_rule()),
    }
  }
  Ok(lines)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum Line<'a>
{
  CODE(&'a str),
  BEGIN_CODEGEN{indentation: u32, identifier: &'a str, before_marker: &'a str, after_marker: &'a str},
}

fn parse_line(node: crate::pest::iterators::Pair<Rule>) -> Result<Line>
{
  use Line::*;
  use Rule::{code_line, begin_marker_line};

  let l = match node.as_rule()
  {
    code_line => CODE(node.as_str()),
    begin_marker_line =>
    {
      let mut xs = node.into_inner();
      let indentation = xs.next().unwrap();
      let before_marker = xs.next().unwrap();
      let identifier = xs.next().unwrap();
      let after_marker = xs.next().unwrap();

      debug_assert_eq!(indentation.as_rule(), Rule::indentation);
      let indentation = indentation.as_str().len() as u32; // TODO catch?

      debug_assert_eq!(before_marker.as_rule(), Rule::before_marker);
      let before_marker = before_marker.as_str();
      
      debug_assert_eq!(identifier.as_rule(), Rule::identifier);
      let identifier = identifier.as_str();

      debug_assert_eq!(after_marker.as_rule(), Rule::after_marker);
      let after_marker = after_marker.as_str();

      Line::BEGIN_CODEGEN{indentation, before_marker, identifier, after_marker}
    }
    _ => unimplemented!("{:?}", node.as_rule()),
  };

  return Ok(l);
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
  
  #[test]
  fn lines() -> Result
  {
    assert_eq!(parse_line("")?, Line::CODE(""));
    assert_eq!(parse_line("xyz")?, Line::CODE("xyz"));
    assert_eq!(parse_line("  // << codegen foo >> let's go!")?, Line::BEGIN_CODEGEN{indentation: 2, identifier: "foo", before_marker: "// ", after_marker: " let's go!"});

    Ok(())
  }


  fn parse_line(code: &str) -> Result<Line>
  {
    let mut result = Section_Parser::parse(Rule::line, code)?;

    super::parse_line(result.next().unwrap().into_inner().next().unwrap())
  }
}

use crate::pest::Parser;