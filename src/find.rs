#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Section<'a>
{
  HAND_WRITTEN(&'a str)
}

pub struct Finder
{
}

impl Finder
{
  pub fn new() -> Self
  {
    Finder{
    }
  }

  pub fn find(&mut self, code: &str) -> Result<&[Section], Error>
  {
    Ok(&[])
  }
}

pub type Result<T=(), E=Error> = std::result::Result<T, E>;

#[derive(Clone, Debug)]
pub enum Error
{
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn trivial() -> Result
  {
    let mut finder = Finder::new();
    assert_eq!(finder.find("")?, &[]);

    Ok(())
  }
}