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
      let mut indenter = Line_Indenter::new(&mut bytes, rng.start, self.0);

      for i in (0..indenter.src_cursor).rev()
      {
        if indenter.bytes[i] == b'\n'
        {
          indenter.copy_content(i+1);
          indenter.fill_indentation();
        }
      }

      indenter.copy_content(0);
      indenter.fill_indentation();
      debug_assert_eq!(indenter.dst_cursor, 0, "{:?}", std::str::from_utf8(indenter.bytes).unwrap());
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
  dst_cursor: usize,
  src_cursor: usize,
  indentation: usize,
}

impl<'a> Line_Indenter<'a>
{
  fn new(bytes: &'a mut Vec<u8>, start: usize, indentation: usize) -> Self
  {
    let num_linebreaks = bytes[start..].iter().copied().filter(|&x| x==b'\n').count();
    let num_indentations = num_linebreaks + 1;

    let src_cursor = bytes.len();
    let dst_cursor = src_cursor + num_indentations * indentation;
    bytes.resize(dst_cursor, b'#');

    Line_Indenter{
      bytes: &mut bytes[start..],
      src_cursor,
      dst_cursor,
      indentation,
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
}

use std::fmt::{self, Write};
use std::ops::{Range, RangeFrom};