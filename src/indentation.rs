#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Indentation(pub usize);

impl Indentation
{
  pub fn indent_str(self, text: &str) -> String
  {
    indent_lines(text, self.0)
  }

  pub fn unindent_str(self, text: &str) -> Result<String, Unindent_Error>
  {
    unindent_lines(text, self.0)
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

fn unindent_lines(input: &str, indentation: usize) -> Result<String, Unindent_Error>
{
  let mut output = String::with_capacity(input.len());

  for line in input.lines()
  {
    let keep_after = line.len().min(indentation);
    if !line.as_bytes()[..keep_after].iter().all(|&x| x==b' ')
    { return Err(Unindent_Error::NON_WS_IN_INDENTATION) }
    output.push_str(&line[keep_after..]);
    output.push('\n');
  }

  Ok(output)
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

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum Unindent_Error
{
  #[error("Non whitespace character in indentation")]
  NON_WS_IN_INDENTATION,
}

pub fn ensure_tailing_linebreak(mut xs: String) -> String
{
  if !xs.is_empty() && !xs.ends_with('\n')
  {
    xs.push('\n');
  }

  xs
}

#[cfg(test)]
mod test
{
  macro_rules! indent {
    ($i:expr, $str:expr) => {
      crate::indentation::Indentation($i).indent_str($str).as_str()
    };
  }

  macro_rules! unindent {
    ($i:expr, $str:expr) => {
      crate::indentation::Indentation($i).unindent_str($str).unwrap().as_str()
    };
  }

  macro_rules! assert_indent {
    ($i:expr, $unindented:expr, $indented:expr) => {
      assert_eq!(indent!($i, $unindented), $indented);
      assert_eq!(unindent!($i, $indented), crate::indentation::ensure_tailing_linebreak($unindented.to_owned()));
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

  #[test]
  fn test_unindent_invalid_indentation()
  {
    assert_eq!(crate::indentation::Indentation(2).unindent_str("xyz"), Err(crate::indentation::Unindent_Error::NON_WS_IN_INDENTATION));
    assert_eq!(crate::indentation::Indentation(2).unindent_str(" \n").unwrap(), "\n");
  }
}

use std::fmt::{self, Write};