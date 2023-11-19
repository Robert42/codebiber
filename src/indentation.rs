#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl fmt::Display for Indentation
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    for _ in 0..self.0
    {
      write!(f, "{}", ' ')?;
    }
    Ok(())
  }
}

use std::fmt;