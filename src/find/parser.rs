use super::*;

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
  use Rule::*;

  let mut sections = smallvec![];

  let result = Section_Parser::parse(file, code)?;
  for r in result
  {
    match r.as_rule()
    {
      line => match parse_line(r.into_inner().next().unwrap())?
      {
        Line::CODE(content) => sections.push(HANDWRITTEN(content)),
        Line::BEGIN_CODEGEN{..} => todo!(),
        Line::END_CODEGEN{..} => todo!(),
      },
      EOI => (),
      _ => unimplemented!("{:?}", r.as_rule()),
    }
  }

  Ok(sections)
}

#[cfg(test)]
mod test
{
  use super::*;
  use Section::*;
  
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