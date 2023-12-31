use super::*;

pub fn process_file<P, F>(path: P, cfg: Config, f: &F) -> Result
where F: Fn(&str) -> Fmt_Result,
      P: AsRef<Path>,
{
  let path = path.as_ref();

  let input = std::fs::read_to_string(path)?;

  if let Some(generated) = gen::generate(&input, cfg, f)?
  {
    std::fs::write(path, generated)?;
  }

  Ok(())
}

pub fn process_files<P, F>(paths: &[P], cfg: Config, f: F) -> Result
where F: Fn(&str) -> Fmt_Result,
      P: AsRef<Path>,
{
  for path in paths
  {
    process_file(path, cfg, &f)?;
  }

  Ok(())
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Process_Error
{
  #[error("{0}")]
  IO(#[from] std::io::Error),
  #[error("Generation error: {0}")]
  GEN(#[from] gen::Gen_Error),
}

use std::path::Path;