use super::*;

use super::parse_file::{find as parse_sections, Section};
use indentation::ensure_tailing_linebreak;

pub type Fmt_Result<T=Option<String>> = std::result::Result<T, std::fmt::Error>;

pub fn generate<F>(input: &str, mut f: F) -> Result<Option<String>>
where F: FnMut(&str) -> Fmt_Result
{
  use Section::*;
  let sections = parse_sections(input)?;

  match &sections[..]
  {
    &[] | &[HANDWRITTEN(_)] => return Ok(None),
    _ => (),
  }

  let mut generated = String::with_capacity(input.len()+4096);
  let mut changed = false;

  for sec in sections.iter()
  {
    match sec
    {
      HANDWRITTEN(code) => generated += code,
      CODEGEN { identifier, code: old_code, checksum: old_checksum, begin, end } =>
      {
        let old_code = begin.indentation.unindent_str(old_code)?;
        check_code_checksum(&old_code, *old_checksum)?;
        let old_checksum =
        {
          let actual_checksum = crc32::hash(old_code.as_bytes());
          changed = changed || *old_checksum == None;
          actual_checksum
        };

        write!(&mut generated, "{i}{before}<< codegen {ident} >>{after}\n", i=begin.indentation, before=begin.before_marker, after=begin.after_marker, ident=identifier)?;
        
        let new_code = f(identifier)?.map(ensure_tailing_linebreak).unwrap_or(old_code);
        let new_checksum = crc32::hash(new_code.as_bytes());
        generated += begin.indentation.indent_str(new_code.as_str()).as_str();

        write!(&mut generated, "{i}{before}<< /codegen ", i=begin.indentation, before=end.before_marker)?;
        crc32::fmt_write(&mut generated, new_checksum);
        write!(&mut generated, " >>{after}\n", after=end.after_marker)?;

        changed = changed || new_checksum != old_checksum;
      }
    }
  }

  return if changed {Ok(Some(generated))} else {Ok(None)};
}

fn check_code_checksum(code: &str, loaded_checksam: Option<crc32::Hash>) -> Result<crc32::Hash>
{
  let actual = crc32::hash(code.as_bytes());
  if let Some(loaded) = loaded_checksam
  {
    if actual != loaded
    {
      return Err(Gen_Error::WRONG_CHECKSUM{actual, loaded});
    }
  }

  return Ok(actual);
}

