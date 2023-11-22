use super::*;

#[cfg(test)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line<'a>
{
  CODE(&'a str),
  BEGIN_CODEGEN{marker: Marker<'a>, identifier: &'a str,},
  END_CODEGEN{marker: Marker<'a>, checksum: &'a str,},
}

#[cfg(test)]
pub fn parse(node: crate::pest::iterators::Pair<Rule>) -> Result<Line>
{
  use Line::*;
  use Rule::{code_line, begin_marker_line, end_marker_line};

  let l = match node.as_rule()
  {
    code_line => CODE(node.as_str()),
    begin_marker_line =>
    {
      let (marker, identifier) = parse_marker(node);
      Line::BEGIN_CODEGEN{marker, identifier}
    }
    end_marker_line =>
    {
      let (marker, checksum) = parse_marker(node);
      Line::END_CODEGEN{marker, checksum}
    }
    _ => unimplemented!("{:?}", node.as_rule()),
  };

  return Ok(l);
}

pub fn parse_begin_marker(node: crate::pest::iterators::Pair<Rule>) -> (Marker, &str)
{
  debug_assert!(node.as_rule() == Rule::begin_marker_line);
  return parse_marker(node);
}

pub fn parse_end_marker(node: crate::pest::iterators::Pair<Rule>) -> (Marker, &str)
{
  debug_assert!(node.as_rule() == Rule::end_marker_line);
  return parse_marker(node);
}

fn parse_marker(node: crate::pest::iterators::Pair<Rule>) -> (Marker, &str)
{
  debug_assert!(node.as_rule() == Rule::begin_marker_line || node.as_rule() == Rule::end_marker_line);
  let mut xs = node.into_inner();

  let indentation = xs.next().unwrap();
  let before_marker = xs.next().unwrap();

  debug_assert_eq!(indentation.as_rule(), Rule::indentation);
  let indentation = Indentation(indentation.as_str().len());

  debug_assert_eq!(before_marker.as_rule(), Rule::before_marker);
  let before_marker = before_marker.as_str();
  
  let identifier = xs.next().unwrap();
  let (identifier, after_marker) = match identifier.as_rule()
  {
    Rule::identifier | Rule::checksum =>
    {
      let after_marker = xs.next().unwrap();
      debug_assert_eq!(after_marker.as_rule(), Rule::after_marker);
      let after_marker = after_marker.as_str();

      (identifier.as_str(), after_marker)
    }
    Rule::after_marker => ("", identifier.as_str()),
    _ => unreachable!("Rule::{:?} span: {:?}", identifier.as_rule(), identifier.as_str()),
  };

  (Marker{indentation, before_marker, after_marker}, identifier)
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn lines() -> Result
  {
    let indentation = I(2);

    assert_eq!(parse_line("")?, Line::CODE(""));
    assert_eq!(parse_line("xyz")?, Line::CODE("xyz"));
    assert_eq!(parse_line("  // << codegen foo >> let's go!")?, Line::BEGIN_CODEGEN{identifier: "foo", marker: Marker{indentation, before_marker: "// ", after_marker: " let's go!"}});
    assert_eq!(parse_line("  // << /codegen f00baa >> nice!")?, Line::END_CODEGEN{checksum: "f00baa", marker: Marker{indentation, before_marker: "// ", after_marker: " nice!"}});
    assert_eq!(parse_line("  // << /codegen >> nice!")?, Line::END_CODEGEN{checksum: "", marker: Marker{indentation, before_marker: "// ", after_marker: " nice!"}});
    assert_eq!(parse_line("  // << /codegen>> nice!")?, Line::END_CODEGEN{checksum: "", marker: Marker{indentation, before_marker: "// ", after_marker: " nice!"}});

    Ok(())
  }


  fn parse_line(code: &str) -> Result<Line>
  {
    let mut result = Section_Parser::parse(Rule::line, code)?;

    super::parse(result.next().unwrap().into_inner().next().unwrap())
  }

  use Indentation as I;
}