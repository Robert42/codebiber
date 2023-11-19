use super::*;

pub fn generate<F>(input: &str, f: F) -> Result<Option<String>>
where F: Fn(&str, &mut String)
{
  Ok(None)
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum Error
{
  #[error("{0}")]
  FIND(#[from] crate::find::Error),
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn test_trivial()
  {
    assert_eq!(generate("", |_,_| ()).pretty_unwrap(), None);
  }
}