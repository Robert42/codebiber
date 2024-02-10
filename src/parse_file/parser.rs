use super::*;

mod line;

#[derive(Parser)]
#[grammar = "parse_file/section_grammar.pest"]
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
    
      let checksum = parse_checksum(checksum);

      Section::CODEGEN { identifier, code, checksum, begin, end }
    }
    _ => unreachable!(),
  };

  Ok(s)
}

fn parse_checksum(checksum: &str) -> ArrayVec<u8, 32>
{
  debug_assert!(checksum.len() <= 64, "I expect the parser to guarantee 32 less hex digits!\n{checksum:?}");
  debug_assert_eq!(checksum.len()%2, 0, "I expect the parser to guarantee that");

  let mut xs = ArrayVec::<u8, 32>::new();

  let checksum_bytes = checksum.as_bytes();
  for digit_pair in (0..checksum_bytes.len()/2).map(|i| [checksum_bytes[i*2], checksum_bytes[i*2+1]])
  {
    xs.push(u8_from_hex(digit_pair));
  }
  xs
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
    assert_eq!(find("").unwrap_display(), smallvec![] as Section_List);
    assert_eq!(find("xyz").unwrap_display(), smallvec![HANDWRITTEN("xyz")] as Section_List);
    assert_eq!(find("xyz\nuvw").unwrap_display(), smallvec![HANDWRITTEN("xyz\nuvw")] as Section_List);
    assert_eq!(find("// << codegen foo >>\n// << /codegen >>\n").unwrap_display(), smallvec![
      CODEGEN{
        identifier: "foo",
        code: "",
        checksum: ArrayVec::new(),
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
      smallvec![
        HANDWRITTEN("x\ny\nz\n"),
        CODEGEN{
          identifier: "blub",
          code: "  uvw\n",
          checksum: ArrayVec::new(),
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

use crate::pest::Parser;
use crate::indentation::Indentation;