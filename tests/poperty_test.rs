extern crate proptest;
use proptest::prelude::*;

#[macro_use]
extern crate lazy_regex;

#[derive(Clone, Debug)]
enum Section
{
  HANDWRITTEN(String),
}

use Section::*;

fn format_input(sections: &[Section], tailing_newline: bool) -> String
{
  let mut code = String::new();
  for s in sections.into_iter()
  {
    match s
    {
      HANDWRITTEN(c) => code += c.as_str(),
    }
  }
  set_tailing_linebreak(code, tailing_newline)
}

fn format_expected_output(sections: &[Section], tailing_newline: bool) -> String
{
  let mut code = String::new();
  for s in sections.into_iter()
  {
    match s
    {
      HANDWRITTEN(c) => code += c.as_str(),
    }
  }
  set_tailing_linebreak(code, tailing_newline)
}

proptest!
{
  #[test]
  fn roundtrip(sections in many_sections(), hash_bytes in 0..64, tailing_newline: bool)
  {
    let input = format_input(&sections[..], tailing_newline);
    let expected = format_expected_output(&sections[..], tailing_newline);

    assert_eq!(input, expected);
  }
}

fn many_sections() -> impl Strategy<Value = Vec<Section>>
{
  let section = prop_oneof![
    code().prop_map(|code| Section::HANDWRITTEN(code)),
  ];

  prop::collection::vec(section, 0..16)
}

fn code() -> impl Strategy<Value = String>
{
  prop_oneof![
    ".*".prop_filter("regular code is not allowed to contain `<< codegen`",
      |code| !regex_is_match!("<< *\\/?codegen", &code)
    )
  ]
}

fn set_tailing_linebreak(mut code: String, expect_tailing_linebreak: bool) -> String
{
  let len = code.len();
  let ends_with_linebreak = code.as_bytes().last().cloned() == Some(b'\n');

  match (expect_tailing_linebreak, ends_with_linebreak)
  {
    (false, false) | (true, true) => (),
    (false, true) => code.truncate(len-1),
    (true, false) => code.push('\n'),
  }

  code
}