use super::*;

use super::find::{find as parse_sections, Section};

#[derive(Default)]
pub struct Config
{
  pub store_checksum: bool,
}

pub fn generate<F>(input: &str, cfg: Config, f: F) -> Result<Option<String>>
where F: Fn(&str, &mut String) -> std::fmt::Result<>
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

  for &sec in sections.iter()
  {
    match sec
    {
      HANDWRITTEN(code) => generated += code,
      CODEGEN { identifier, code: old_code, checksum: old_checksum, begin, end } =>
      {
        let old_checksum = if let Some(old_checksum) = old_checksum
        {
          if blake3::hash(old_code.as_bytes()) != old_checksum {todo!()}
          old_checksum
        }else
        {
          changed = cfg.store_checksum;
          blake3::hash(old_code.as_bytes())
        };

        write!(&mut generated, "{i}{before}<< codegen {ident} >>{after}\n", i=begin.indentation, before=begin.before_marker, after=begin.after_marker, ident=identifier)?;
        
        let generated_begin = generated.len();
        f("TODO: pass identifier", &mut generated)?;
        if generated.as_bytes().last().copied() != Some(b'\n')
        {
          generated += "\n";
        }
        let new_code = &generated[generated_begin..];

        let new_checksum = blake3::hash(new_code.as_bytes());

        match cfg.store_checksum
        {
          true  => write!(&mut generated, "{i}{before}<< /codegen {checksum} >>{after}\n", i=end.indentation, before=end.before_marker, after=end.after_marker, checksum=new_checksum)?,
          false => write!(&mut generated, "{i}{before}<< /codegen >>{after}\n",            i=end.indentation, before=end.before_marker, after=end.after_marker)?,
        }

        changed = changed || new_checksum != old_checksum;
      }
    }
  }

  return if changed {Ok(Some(generated))} else {Ok(None)};
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error
{
  #[error("{0}")]
  FIND(#[from] crate::find::Error),
  #[error("fmt error: {0}")]
  FMT(#[from] std::fmt::Error),
}

#[cfg(test)]
mod test
{
  use super::*;

  const CFG : Config = Config{store_checksum: false};

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
}

use std::fmt::Write;