use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl Indentation
{
  pub fn indent_subrange(self, text: &mut String, rng: RangeFrom<usize>)
  {
    if rng.start == text.len() {return}
    // TODO: shrink the rng by one if it ends with a linebreak?

    let mut bytes = std::mem::take(text).into_bytes();

    {
      let indenter = Line_Indenter::new(&mut bytes, rng.start, self.0);
      indenter.indent();
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
}

struct Line_Indenter<'a>
{
  bytes: &'a mut [u8],
  linebreaks: SmallVec<[usize; 1024]>,
  dst_cursor: usize,
  src_cursor: usize,
  indentation: usize,
}

impl<'a> Line_Indenter<'a>
{
  fn new(bytes: &'a mut Vec<u8>, start: usize, indentation: usize) -> Self
  {
    let mut linebreaks = smallvec![];
    linebreaks.push(0);
    for (i,&x) in bytes[start..].iter().enumerate()
    {
      if x == b'\n' { linebreaks.push(i+1); }
    }

    // prevent trailing whitespace
    {
      let mut i = 0;
      let mut j = 0;
      while j < linebreaks.len()
      {
        if j+1 == linebreaks.len() || linebreaks[j]+1 != linebreaks[j+1]
        {
          linebreaks[i] = linebreaks[j];
          i += 1;
        }
        j += 1;
      }
      linebreaks.truncate(i);
    }
    if linebreaks.last().copied() == Some(bytes.len())
    {
      linebreaks.pop();
    }

    let num_linebreaks = linebreaks.len();

    let src_cursor = bytes.len()-start;
    let dst_cursor = src_cursor + num_linebreaks * indentation;
    bytes.resize(dst_cursor, b'#');

    Line_Indenter{
      bytes: &mut bytes[start..],
      linebreaks,
      src_cursor,
      dst_cursor,
      indentation,
    }
  }

  fn indent(mut self)
  {
    let linebreaks = std::mem::take(&mut self.linebreaks);
    for &i in linebreaks.iter().rev()
    {
      self.copy_content(i);
      self.fill_indentation();
    }
  }

  fn copy_content(&mut self, from: usize)
  {
    let n = self.src_cursor - from;
    let src_rng = from..self.src_cursor;
    self.src_cursor = from;

    self.dst_cursor -= n;
    self.bytes.copy_within(src_rng, self.dst_cursor);
  }

  fn fill_indentation(&mut self)
  {
    self.dst_cursor -= self.indentation;
    fill_with_space(self.bytes, self.dst_cursor..self.dst_cursor+self.indentation);
  }
}

fn fill_with_space(bytes: &mut [u8], rng: Range<usize>)
{
  for i in rng { bytes[i] = b' '; }
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
    assert_eq!(indent!(4, "Hello, World!"), "    Hello, World!");
  }

  #[test]
  fn test_with_lienbreak()
  {
    assert_eq!(indent!(2, "x\ny\nz"), "  x\n  y\n  z");
  }

  #[test]
  fn test_dont_add_trailing_whitespace()
  {
    assert_eq!(indent!(2, "x\n\ny\n\n\nz"), "  x\n\n  y\n\n\n  z");
    assert_eq!(indent!(2, "x\n"), "  x\n");
  }

  #[test]
  fn test_difficult_cases()
  {
    assert_eq!(indent!(2, "\nx"), "\n  x");
  }
}

use std::fmt::{self, Write};
use std::ops::{Range, RangeFrom};