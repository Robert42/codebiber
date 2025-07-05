use super::*;

mod line;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Syntax_Error
{
  EXPECTED_IDENTIFIER,
  EXPECTED_SNIPPET(&'static str),
  UNEXPECTED_END,
  CHECKSUM_WRONG_LENGTH,
  NESTED_CODEGEN_NOT_SUPPORTED,
  CODEGEN_END_WITHOUT_MATCHING_BEGIN,
}

pub fn parse<'a>(full_code: &'a str) -> Result<Section_List<'a>>
{
  let mut sections = Vec::with_capacity(16);
  let mut state_machine = State_Machine::default();

  for line in full_code.lines()
  {
    state_machine.consume_line(full_code, &mut sections, line)?;
  }

  state_machine.end(full_code, &mut sections)?;

  Ok(sections)
}

#[derive(Clone, Copy, Default)]
pub enum State_Machine<'a>
{
  #[default]
  NOTHING,
  HANDWRITTEN(&'a str),
  CODEGEN{marker: Marker<'a>, identifier: &'a str, code: Option<&'a str>},
}

impl<'a> State_Machine<'a>
{
  fn consume_line(&mut self, full_code: &'a str, sections: &mut Vec<Section<'a>>, line_span: &'a str) -> Result<()>
  {
    let syntax_error = |e: Syntax_Error| -> Parse_Error
    {
      let line = slice_join(full_code, &full_code[..0], &line_span[..0]).lines().count();
      return Parse_Error::SYNTAX(Error_Location{path: None, line}, e);
    };

    use self::line::Line;
    use State_Machine::*;
    let line = self::line::parse(line_span).map_err(syntax_error)?;

    let slice_join = |a, b| self::slice_join(full_code, a, b);
    
    *self = match (*self, line)
    {
    (NOTHING, Line::CODE(span)) => HANDWRITTEN(span),
    (NOTHING, Line::BEGIN_CODEGEN{marker, identifier}) => CODEGEN{marker, identifier, code: None},
    (HANDWRITTEN(so_far), Line::CODE(span)) => HANDWRITTEN(slice_join(so_far, span)),
    (HANDWRITTEN(so_far), Line::BEGIN_CODEGEN{marker, identifier}) =>
    {
      sections.push(Section::HANDWRITTEN(slice_join(so_far, &line_span[..0])));
      CODEGEN{marker, identifier, code: None}
    }
    (CODEGEN{marker, identifier, code: None}, Line::CODE(span)) => CODEGEN{marker, identifier, code: Some(span)},
    (CODEGEN{marker, identifier, code: Some(code)}, Line::CODE(span)) => CODEGEN{marker, identifier, code: Some(slice_join(code, span))},
    (CODEGEN{..}, Line::BEGIN_CODEGEN{..}) => Err(Syntax_Error::NESTED_CODEGEN_NOT_SUPPORTED).map_err(syntax_error)?,
    (CODEGEN{marker: begin, identifier, code}, Line::END_CODEGEN{marker: end, checksum}) =>
    {
      let checksum = parse_checksum(checksum).map_err(syntax_error)?;
      let code = code.unwrap_or(&line_span[..0]);
      let code = slice_join(code, &line_span[..0]);
      sections.push(Section::CODEGEN{identifier, code, checksum, begin, end});
      NOTHING
    }
    (_, Line::END_CODEGEN{..}) => Err(Syntax_Error::CODEGEN_END_WITHOUT_MATCHING_BEGIN).map_err(syntax_error)?,
    };

    Ok(())
  }

  fn end(self, full_code: &'a str, sections: &mut Vec<Section<'a>>) -> Result<()>
  {
    use State_Machine::*;
    match self
    {
    NOTHING => (),
    HANDWRITTEN(code) => sections.push(Section::HANDWRITTEN(slice_join(full_code, code, end_slice(full_code)))),
    CODEGEN{..} => todo!("error"),
    }

    Ok(())
  }
}

fn end_slice<'a>(slice: &'a str) -> &'a str
{
  return &slice[slice.len()..];
}

fn slice_join<'a>(full_slice: &'a str, a: &'a str, b: &'a str) -> &'a str
{
  let len = full_slice.len();
  let origin = full_slice.as_ptr();
  let begin = a.as_ptr();
  let end = b[b.len()..].as_ptr();

  let begin = begin as usize - origin as usize;
  let end = end as usize - origin as usize;

  assert!(begin <= end);
  assert!(end <= len);
  return &full_slice[begin .. end];
}

fn parse_checksum(checksum: &str) -> Result<Option<crc32::Hash>, Syntax_Error>
{
  if checksum.len() == 0
  {
    return Ok(None);
  }
  if checksum.len() != crc32::CHECKSUM_TEXT_LEN 
  {
    return Err(Syntax_Error::CHECKSUM_WRONG_LENGTH);
  }

  assert!(checksum.chars().all(|x| x.is_ascii_hexdigit()), "The parser should have guaranteed it!");

  return Ok(Some(u32::from_str_radix(checksum, 16).expect("Right number of hex chars")));
}

#[cfg(test)]
mod test
{
  use super::*;
  use Section::*;
  
