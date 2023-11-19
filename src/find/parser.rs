use super::*;

#[cfg(test)]
mod line;
use line::{parse as parse_line, Line};

#[derive(Parser)]
#[grammar = "find/section_grammar.pest"]
pub struct Section_Parser
{
}

pub type Syntax_Error = crate::pest::error::Error<Rule>;

pub fn parse(code: &str) -> Result<Section_List>
{
  use Section::*;

  let mut sections = smallvec![];

  let result = Section_Parser::parse(Rule::file, code)?;
  for r in result
  {
    match r.as_rule()
    {
      Rule::line => match parse_line(r.into_inner().next().unwrap())?
      {
        Line::CODE(content) => sections.push(HANDWRITTEN(content)),
        Line::BEGIN_CODEGEN{..} => todo!(),
        Line::END_CODEGEN{..} => todo!(),
      },
      Rule::EOI => (),
      _ => unimplemented!("{:?}", r.as_rule()),
    }
  }

  Ok(sections)
}

fn parse_section(node: crate::pest::iterators::Pair<Rule>) -> Result<Section>
{
  debug_assert_eq!(node.as_rule(), Rule::section);
  
  let node = node.into_inner().next().unwrap();
  let s = match node.as_rule()
  {
    Rule::code => Section::HANDWRITTEN(node.as_str()),
    Rule::generated => todo!(),
    _ => unreachable!(),
  };

  Ok(s)
}

#[cfg(test)]
mod test
{
  use super::*;
  use Section::*;
  
  #[test]
  fn test_parse_section() -> Result
  {
    assert!(parse_section("").is_err());
    assert_eq!(parse_section("xyz")?, HANDWRITTEN("xyz"));
    assert_eq!(parse_section("x\ny\nz")?, HANDWRITTEN("x\ny\nz"));
    assert_eq!(parse_section("x\ny\n")?, HANDWRITTEN("x\ny\n"));

    Ok(())
  }

  fn parse_section(code: &str) -> Result<Section>
  {
    let mut result = Section_Parser::parse(Rule::section, code)?;
  
    super::parse_section(result.next().unwrap())
  }
  
  #[test]
  fn trivial() -> Result
  {
    assert_eq!(find("")?, smallvec![HANDWRITTEN("")] as Section_List);
    assert_eq!(find("xyz")?, smallvec![HANDWRITTEN("xyz")] as Section_List);
    assert_eq!(find("xyz\nuvw")?, smallvec![HANDWRITTEN("xyz"), HANDWRITTEN("uvw")] as Section_List);
    /*
    assert_eq!(find("// << codegen foo >>\n// << /codegen >>\n")?, smallvec![
      CODEGEN{
        indentation: 0,
        identifier: "foo",
        code: "",
        checksum: None,
        begin: Marker{
          indentation: 0,
          before_marker: "// ",
          after_marker: "",
        },
        end: Marker{
          indentation: 0,
          before_marker: "// ",
          after_marker: "",
        },
      },
    ] as Section_List);
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