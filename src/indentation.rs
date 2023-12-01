use super::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl Indentation
{
  pub fn indent_string(self, text: &mut String)
  {
    let mut bytes = std::mem::take(text).into_bytes();

    indent_lines(&mut bytes, 0, self.0);

    *text = unsafe{ String::from_utf8_unchecked(bytes) };
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

fn indent_lines(bytes_buf: &mut Vec<u8>, start: usize, indentation: usize)
{
  if indentation == 0 || start == bytes_buf.len() {return;} // microoptimization

  assert!(bytes_buf.len() < u32::MAX as usize);

  let bytes = &bytes_buf[start..];
  let mut linebreaks : SmallVec<[u32; 32768]> = smallvec![];
  linebreaks.push(0);
  for (i,&x) in bytes.iter().enumerate()
  {
    if x == b'\n' { linebreaks.push(i as u32+1); }
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
  if linebreaks.last().copied() == Some(bytes.len() as u32)
  {
    linebreaks.pop();
  }

  let num_linebreaks = linebreaks.len();

  let src_cursor = bytes.len();
  let dst_cursor = src_cursor + num_linebreaks * indentation;
  bytes_buf.resize(start + dst_cursor, b'#');

  let mut indenter = Line_Indenter{
    bytes: &mut bytes_buf[start..],
    src_cursor,
    dst_cursor,
    indentation,
  };

  for &i in linebreaks.iter().rev()
  {
    indenter.copy_content(i as usize);
    indenter.fill_indentation();
  }
}

#[cfg(test)]
fn indent_lines_simpl_impl(input: &[u8], start: usize, indentation: usize) -> Vec<u8>
{
  let mut output = Vec::with_capacity(start + (input.len()-start+1)*(indentation+1));
  output.extend_from_slice(&input[..start]);

  let indentation = std::iter::repeat(b' ').take(indentation);
  for (i, line) in input[start..].split(|&x| x == b'\n').enumerate()
  {
    if i != 0 { output.push(b'\n'); }
    if line.is_empty() {continue}
    output.extend(indentation.clone());
    output.extend_from_slice(line);
  }

  output
}

impl<'a> Line_Indenter<'a>
{
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
  use super::indent_lines_simpl_impl;
  use crate::proptest::prelude::*;

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

  proptest!
  {
    #[test]
    fn prop_same_behavior_as_simple_impl(text in "(\\PC*\\n)*\\PC*", indentation in 0..=255_usize, start in 0.0..=1.0)
    {
      let start : f64 = start * (text.len() as f64 - 1.0);
      let start = start.round() as usize;
      let start = start.max(0).min(if text.len()>0 {text.len()-1} else {0});

      let expected = indent_lines_simpl_impl(text.as_bytes(), start, indentation);

      let mut text = text.into_bytes();
      super::indent_lines(&mut text, start, indentation);
      let actual = text;

      assert_eq!(expected, actual);
    }
  }

  #[test]
  fn test_indent_lines_simpl_impl()
  {
    fn f(code: &str, start: usize, indentation: usize) -> String
    {
      let x = indent_lines_simpl_impl(code.as_bytes(), start, indentation);
      let x : String = String::from_utf8(x).unwrap();
      x
    }
    assert_eq!(f("", 0, 0), "");
    assert_eq!(f("", 0, 1), "");
    assert_eq!(f("x", 0, 0), "x");
    assert_eq!(f("x\ny", 0, 0), "x\ny");
    assert_eq!(f("x\ny", 0, 2), "  x\n  y");
    assert_eq!(f("x\n\n\ny", 0, 2), "  x\n\n\n  y");
  }
}

use std::fmt::{self, Write};
use std::ops::{Range, RangeFrom};