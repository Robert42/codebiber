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
  code
}

proptest!
{
  #[test]
  fn roundtrip(sections in many_sections(), tailing_newline: bool)
  {
    let input = format_input(&sections[..], tailing_newline);
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