pub type Result<T=(), E=Gen_Error> = std::result::Result<T, E>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Gen_Error
{
  #[error("{0}")]
  FIND(#[from] crate::parse_file::Parse_Error),
  #[error("fmt error: {0}")]
  FMT(#[from] std::fmt::Error),
  #[error("wrong crc32 checksum. Was the code modified in between?\nLoaded crc32 checksum: 0x{loaded:08x}\nActual crc32 checksum: 0x{actual:08x?}")]
  WRONG_CHECKSUM{actual: crc32::Hash, loaded: crc32::Hash},
  #[error("The code generating function modified code outside the code section")]
  FORBIDDEN,
  #[error("The old code has a smaller indentation than the marker")]
  UNINDENT_ERROR(#[from] crate::indentation::Unindent_Error),
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn test_trivial()
  {
    assert_eq!(generate("", |_| Ok(Some("abc".to_owned()))).unwrap(), None);
    assert_eq!(generate("xyz", |_| Ok(Some("abc".to_owned()))).unwrap(), None);
  }

  #[test]
  fn test_simple_replace()
  {
    assert_eq!(generate("<< codegen foo >>\nxyz\n<< /codegen e1ea7cd2 >>", |_| Ok(Some("xyz".to_owned())) ).unwrap(), None);
    assert_eq!(generate("<< codegen foo >>\nxyz\n<< /codegen >>", |_| Ok(Some("uvw".to_owned())) ).unwrap(), Some("<< codegen foo >>\nuvw\n<< /codegen ad729d7f >>\n".to_owned()));
    assert_eq!(generate("<< codegen foo >>\nremove me\n<< /codegen >>", |_| Ok(Some("".to_owned()))).unwrap(), Some("<< codegen foo >>\n<< /codegen 00000000 >>\n".to_owned()));
    assert_eq!(generate("abc\ndefg<< codegen foo >>hijk\nxyz\nlmnop<< /codegen >>qrst\nuvw", |_| Ok(Some("uvw".to_owned())) ).unwrap(), Some("abc\ndefg<< codegen foo >>hijk\nuvw\nlmnop<< /codegen ad729d7f >>qrst\nuvw".to_owned()));
  }

  #[test]
  fn test_use_identifier()
  {
    assert_eq!(generate("<< codegen answer >>\n<< /codegen >>\n<< codegen finestructure_constant >>\n<< /codegen >>",
      |i|
      {
        let code = match i
        {
        "answer" => "42",
        "finestructure_constant" => "137",
        _ => unreachable!("{i}"),
        };
        Ok(Some(code.to_owned()))
      }).unwrap(), Some("<< codegen answer >>\n42\n<< /codegen d1862931 >>\n<< codegen finestructure_constant >>\n137\n<< /codegen 3f2c523b >>\n".to_owned()));
  }
  
  #[test]
  fn test_check_checksum()
  {
    assert_eq!(check_code_checksum("42", None), Ok(crc32::hash(b"42")));
    assert_eq!(check_code_checksum("42", Some(crc32::hash(b"42"))), Ok(crc32::hash(b"42")));
  }

  #[test]
  fn test_checksum()
  {
    fn gen(n: &str) -> Fmt_Result
    {
      let x = match n
      {
        "empty" => "",
        "42" => "42",
        "newline" => "\n",
        "42_newline" => "42\n",
        n => unimplemented!("{n}"),
      };
      Ok(Some(x.to_owned()))
    }

    // differenet lengths
    assert_eq!(generate("<< codegen empty >>\n<< /codegen >>", gen).unwrap(), Some("<< codegen empty >>\n<< /codegen 00000000 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen 00 >>", gen), Err(Gen_Error::FIND(parse_file::Parse_Error::SYNTAX(parse_file::Syntax_Error::CHECKSUM_WRONG_LENGTH))));
    
    // replace content
    assert_eq!(generate("<< codegen 42 >>\n<< /codegen 00000000>>", gen).unwrap(), Some("<< codegen 42 >>\n42\n<< /codegen d1862931 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n42\n<< /codegen d1862931 >>", gen).unwrap(), Some("<< codegen empty >>\n<< /codegen 00000000 >>\n".to_owned()));
    
    // newline handling
    assert_eq!(generate("<< codegen 42_newline >>\n42\n<< /codegen d1862931>>", gen).unwrap(), None);
    assert_eq!(generate("<< codegen newline >>\n<< /codegen 00000000>>", gen).unwrap(), Some("<< codegen newline >>\n\n<< /codegen 32d70693 >>\n".to_owned()));

    // bug: dirty flag overwritten:
    assert_eq!(generate("<< codegen empty >>\n<< /codegen 00000000 >>\n<< codegen empty >>\n<< /codegen >>", gen).unwrap(), Some("<< codegen empty >>\n<< /codegen 00000000 >>\n<< codegen empty >>\n<< /codegen 00000000 >>\n".to_owned()));
  }
  
  #[test]
  fn test_indentation()
  {
    fn gen(n: &str) -> Fmt_Result
    {
      let x = match n
      {
        "x" => "42\n137\n1337",
        n => unimplemented!("{n}"),
      };
      Ok(Some(x.into()))
    }

    assert_eq!(generate("<< codegen x >>\n<< /codegen >>", gen).unwrap(), Some("<< codegen x >>\n42\n137\n1337\n<< /codegen dda9452f >>\n".to_owned()));
    assert_eq!(generate("  << codegen x >>\n<< /codegen >>", gen).unwrap(), Some("  << codegen x >>\n  42\n  137\n  1337\n  << /codegen dda9452f >>\n".to_owned()));
  }
  
  #[test]
  fn test_hash_and_indentation()
  {
    fn gen(n: &str) -> Fmt_Result
    {
      let x = match n
      {
        "x" => "42\n  137\n1337",
        n => unimplemented!("{n}"),
      };
      Ok(Some(x.into()))
    }

    assert_eq!(generate("<< codegen x >>\n<< /codegen >>", gen).unwrap(), Some("<< codegen x >>\n42\n  137\n1337\n<< /codegen f1656245 >>\n".to_owned()));
    assert_eq!(generate("  << codegen x >>\n<< /codegen >>", gen).unwrap(), Some("  << codegen x >>\n  42\n    137\n  1337\n  << /codegen f1656245 >>\n".to_owned()));
  }
  
  #[test]
  fn allow_skipping_sections()
  {
    fn ignore(_n: &str) -> Fmt_Result
    {
      Ok(None)
    }

    assert_eq!(generate("<< codegen x >>\nxyuz\nuv\n<< /codegen 2fddd919 >>", ignore).unwrap(), None);
    assert_eq!(generate("<< codegen x >>\nxyuz\nuv\n<< /codegen 2fddd919>>", ignore).unwrap(), None);
    assert_eq!(generate("<< codegen x >>\nxyuz\nuv\n<< /codegen >>", ignore).unwrap(), Some("<< codegen x >>\nxyuz\nuv\n<< /codegen 2fddd919 >>\n".to_string()));
    assert_eq!(generate("  << codegen x >>\n  xyuz\n  <>\n    []\n  uv\n<< /codegen c1e2b1d0 >>", ignore).unwrap(), None);
    assert_eq!(generate("  << codegen x >>\n  xyuz\n  <>\n    []\n  uv\n<< /codegen >>", ignore).unwrap(), Some("  << codegen x >>\n  xyuz\n  <>\n    []\n  uv\n  << /codegen c1e2b1d0 >>\n".to_string()));
  }
}

use std::fmt;
use fmt::Write;