  #[test]
  fn test_parse_section() -> Result
  {
    assert_eq!(parse("").unwrap(), vec![]);
    assert_eq!(parse("xyz").unwrap(), vec![HANDWRITTEN("xyz")]);
    assert_eq!(parse("x\ny\nz").unwrap(), vec![HANDWRITTEN("x\ny\nz")]);
    assert_eq!(parse("x\ny\n").unwrap(), vec![HANDWRITTEN("x\ny\n")]);

    Ok(())
  }

  #[test]
  fn trivial()
  {
    assert_eq!(find("").unwrap(), vec![] as Section_List);
    assert_eq!(find("xyz").unwrap(), vec![HANDWRITTEN("xyz")] as Section_List);
    assert_eq!(find("xyz\nuvw").unwrap(), vec![HANDWRITTEN("xyz\nuvw")] as Section_List);
    assert_eq!(find("// << codegen foo >>\n// << /codegen >>\n").unwrap(), vec![
      CODEGEN{
        identifier: "foo",
        code: "",
        checksum: None,
        begin: Marker{
          indentation: I(0),
          before_marker: "// ",
          after_marker: "",
        },
        end: Marker{
          indentation: I(0),
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
      find(code).unwrap(),
      vec![
        HANDWRITTEN("x\ny\nz\n"),
        CODEGEN{
          identifier: "blub",
          code: "  uvw\n",
          checksum: None,
          begin: Marker{
            indentation: I(2),
            before_marker: "// ",
            after_marker: "",
          },
          end: Marker{
            indentation: I(1),
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
    assert_eq!(parse_checksum("").unwrap(), None);
    assert_eq!(parse_checksum("42"), Err(Syntax_Error::CHECKSUM_WRONG_LENGTH));
    assert_eq!(parse_checksum("01234567").unwrap(), Some(0x01234567));
    assert_eq!(parse_checksum("0123456789abcdef"), Err(Syntax_Error::CHECKSUM_WRONG_LENGTH));

    let checksum = crc32::hash(b"42");
    assert_eq!(parse_checksum(crc32::fmt(checksum).as_str()).unwrap(), Some(checksum));
  }

  use Indentation as I;
}

impl fmt::Display for Syntax_Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    use Syntax_Error::*;
    match self
    {
    EXPECTED_IDENTIFIER => write!(f, "Expected identifier"),
    EXPECTED_SNIPPET(s) => write!(f, "Expected {s:?}"),
    UNEXPECTED_END => write!(f, "Unexpected end"),
    CHECKSUM_WRONG_LENGTH => write!(f, "Checksum has wrong length"),
    NESTED_CODEGEN_NOT_SUPPORTED => write!(f, "Nested code generatoin blocks are not supported"),
    CODEGEN_END_WITHOUT_MATCHING_BEGIN => write!(f, "`<< /codegen` without matching `<< codegen`"),
    }
  }
}

use crate::indentation::Indentation;
