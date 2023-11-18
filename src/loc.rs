#[derive(Clone, Copy)]
pub struct Loc
{
  pub line: u32,
  pub column: u32,
}

impl Loc
{
  pub fn to_index(self, code: &str) -> Result<usize, Loc_To_Index_Error>
  {
    let col_begin = line_to_index(self.line, code)?;
    let pos = col_begin + self.column as usize;

    if pos > code.len() {return Err(Loc_To_Index_Error::NOT_EXISTNIG_COLUMN)}
    if code[col_begin..pos].as_bytes().iter().copied().any(|x| x==b'\n')  {return Err(Loc_To_Index_Error::NOT_EXISTNIG_COLUMN)}

    Ok(pos)
  }
}

fn line_to_index(line: u32, code: &str) -> Result<usize, Loc_To_Index_Error>
{
  if line == 0 {return Ok(0)}
  
  let code = code.as_bytes();
  let mut curr_line = 0;
  for (idx,x) in code.iter().copied().enumerate()
  {
    if x != b'\n' {continue;}

    curr_line += 1;
    if curr_line == line
    {
      return Ok(idx+1);
    }
  }

  Err(Loc_To_Index_Error::NOT_EXISTNIG_LINE)
}

pub fn loc(line: u32, column: u32) -> Loc
{
  Loc{line, column}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Loc_To_Index_Error
{
  NOT_EXISTNIG_LINE,
  NOT_EXISTNIG_COLUMN,
}

#[cfg(test)]
mod test
{
  use super::*;

  #[test]
  fn loc_to_index()
  {
    assert_eq!(loc(0, 0).to_index(""), Ok(0));
    assert_eq!(loc(0, 0).to_index(" "), Ok(0));
    assert_eq!(loc(0, 1).to_index(" "), Ok(1));
    assert_eq!(loc(1, 0).to_index(" \n "), Ok(2));
    assert_eq!(loc(1, 1).to_index(" \n "), Ok(3));
    assert_eq!(loc(0, 1).to_index(" \n "), Ok(1));
    assert_eq!(loc(1, 0).to_index(""), Err(Loc_To_Index_Error::NOT_EXISTNIG_LINE));
    assert_eq!(loc(0, 1).to_index(""), Err(Loc_To_Index_Error::NOT_EXISTNIG_COLUMN));
    assert_eq!(loc(0, 2).to_index(" \n "), Err(Loc_To_Index_Error::NOT_EXISTNIG_COLUMN));
  }
}
