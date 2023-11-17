use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Section
{
  HANDWRITTEN(Range<usize>)
}
use Section::*;

pub struct Finder
{
  sections: Vec<Section>,
}

impl Finder
{
  pub fn new() -> Self
  {
    Finder{
      sections: Vec::with_capacity(256),
    }
  }

  pub fn find(&mut self, code: &str) -> Result<&[Section], Error>
  {
    self.sections.clear();
    
    if code.len() > 0
    {
      self.sections.push(HANDWRITTEN(0..code.len()));
    }
    Ok(&self.sections[..])
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
    assert_eq!(finder.find("xyz")?, &[HANDWRITTEN(0..3)]);

    Ok(())
  }
}
