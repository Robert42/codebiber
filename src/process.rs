use super::*;

pub fn process_file<P, F>(path: P, f: &F) -> Result
where F: Fn(&str) -> Fmt_Result,
      P: AsRef<Path>,
{
  let path = path.as_ref();

  let add_path_to_err = |e: gen::Gen_Error| -> gen::Gen_Error
  {
    use gen::Gen_Error::PARSE;
    use parse_file::{Parse_Error::SYNTAX, Error_Location};
    match e
    {
      PARSE(SYNTAX(Error_Location{path: None, line}, e)) => PARSE(SYNTAX(Error_Location{path: Some(path.to_owned()), line}, e)),
      other => other,
    }
  };

  let input = std::fs::read_to_string(path)?;

  if let Some(generated) = gen::generate(&input, f ).map_err(add_path_to_err)?
  {
    std::fs::write(path, generated)?;
  }

  Ok(())
}

pub fn process_files<P, F>(paths: &[P], f: F) -> Result
where F: Fn(&Path, &str) -> Fmt_Result,
      P: AsRef<Path>,
{
  for path in paths
  {
    let path = path.as_ref();

    process_file(path, &|name: &str| f(path, name))?;
  }

  Ok(())
}

pub fn glob_by_suffix<P: AsRef<Path>>(dir: P, suffixes: &[&str]) -> Result<Vec<PathBuf>, std::io::Error>
{
  fn is_extension(p: &Path, s: &str) -> bool
  {
    return p.extension().map(|e| e == s).unwrap_or(false)
  }
  let is_any_expected_extension = |p: &Path|
  {
    return suffixes.into_iter().any(|s| is_extension(p, s))
  };
  return glob(dir, &is_any_expected_extension);
}

/*
#[test]
fn glob_sample()
{
  let mut s = String::new();
  let files = glob_by_suffix(".", &["rs"]).unwrap();
  for p in files.iter()
  {
    s.push_str(&format!("{}\n", p.display()));
  }
  panic!("== {} ==\n{}", files.len(), s.as_str())
}
*/

pub fn glob<P: AsRef<Path>, Pred>(dir: P, pred: &Pred) -> Result<Vec<PathBuf>, std::io::Error>
  where Pred: Fn(&Path)->bool
{
  let dir = dir.as_ref();
  let mut xs = Vec::new();
  _visit_dir_glob(&mut xs, dir, pred)?;
  xs.sort();
  return Ok(xs);
}

fn _visit_dir_glob<P>(out: &mut Vec<PathBuf>, dir: &Path, pred: &P) -> std::io::Result<()>
  where P: Fn(&Path)->bool
{
  if dir.is_dir()
  {
    for f in fs::read_dir(dir)?
    {
      let f = f?.path();
      let path = f.as_path();
      if path.is_dir()
      {
        _visit_dir_glob(out, path, pred)?;
      }else if path.is_file()
      {
        if pred(&path)
        {
          out.push(path.canonicalize()?);
        }
      }
    }
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

use std::path::{Path, PathBuf};
use std::fmt;
use std::fs;

