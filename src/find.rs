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
}

fn parse_line(node: crate::pest::iterators::Pair<Rule>) -> Result<Line>
{
  use Line::*;
  use Rule::*;

  let l = match node.as_rule()
  {
    code_line => CODE(node.as_str()),
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

    Ok(())
  }


  fn parse_line(code: &str) -> Result<Line>
  {
    let mut result = Section_Parser::parse(Rule::line, code)?;

    super::parse_line(result.next().unwrap().into_inner().next().unwrap())
  }
}

use crate::pest::Parser;