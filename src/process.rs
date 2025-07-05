use super::*;

pub fn process_file<P, F>(path: P, f: &F) -> Result
where F: Fn(&str) -> Fmt_Result,
      P: AsRef<Path>,
{
  let path = path.as_ref();

  let input = std::fs::read_to_string(path)?;

  if let Some(generated) = gen::generate(&input, f)?
  {
    std::fs::write(path, generated)?;
  }

  Ok(())
}

pub fn process_files<P, F>(paths: &[P], f: F) -> Result
where F: Fn(&str) -> Fmt_Result,
      P: AsRef<Path>,
{
  for path in paths
  {
    process_file(path, &f)?;
  }

  Ok(())
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum Process_Error
{
  IO(std::io::Error),
  GEN(gen::Gen_Error),
}

impl fmt::Display for Process_Error
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    use Process_Error::*;
    match self
    {
      IO(e) => write!(f, "{e}"),
      GEN(e) => write!(f, "Generation error: {e}"),
    }
  }
}

impl From<std::io::Error> for Process_Error {fn from(e: std::io::Error) -> Self {Process_Error::IO(e)}}
impl From<gen::Gen_Error> for Process_Error {fn from(e: gen::Gen_Error) -> Self {Process_Error::GEN(e)}}

use std::path::Path;
use std::fmt;
