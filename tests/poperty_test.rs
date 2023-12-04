#![allow(non_camel_case_types)]

use codemask::Indentation;
use codemask::Pretty_Unwrap;

extern crate proptest;
use proptest::prelude::*;

#[macro_use]
extern crate lazy_regex;

extern crate blake3;

#[derive(Clone, Debug)]
enum Section
{
  HANDWRITTEN(String),
  GENERATED{code: String, name: String, surround: Surround, generated_with_config: codemask::Config, action: Action},
}

#[derive(Clone, Debug)]
enum Action
{
  SKIP,
  KEEP,
  REPLACE_WITH(String),
}

use Section::*;
use Action::*;

extern crate codemask;

fn format_input(sections: &[Section]) -> String
{
  let mut out = String::new();
  for s in sections.iter()
  {
    match s
    {
      HANDWRITTEN(c) => out += c.as_str(),
      GENERATED{code, name, generated_with_config, surround, action: _} =>
        format_generated_code(&mut out, code.as_str(), name.as_str(), surround, *generated_with_config).unwrap(),
    }
  }
  out
}

fn format_expected_output(sections: &[Section], cfg: codemask::Config) -> Option<String>
{
  let mut has_some_change = false;
  let mut out = String::new();
  for s in sections.iter()
  {
    match s
    {
      HANDWRITTEN(c) => out += c.as_str(),
      GENERATED{code: old_code, name, generated_with_config, surround, action} => {
        has_some_change = has_some_change || generated_with_config!=&cfg;
        let code = match action {
          SKIP | KEEP => old_code,
          REPLACE_WITH(new_code) => {has_some_change = has_some_change || new_code!=old_code; new_code}
        };
        format_generated_code(&mut out, code.as_str(), name.as_str(), surround, cfg).unwrap()
      }
    }
  }

  match has_some_change
  {
    true => Some(out),
    false => None,
  }
}

fn format_generated_code(out: &mut String, code: &str, name: &str, surround: &Surround, config: codemask::Config) -> std::fmt::Result
{
  let code = surround.indent.indent_str(code);

  use std::fmt::Write;
  surround.begin(out, Some(name))?;
  write!(out, "{code}")?;
  let hash = blake3::hash(code.as_bytes()).to_hex();
  let suffix = match config.checksum_bytes_to_store
  {
    0 => None,
    n => Some(&hash[..2*n as usize]),
  };
  surround.end::<&str>(out, suffix)
}

#[derive(Clone, Debug)]
struct Surround
{
  begin: Surround_Marker,
  end: Surround_Marker,
  indent: Indentation,
}

#[derive(Clone, Debug)]
struct Surround_Marker
{
  before: String,
  after: String,
}

impl Surround
{
  fn begin<Suffix: std::fmt::Display>(&self, out: &mut String, suffix: Option<Suffix>) -> std::fmt::Result
  {
    self.begin.write(out, false, self.indent, suffix)
  }

  fn end<Suffix: std::fmt::Display>(&self, out: &mut String, suffix: Option<Suffix>) -> std::fmt::Result
  {
    self.end.write(out, true, self.indent, suffix)
  }
}

impl Surround_Marker
{
  fn write<Suffix: std::fmt::Display>(&self, out: &mut String, closing: bool, indent: Indentation, suffix: Option<Suffix>) -> std::fmt::Result
  {
    use std::fmt::Write;
    ensure_newline(out);
    
    write!(out, "{}{}<< {}codegen", indent, self.before, if closing {"/"} else {""})?;
    if let Some(suffix) = suffix
    {
      write!(out, " {}", suffix)?;
    }
    writeln!(out, " >>{}", self.after)?;

    Ok(())
  }
}

proptest!
{
  #[test]
  fn roundtrip(sections in many_sections(), cfg in config())
  {
    let input = format_input(&sections[..]);
    let expected = format_expected_output(&sections[..], cfg);

    let mut codes : Vec<Option<String>> = sections.iter().filter_map(|s| match s {
        HANDWRITTEN(..) => None,
        GENERATED { action: SKIP, .. } => Some(None),
        GENERATED { action: KEEP, code, .. }
        | GENERATED { action: REPLACE_WITH(code), .. } => Some(Some(code.clone())),
    }).collect();
    codes.reverse();
    let actual = codemask::generate(input.as_str(), cfg, move |_| Ok(codes.pop().unwrap())).pretty_expect_with_code(input.as_str());

    assert_eq!(actual, expected);
  }
}

fn many_sections() -> impl Strategy<Value = Vec<Section>>
{
  let action = prop_oneof![
    Just(SKIP),
    Just(KEEP),
    code().prop_map(|code| REPLACE_WITH(code)),
  ];

  let surround = surround();

  let section = prop_oneof![
    (code(), prop::bool::ANY).prop_map(|(code, tailing_linebreak)| Section::HANDWRITTEN(set_tailing_linebreak(code, tailing_linebreak))),
    (code(), ident(), config(), surround, action).prop_map(|(code, name, generated_with_config, surround, action)|
      Section::GENERATED{code, name, generated_with_config, surround, action}),
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

fn surround() -> impl Strategy<Value = Surround>
{
  let indent = (..u8::MAX).prop_map(|i| Indentation(i.into()));
  let surround = (surround_marker(), surround_marker(), indent).prop_map(|(begin, end, indent)| Surround{begin, end, indent});
  surround
}

fn surround_marker() -> impl Strategy<Value = Surround_Marker>
{
  let before = inline_code().prop_filter("surround_marker() /* before */", |code| !code.ends_with('<') && !code.contains("<<"));
  let after = inline_code();

  let surround = (before, after).prop_map(|(before, after)| Surround_Marker{before, after});
  surround
}

fn code() -> impl Strategy<Value = String>
{
  "(\n)?(.*\n)*.*(\n)?".prop_filter(
    "regular code is not allowed to contain `<< codegen`",
    no_marker)
}

fn inline_code() -> impl Strategy<Value = String>
{
  "[^\n]*".prop_filter(
    "regular code is not allowed to contain `<< codegen`",
    no_marker)
}

fn no_marker<S: AsRef<str>>(code: &S) -> bool {!regex_is_match!("<< *\\/?codegen", code.as_ref())}

fn ident() -> impl Strategy<Value = String>
{
  "[_a-zA-Z][_a-zA-Z0-9]*"
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

fn ensure_newline(code: &mut String)
{
  if !code.is_empty() && !code.ends_with('\n') {code.push('\n');}
}
