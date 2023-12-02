extern crate proptest;
use proptest::prelude::*;

#[macro_use]
extern crate lazy_regex;

extern crate blake3;

#[derive(Clone, Debug)]
enum Section
{
  HANDWRITTEN(String),
}

use Section::*;

extern crate codemask;

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

fn format_expected_output(sections: &[Section], tailing_newline: bool) -> Option<String>
{
  let mut has_some_change = false;
  let mut code = String::new();
  for s in sections.into_iter()
  {
    match s
    {
      HANDWRITTEN(c) => code += c.as_str(),
    }
  }

  match has_some_change
  {
    true => Some(set_tailing_linebreak(code, tailing_newline)),
    false => None,
  }
}

proptest!
{
  #[test]
  fn roundtrip(sections in many_sections(), cfg in config(), tailing_newline: bool)
  {
    let input = format_input(&sections[..], tailing_newline);
    let expected = format_expected_output(&sections[..], tailing_newline);

    let actual = codemask::generate(input.as_str(), cfg, |_| Ok(None)).unwrap();

    assert_eq!(actual, expected);
  }
}

fn many_sections() -> impl Strategy<Value = Vec<Section>>
{
  let section = prop_oneof![
    code().prop_map(|code| Section::HANDWRITTEN(code)),
  ];

  prop::collection::vec(section, 0..16)
}

fn config() -> impl Strategy<Value = codemask::Config>
{
  prop_oneof![
    (0..(blake3::KEY_LEN as u8)).prop_map(|checksum_bytes_to_store|
      codemask::Config{
        checksum_bytes_to_store,
    }),
  ]
}

fn code() -> impl Strategy<Value = String>
{
  prop_oneof![
    "(\n)?(.*\n)*.*(\n)?".prop_filter("regular code is not allowed to contain `<< codegen`",
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