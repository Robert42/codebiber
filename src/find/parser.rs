use super::*;

mod line;

#[derive(Parser)]
#[grammar = "find/section_grammar.pest"]
pub struct Section_Parser
{
}

pub type Syntax_Error = crate::pest::error::Error<Rule>;

pub fn parse(code: &str) -> Result<Section_List>
{
  let mut sections = smallvec![];

  let result = Section_Parser::parse(Rule::file, code)?;
  for r in result
  {
    match r.as_rule()
    {
      Rule::section => sections.push(parse_section(r)?),
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
    Rule::generated => {
      let mut xs = node.into_inner();
      let (begin, identifier) = line::parse_begin_marker(xs.next().unwrap());
      let code = xs.next().unwrap().as_str();
      let (end, checksum) = line::parse_end_marker(xs.next().unwrap());
    
      let checksum = parse_checksum(checksum)?;

      Section::CODEGEN { identifier, code, checksum, begin, end }
    }
    _ => unreachable!(),
  };

  Ok(s)
}

fn parse_checksum(checksum: &str) -> Result<Option<blake3::Hash>>
{
  match checksum.as_bytes()
  {
    [] => Ok(None),
    _ => Ok(Some(blake3::Hash::from_hex(checksum)?)),
  }
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
  fn trivial()
  {
    assert_eq!(find("").pretty_unwrap(), smallvec![] as Section_List);
    assert_eq!(find("xyz").pretty_unwrap(), smallvec![HANDWRITTEN("xyz")] as Section_List);
    assert_eq!(find("xyz\nuvw").pretty_unwrap(), smallvec![HANDWRITTEN("xyz\nuvw")] as Section_List);
    assert_eq!(find("// << codegen foo >>\n// << /codegen >>\n").pretty_unwrap(), smallvec![
      CODEGEN{
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
  }
  
  #[test]
  fn test_multiple_sections()
  {
    let code = "x\ny\nz\n  // << codegen blub >>\n  uvw\n // << /codegen >>\nabc";
    assert_eq!(
      find(code).pretty_unwrap(),
      smallvec![
        HANDWRITTEN("x\ny\nz\n"),
        CODEGEN{
          identifier: "blub",
          code: "  uvw\n",
          checksum: None,
          begin: Marker{
            indentation: 2,
            before_marker: "// ",
            after_marker: "",
          },
          end: Marker{
            indentation: 1,
            before_marker: "// ",
            after_marker: "",
          },
        },
        HANDWRITTEN("abc"),
      ] as Section_List);
  }
  
  #[test]
  fn test_checksum()
  {
    assert_eq!(parse_checksum("").pretty_unwrap(), None);
    assert_eq!(parse_checksum("x").is_err(), true);
    assert_eq!(parse_checksum("42").is_err(), true);

    let checksum = blake3::hash(b"42");
    assert_eq!(parse_checksum(checksum.to_string().as_str()).pretty_unwrap(), Some(checksum));
  }
}

use crate::pest::Parser;