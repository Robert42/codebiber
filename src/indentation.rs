#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl Indentation
{
  pub fn indent_string(self, text: &mut String)
  {
    let bytes = indent_lines(text.as_bytes(), self.0);

    *text = unsafe{ String::from_utf8_unchecked(bytes) };
  }

  pub fn indent_str(self, text: &str) -> String
  {
    let mut x = text.to_owned();
    self.indent_string(&mut x);
    x
  }
}

fn indent_lines(input: &[u8], indentation: usize) -> Vec<u8>
{
  let mut output = Vec::with_capacity((input.len()+1)*(indentation+1));

  let indentation = std::iter::repeat(b' ').take(indentation);
  for (i, line) in input.split(|&x| x == b'\n').enumerate()
  {
    if i != 0 { output.push(b'\n'); }
    if line.is_empty() {continue}
    output.extend(indentation.clone());
    output.extend_from_slice(line);
  }

  output
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

  #[test]
  fn test_indent_lines_simpl_impl()
  {
    fn f(code: &str, indentation: usize) -> String
    {
      let x = super::indent_lines(code.as_bytes(), indentation);
      let x : String = String::from_utf8(x).unwrap();
      x
    }
    assert_eq!(f("", 0), "");
    assert_eq!(f("", 1), "");
    assert_eq!(f("x", 0), "x");
    assert_eq!(f("x\ny", 0), "x\ny");
    assert_eq!(f("x\ny", 2), "  x\n  y");
    assert_eq!(f("x\n\n\ny", 2), "  x\n\n\n  y");
  }
}

use std::fmt::{self, Write};