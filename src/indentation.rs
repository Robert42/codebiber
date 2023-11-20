use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl Indentation
{
  pub fn indent_subrange(self, text: &mut String, rng: std::ops::RangeFrom<usize>)
  {
    if rng.start == text.len() {return}
    // TODO: shrink the rng by one if it ends with a linebreak?

    let mut bytes = std::mem::take(text).into_bytes();

    let num_linebreaks = bytes[rng.clone()].iter().copied().filter(|&x| x==b'\n').count();
    let num_indentations = num_linebreaks + 1;

    let src_cursor = bytes.len();
    let dst_cursor = src_cursor + num_indentations * self.0;
    bytes.resize(dst_cursor, b'#');

    {
      let bytes = &mut bytes[rng];

      bytes.copy_within(..src_cursor, dst_cursor-src_cursor);
      for i in 0..self.0 { bytes[i] = b' '; }
    }

    *text = unsafe{ String::from_utf8_unchecked(bytes) };
  }

  pub fn indent_string(self, text: &mut String)
  {
    self.indent_subrange(text, 0..)
  }

  pub fn indent_str(self, text: &str) -> String
  {
    let mut x = text.to_owned();
    self.indent_string(&mut x);
    x
  }

  pub fn to_small_str(self) -> SmallString::<[u8; 128]>
  {
    let mut xs = SmallString::new();
    xs.reserve(self.0);
    write!(&mut xs, "{}", self).unwrap();
    xs
  }
}

impl fmt::Display for Indentation
{
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
  {
    for _ in 0..self.0
    {
      f.write_char(' ')?;
    }
    Ok(())
  }
}

#[cfg(test)]
mod test
{
  macro_rules! indent {
    ($i:expr, $str:expr) => {
      crate::indentation::Indentation($i).indent_str($str).as_str()
    };
  }

  #[test]
  fn test_trivial()
  {
    assert_eq!(indent!(0, ""), "");
    assert_eq!(indent!(0, "x"), "x");
  }

  #[test]
  fn test_simpl()
  {
    assert_eq!(indent!(2, ""), "");
    assert_eq!(indent!(2, "x"), "  x");
  }
}

use std::fmt::{self, Write};