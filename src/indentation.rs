#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl Indentation
{
  pub fn indent_str(self, text: &str) -> String
  {
    indent_lines(text, self.0)
  }
}

fn indent_lines(input: &str, indentation: usize) -> String
{
  let mut output = String::with_capacity((input.len()+1)*(indentation+1));

  let indentation = std::iter::repeat(' ').take(indentation);
  for line in input.lines()
  {
    if !line.is_empty()
    {
      output.extend(indentation.clone());
      output.push_str(line);
    }
    output.push('\n');
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

  macro_rules! assert_indent {
    ($i:expr, $unindented:expr, $indented:expr) => {
      assert_eq!(indent!($i, $unindented), $indented);
    };
  }

  #[test]
  fn test_trivial()
  {
    assert_indent!(0, "", "");
    assert_indent!(0, "x", "x\n");
    assert_indent!(0, "x\ny", "x\ny\n");
  }

  #[test]
  fn test_simpl()
  {
    assert_indent!(2, "", "");
    assert_indent!(2, "x", "  x\n");
    assert_indent!(2, "x\ny", "  x\n  y\n");
    assert_indent!(4, "Hello, World!", "    Hello, World!\n");
  }

  #[test]
  fn test_with_lienbreak()
  {
    assert_indent!(2, "x\ny\nz", "  x\n  y\n  z\n");
  }

  #[test]
  fn test_dont_add_trailing_whitespace()
  {
    assert_indent!(2, "x\n\n\ny", "  x\n\n\n  y\n");
    assert_indent!(2, "x\n\ny\n\n\nz", "  x\n\n  y\n\n\n  z\n");
    assert_indent!(2, "x\n", "  x\n");
  }

  #[test]
  fn test_difficult_cases()
  {
    assert_indent!(2, "\nx", "\n  x\n");
  }
}

use std::fmt::{self, Write};