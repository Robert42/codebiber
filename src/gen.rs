use super::*;

use super::parse_file::{find as parse_sections, Section};

#[derive(Default)]
pub struct Config
{
  pub checksum_bytes_to_store: u8,
}

pub fn generate<F>(input: &str, cfg: Config, f: F) -> Result<Option<String>>
where F: Fn(&str, &mut String) -> std::fmt::Result<>
{
  debug_assert!(cfg.is_valid());

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
        check_code_checksum(old_code, old_checksum)?;
        let old_checksum =
        {
          let actual_checksum = blake3::hash(old_code.as_bytes());
          changed = changed || cfg.checksum_bytes_to_store as usize != old_checksum.len();
          actual_checksum
        };

        write!(&mut generated, "{i}{before}<< codegen {ident} >>{after}\n", i=begin.indentation, before=begin.before_marker, after=begin.after_marker, ident=identifier)?;
        
        let generated_begin = generated.len();
        f(identifier, &mut generated)?;
        if generated.as_bytes().last().copied() != Some(b'\n')
        {
          generated += "\n";
        }
        begin.indentation.indent_subrange(&mut generated, generated_begin..);
        let new_code = &generated[generated_begin..];

        let new_checksum = blake3::hash(new_code.as_bytes());

        write!(&mut generated, "{i}{before}<< /codegen ", i=begin.indentation, before=end.before_marker)?;
        if cfg.checksum_bytes_to_store > 0
        {
          write!(&mut generated, "{checksum} ", checksum=&new_checksum.to_hex()[0..2*cfg.checksum_bytes_to_store as usize])?;
        }
        write!(&mut generated, ">>{after}\n", after=end.after_marker)?;

        changed = changed || new_checksum != old_checksum;
      }
    }
  }

  return if changed {Ok(Some(generated))} else {Ok(None)};
}

