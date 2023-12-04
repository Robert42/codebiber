pub trait Pretty_Unwrap
{
  type Inner;

  #[track_caller]
  fn pretty_unwrap(self) -> Self::Inner;

  #[track_caller]
  fn pretty_expect_with_code(self, code: &str) -> Self::Inner;
}

impl<T, E: std::fmt::Display> Pretty_Unwrap for Result<T, E>
{
  type Inner = T;

  fn pretty_unwrap(self) -> T
  {
    match self
    {
      Ok(x) => x,
      Err(e) =>
      {
        panic!("{e}");
      }
    }
  }

  #[track_caller]
  fn pretty_expect_with_code(self, code: &str) -> Self::Inner
  {
    match self
    {
      Ok(x) => x,
      Err(e) =>
      {
        panic!("{e}\ncode:\n```\n{code}\n```");
      }
    }
  }
}