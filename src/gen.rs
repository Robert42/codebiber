use super::*;

use super::parse_file::{find as parse_sections, Section};

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Config
{
  pub checksum_bytes_to_store: u8,
}

pub type Fmt_Result<T=Option<String>> = std::result::Result<T, std::fmt::Error>;

pub fn generate<F>(input: &str, cfg: Config, mut f: F) -> Result<Option<String>>
where F: FnMut(&str) -> Fmt_Result
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
        
        let _tmp_buffer;
        let new_code = match f(identifier)?
        {
          Some(generated_code) => 
          {
            _tmp_buffer = begin.indentation.indent_str(generated_code.as_str());
            _tmp_buffer.as_str()
          }
          None => old_code
        };
        let new_checksum = blake3::hash(new_code.as_bytes());
        generated += new_code;

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
    return Err(Gen_Error::WRONG_CHECKSUM(actual_hashsum));
  }

  return Ok(actual_hashsum);
}

pub type Result<T=(), E=Gen_Error> = std::result::Result<T, E>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Gen_Error
{
  #[error("{0}")]
  FIND(#[from] crate::parse_file::Parse_Error),
  #[error("fmt error: {0}")]
  FMT(#[from] std::fmt::Error),
  #[error("wrong blake3 checksum. Was the code modified in between?\nActual blake3 checksum: {0}")]
  WRONG_CHECKSUM(blake3::Hash),
  #[error("The code generating function modified code outside the code section")]
  FORBIDDEN,
}

impl Config
{
  pub fn is_valid(&self) -> bool
  {
    self.checksum_bytes_to_store <= blake3::KEY_LEN as u8
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
    assert_eq!(generate("", CFG, |_| Ok(Some("abc".to_owned()))).unwrap_display(), None);
    assert_eq!(generate("xyz", CFG, |_| Ok(Some("abc".to_owned()))).unwrap_display(), None);
  }

  #[test]
  fn test_simple_replace()
  {
    assert_eq!(generate("<< codegen foo >>\nxyz\n<< /codegen >>", CFG, |_| Ok(Some("xyz".to_owned())) ).unwrap_display(), None);
    assert_eq!(generate("<< codegen foo >>\nxyz\n<< /codegen >>", CFG, |_| Ok(Some("uvw".to_owned())) ).unwrap_display(), Some("<< codegen foo >>\nuvw\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("<< codegen foo >>\nremove me\n<< /codegen >>", CFG, |_| Ok(Some("".to_owned()))).unwrap_display(), Some("<< codegen foo >>\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("abc\ndefg<< codegen foo >>hijk\nxyz\nlmnop<< /codegen >>qrst\nuvw", CFG, |_| Ok(Some("uvw".to_owned())) ).unwrap_display(), Some("abc\ndefg<< codegen foo >>hijk\nuvw\nlmnop<< /codegen >>qrst\nuvw".to_owned()));
  }

  #[test]
  fn test_use_identifier()
  {
    assert_eq!(generate("<< codegen answer >>\n<< /codegen >>\n<< codegen finestructure_constant >>\n<< /codegen >>", CFG,
      |i|
      {
        let code = match i
        {
        "answer" => "42",
        "finestructure_constant" => "137",
        _ => unreachable!("{i}"),
        };
        Ok(Some(code.to_owned()))
      }).unwrap_display(), Some("<< codegen answer >>\n42\n<< /codegen >>\n<< codegen finestructure_constant >>\n137\n<< /codegen >>\n".to_owned()));
  }
  
  #[test]
  fn test_check_checksum()
  {
    assert_eq!(check_code_checksum("42", &ArrayVec::new()), Ok(blake3::hash(b"42")));
    assert_eq!(check_code_checksum("42", &blake3::hash(b"42").as_bytes().iter().copied().collect()), Ok(blake3::hash(b"42")));
    assert_eq!(check_code_checksum("42", &blake3::hash(b"42").as_bytes()[0..4].iter().copied().collect()), Ok(blake3::hash(b"42")));
    assert_eq!(check_code_checksum("42", &blake3::hash(b"42").as_bytes()[1..5].iter().copied().collect()), Err(Gen_Error::WRONG_CHECKSUM(blake3::hash(b"42"))));
  }

  #[test]
  fn test_checksum()
  {
    const CKSM_0 : Config = Config{checksum_bytes_to_store: 0, .. CFG};
    const CKSM_2 : Config = Config{checksum_bytes_to_store: 2, .. CFG};
    const CKSM_4 : Config = Config{checksum_bytes_to_store: 4, .. CFG};
    const CKSM_5 : Config = Config{checksum_bytes_to_store: 5, .. CFG};
    const CKSM_MAX : Config = Config{checksum_bytes_to_store: blake3::KEY_LEN as u8, .. CFG};

    fn gen(n: &str) -> Fmt_Result
    {
      let x = match n
      {
        "empty" => "",
        "42" => "42",
        "newline" => "\n",
        "42_newline" => "42\n",
        n => todo!("{n}"),
      };
      Ok(Some(x.to_owned()))
    }

    // differenet lengths
    assert_eq!(generate("<< codegen empty >>\n<< /codegen >>", CKSM_0, gen).unwrap_display(), None);
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13 >>", CKSM_0, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13 >>", CKSM_2, gen).unwrap_display(), None);
    assert_eq!(generate("<< codegen empty >>\n<< /codegen >>", CKSM_2, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen af13 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13>>", CKSM_4, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen af1349b9 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13>>", CKSM_5, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen af1349b9f5 >>\n".to_owned()));
    
    // replace content
    assert_eq!(generate("<< codegen 42 >>\n<< /codegen af1349b9f5>>", CKSM_5, gen).unwrap_display(), Some("<< codegen 42 >>\n42\n<< /codegen a16072b1b0 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n42\n<< /codegen a16072b1b0>>", CKSM_5, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen af1349b9f5 >>\n".to_owned()));
    
    // newline handling
    assert_eq!(generate("<< codegen 42_newline >>\n42\n<< /codegen a16072b1>>", CKSM_5, gen).unwrap_display(), Some("<< codegen 42_newline >>\n42\n<< /codegen a16072b1b0 >>\n".to_owned()));
    assert_eq!(generate("<< codegen newline >>\n<< /codegen af1349b9f5>>", CKSM_5, gen).unwrap_display(), Some("<< codegen newline >>\n\n<< /codegen 295192ea1e >>\n".to_owned()));

    // bug: dirty flag overwritten:
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13 >>\n<< codegen empty >>\n<< /codegen >>", CKSM_0, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen >>\n<< codegen empty >>\n<< /codegen >>\n".to_owned()));

    // max length
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af13>>", CKSM_MAX, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262 >>\n".to_owned()));
    assert_eq!(generate("<< codegen empty >>\n<< /codegen af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262>>", CKSM_2, gen).unwrap_display(), Some("<< codegen empty >>\n<< /codegen af13 >>\n".to_owned()));
  }
  
  #[test]
  fn test_indentation()
  {
    fn gen(n: &str) -> Fmt_Result
    {
      let x = match n
      {
        "x" => "42\n137\n1337",
        n => todo!("{n}"),
      };
      Ok(Some(x.into()))
    }

    assert_eq!(generate("<< codegen x >>\n<< /codegen >>", CFG, gen).unwrap_display(), Some("<< codegen x >>\n42\n137\n1337\n<< /codegen >>\n".to_owned()));
    assert_eq!(generate("  << codegen x >>\n<< /codegen >>", CFG, gen).unwrap_display(), Some("  << codegen x >>\n  42\n  137\n  1337\n  << /codegen >>\n".to_owned()));
  }
  
  #[test]
  fn allow_skipping_sections()
  {
    fn ignore(_n: &str) -> Fmt_Result
    {
      Ok(None)
    }

    assert_eq!(generate("<< codegen x >>\nxyuz\nuv\n<< /codegen >>", CFG, ignore).unwrap_display(), None);
    assert_eq!(generate("  << codegen x >>\nxyuz\n  <>\n    []\nuv\n<< /codegen >>", CFG, ignore).unwrap_display(), None);
  }
}

use std::fmt;
use fmt::Write;