use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line<'a>
{
  CODE(&'a str),
  BEGIN_CODEGEN{marker: Marker<'a>, identifier: &'a str,},
  END_CODEGEN{marker: Marker<'a>, checksum: &'a str,},
}

pub fn parse(node: crate::pest::iterators::Pair<Rule>) -> Result<Line>
{
  use Line::*;
  use Rule::{code_line, begin_marker_line, end_marker_line};

  let l = match node.as_rule()
  {
    code_line => CODE(node.as_str()),
    begin_marker_line =>
    {
      let (marker, identifier) = parse_marker(node.into_inner());
      Line::BEGIN_CODEGEN{marker, identifier}
    }
    end_marker_line =>
    {
      let (marker, checksum) = parse_marker(node.into_inner());
      Line::END_CODEGEN{marker, checksum}
    }
    _ => unimplemented!("{:?}", node.as_rule()),
  };

  return Ok(l);
}

fn parse_marker(mut xs: crate::pest::iterators::Pairs<Rule>) -> (Marker, &str)
{
  let indentation = xs.next().unwrap();
  let before_marker = xs.next().unwrap();
  let identifier = xs.next().unwrap();
  let after_marker = xs.next().unwrap();

  debug_assert_eq!(indentation.as_rule(), Rule::indentation);
  let indentation = indentation.as_str().len() as usize;

  debug_assert_eq!(before_marker.as_rule(), Rule::before_marker);
  let before_marker = before_marker.as_str();
  
  let identifier = match identifier.as_rule()
  {
    Rule::identifier | Rule::checksum => identifier.as_str(),
    _ => unreachable!("{}", identifier.as_str()),
  };

  debug_assert_eq!(after_marker.as_rule(), Rule::after_marker);
  let after_marker = after_marker.as_str();

  (Marker{indentation, before_marker, after_marker}, identifier)
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn lines() -> Result
  {
    assert_eq!(parse_line("")?, Line::CODE(""));
    assert_eq!(parse_line("xyz")?, Line::CODE("xyz"));
    assert_eq!(parse_line("  // << codegen foo >> let's go!")?, Line::BEGIN_CODEGEN{identifier: "foo", marker: Marker{indentation: 2, before_marker: "// ", after_marker: " let's go!"}});
    assert_eq!(parse_line("  // << /codegen f00ba >> nice!")?, Line::END_CODEGEN{checksum: "f00ba", marker: Marker{indentation: 2, before_marker: "// ", after_marker: " nice!"}});

    Ok(())
  }


  fn parse_line(code: &str) -> Result<Line>
  {
    let mut result = Section_Parser::parse(Rule::line, code)?;

    super::parse_line(result.next().unwrap().into_inner().next().unwrap())
  }
}