use super::*;

mod line;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum Syntax_Error
{
  #[error("Expected identifier")]
  EXPECTED_IDENTIFIER,
  #[error("Expected {0:?}")]
  EXPECTED_SNIPPET(&'static str),
  #[error("Unexpected end")]
  UNEXPECTED_END,
  #[error("Too long checksum")]
  CHECKSUM_TOO_LONG,
  #[error("Odd number of hex digits do not encode bytes")]
  CHECKSUM_NOT_EVEN,
  #[error("Nested code generatoin blocks are not supported")]
  NESTED_CODEGEN_NOT_SUPPORTED,
  #[error("`<< /codegen` without matching `<< codegen`")]
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
    use self::line::Line;
    use State_Machine::*;
    let line = self::line::parse(line_span)?;

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
    (CODEGEN{..}, Line::BEGIN_CODEGEN{..}) => Err(Syntax_Error::NESTED_CODEGEN_NOT_SUPPORTED)?,
    (CODEGEN{marker: begin, identifier, code}, Line::END_CODEGEN{marker: end, checksum}) =>
    {
      let checksum = parse_checksum(checksum)?;
      let code = code.unwrap_or(&line_span[..0]);
      let code = slice_join(code, &line_span[..0]);
      sections.push(Section::CODEGEN{identifier, code, checksum, begin, end});
      NOTHING
    }
    (_, Line::END_CODEGEN{..}) => Err(Syntax_Error::CODEGEN_END_WITHOUT_MATCHING_BEGIN)?,
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

fn parse_checksum(checksum: &str) -> Result<Vec<u8>, Syntax_Error>
{
  if checksum.len() > 64 
  {
    return Err(Syntax_Error::CHECKSUM_TOO_LONG);
  }
  if checksum.len()%2 != 0
  {
    return Err(Syntax_Error::CHECKSUM_NOT_EVEN);
  }

  let mut xs = Vec::<u8>::with_capacity(32);

  let checksum_bytes = checksum.as_bytes();
  for digit_pair in (0..checksum_bytes.len()/2).map(|i| [checksum_bytes[i*2], checksum_bytes[i*2+1]])
  {
    xs.push(u8_from_hex(digit_pair));
  }
  return Ok(xs);
}

fn hex_digit(digit: u8) -> u8
{
  match digit
  {
    b'0' ..= b'9' => digit - b'0',
    b'a' ..= b'f' => digit - b'a' + 10,
    b'A' ..= b'F' => digit - b'A' + 10,
    _ => unreachable!("{digit:?}"),
  }
}

fn u8_from_hex(digits: [u8; 2]) -> u8
{
  debug_assert!(digits[0].is_ascii_hexdigit() && digits[1].is_ascii_hexdigit());
  (hex_digit(digits[0])<<4) | hex_digit(digits[1])
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
    assert_eq!(find("").unwrap_display(), vec![] as Section_List);
    assert_eq!(find("xyz").unwrap_display(), vec![HANDWRITTEN("xyz")] as Section_List);
    assert_eq!(find("xyz\nuvw").unwrap_display(), vec![HANDWRITTEN("xyz\nuvw")] as Section_List);
    assert_eq!(find("// << codegen foo >>\n// << /codegen >>\n").unwrap_display(), vec![
      CODEGEN{
        identifier: "foo",
        code: "",
        checksum: Vec::new(),
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
      find(code).unwrap_display(),
      vec![
        HANDWRITTEN("x\ny\nz\n"),
        CODEGEN{
          identifier: "blub",
          code: "  uvw\n",
          checksum: Vec::new(),
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
    assert_eq!(hex_digit(b'0'), 0);
    assert_eq!(hex_digit(b'9'), 9);
    assert_eq!(hex_digit(b'a'), 10);
    assert_eq!(hex_digit(b'f'), 15);
    assert_eq!(hex_digit(b'A'), 10);
    assert_eq!(hex_digit(b'F'), 15);
    assert_eq!(u8_from_hex([b'4', b'2']), 0x42);

    assert_eq!(parse_checksum("").as_slice(), &[]);
    assert_eq!(parse_checksum("42").as_slice(), &[0x42]);
    assert_eq!(parse_checksum("0123456789abcdef").as_slice(), &[0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);

    let checksum = blake3::hash(b"42");
    assert_eq!(parse_checksum(checksum.to_string().as_str()).as_slice(), checksum.as_bytes());
  }

  use Indentation as I;
}

use crate::indentation::Indentation;