fn check_code_checksum(code: &str, loaded_checksam: &ArrayVec<u8, 32>) -> Result<blake3::Hash>
{
  let actual_hashsum = blake3::hash(code.as_bytes());
  if &actual_hashsum.as_bytes()[..loaded_checksam.len()] != loaded_checksam.as_slice()
  {
    return Err(Error::WRONG_CHECKSUM(actual_hashsum));
  }

  return Ok(actual_hashsum);
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error
{
  #[error("{0}")]
  FIND(#[from] crate::parse_file::Error),
  #[error("fmt error: {0}")]
  FMT(#[from] std::fmt::Error),
  #[error("wrong blake3 checksum. Was the code modified in between?\nActual blake3 checksum: {0}")]
  WRONG_CHECKSUM(blake3::Hash),
}

impl Config
{
  pub fn is_valid(&self) -> bool
  {
    self.checksum_bytes_to_store <= 32
  }
}

#[cfg(test)]
mod test
{
  use super::*;

  const CFG : Config = Config{checksum_bytes_to_store: 0};

  #[test]
  fn test_trivial()
  {
    assert_eq!(generate("", CFG, |_,_| Ok(())).pretty_unwrap(), None);
    assert_eq!(generate("xyz", CFG, |_,_| Ok(())).pretty_unwrap(), None);
  }

  #[test]
  fn test_simple_replace()
  {
    assert_eq!(generate("<< codegen foo >>\nxyz\n<< /codegen >>", CFG, |_,x| write!(x, "xyz")).pretty_unwrap(), None);
    assert_eq!(generate("<< codegen foo >>\nxyz\n<< /codegen >>", CFG, |_,x| write!(x, "uvw")).pretty_unwrap(), Some("<< codegen foo >>\nuvw\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("<< codegen foo >>\nremove me\n<< /codegen >>", CFG, |_,_| Ok(())).pretty_unwrap(), Some("<< codegen foo >>\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("abc\ndefg<< codegen foo >>hijk\nxyz\nlmnop<< /codegen >>qrst\nuvw", CFG, |_,x| write!(x, "uvw")).pretty_unwrap(), Some("abc\ndefg<< codegen foo >>hijk\nuvw\nlmnop<< /codegen >>qrst\nuvw".to_owned()));
  }

  #[test]
  fn test_use_identifier()
  {
    assert_eq!(generate("<< codegen answer >>\n<< /codegen >>\n<< codegen finestructure_constant >>\n<< /codegen >>", CFG,
      |i,x| match i {
        "answer" => write!(x, "42"),
        "finestructure_constant" => write!(x, "137"),
        _ => unreachable!("{i}"),
      }).pretty_unwrap(), Some("<< codegen answer >>\n42\n<< /codegen >>\n<< codegen finestructure_constant >>\n137\n<< /codegen >>\n".to_owned()));
  }
  
  #[test]
  fn test_check_checksum()
  {
    assert_eq!(check_code_checksum("42", &ArrayVec::new()), Ok(blake3::hash(b"42")));
    assert_eq!(check_code_checksum("42", &blake3::hash(b"42").as_bytes().iter().copied().collect()), Ok(blake3::hash(b"42")));
    assert_eq!(check_code_checksum("42", &blake3::hash(b"42").as_bytes()[0..4].iter().copied().collect()), Ok(blake3::hash(b"42")));
    assert_eq!(check_code_checksum("42", &blake3::hash(b"42").as_bytes()[1..5].iter().copied().collect()), Err(Error::WRONG_CHECKSUM(blake3::hash(b"42"))));
  }

  #[test]
  fn test_checksum()
  {
    const CKSM_0 : Config = Config{checksum_bytes_to_store: 0, .. CFG};
    const CKSM_2 : Config = Config{checksum_bytes_to_store: 2, .. CFG};
    const CKSM_4 : Config = Config{checksum_bytes_to_store: 4, .. CFG};
    const CKSM_5 : Config = Config{checksum_bytes_to_store: 5, .. CFG};

    fn gen(n: &str, out: &mut String) -> fmt::Result
    {
      match n
      {
        "empty" => Ok(()),
        "42" => write!(out, "42"),
        "newline" => write!(out, "\n"),
        "42_newline" => write!(out, "42\n"),
        n => todo!("{n}"),
      }
    }

    // differenet lengths
    assert_eq!(generate("<< codegen empty >>\n<< /codegen >>", CKSM_0, gen).pretty_unwrap(), None);
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13 >>", CKSM_0, gen).pretty_unwrap(), Some("<< codegen empty >>\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13 >>", CKSM_2, gen).pretty_unwrap(), None);
    assert_eq!(generate("<< codegen empty >>\n<< /codegen >>", CKSM_2, gen).pretty_unwrap(), Some("<< codegen empty >>\n<< /codegen af13 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13>>", CKSM_4, gen).pretty_unwrap(), Some("<< codegen empty >>\n<< /codegen af1349b9 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13>>", CKSM_5, gen).pretty_unwrap(), Some("<< codegen empty >>\n<< /codegen af1349b9f5 >>\n".to_owned()));
    
    // replace content
    assert_eq!(generate("<< codegen 42 >>\n<< /codegen af1349b9f5>>", CKSM_5, gen).pretty_unwrap(), Some("<< codegen 42 >>\n42\n<< /codegen a16072b1b0 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n42\n<< /codegen a16072b1b0>>", CKSM_5, gen).pretty_unwrap(), Some("<< codegen empty >>\n<< /codegen af1349b9f5 >>\n".to_owned()));
    
    // newline handling
    assert_eq!(generate("<< codegen 42_newline >>\n42\n<< /codegen a16072b1>>", CKSM_5, gen).pretty_unwrap(), Some("<< codegen 42_newline >>\n42\n<< /codegen a16072b1b0 >>\n".to_owned()));
    assert_eq!(generate("<< codegen newline >>\n<< /codegen af1349b9f5>>", CKSM_5, gen).pretty_unwrap(), Some("<< codegen newline >>\n\n<< /codegen 295192ea1e >>\n".to_owned()));

    // bug: dirty flag overwritten:
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13 >>\n<< codegen empty >>\n<< /codegen >>", CKSM_0, gen).pretty_unwrap(), Some("<< codegen empty >>\n<< /codegen >>\n<< codegen empty >>\n<< /codegen >>\n".to_owned()));
  }
  
  #[test]
  fn test_indentation()
  {
    fn gen(n: &str, out: &mut String) -> fmt::Result
    {
      match n
      {
        "x" => write!(out, "42\n137\n1337"),
        n => todo!("{n}"),
      }
    }

    assert_eq!(generate("<< codegen x >>\n<< /codegen >>", CFG, gen).pretty_unwrap(), Some("<< codegen x >>\n42\n137\n1337\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("  << codegen x >>\n<< /codegen >>", CFG, gen).pretty_unwrap(), Some("  << codegen x >>\n  42\n  137\n  1337\n  << /codegen >>\n".to_owned()));
  }
}

use std::fmt;
use fmt::Write;