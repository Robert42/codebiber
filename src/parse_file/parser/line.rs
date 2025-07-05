use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line<'a>
{
  CODE(&'a str),
  BEGIN_CODEGEN{marker: Marker<'a>, identifier: &'a str,},
  END_CODEGEN{marker: Marker<'a>, checksum: &'a str,},
}

pub type Result<T=(), E=Syntax_Error> = std::result::Result<T, E>;

pub fn parse<'a>(line: &'a str) -> Result<Line<'a>>
{
  const BEGIN : &'static str = "<< codegen ";
  const END : &'static str = "<< /codegen";
  const TAG_END : &'static str = ">>";

  debug_assert_eq!(line.contains('\n'), false);

  if let Some(index) = line.find(BEGIN)
  {
    let mut code = &line[..index];
    let indentation = Indentation::parse(&mut code);
    let before_marker = code;

    code = &line[index+BEGIN.len() .. ];
    let identifier = parse_identifier(&mut code)?;
    skip_while(&mut code, |x| x==' ');
    expect(&mut code, TAG_END)?;
    let after_marker = code;

    let marker = Marker{indentation, before_marker, after_marker};
    return Ok(Line::BEGIN_CODEGEN{marker, identifier});
  }else if let Some(index) = line.find(END)
  {
    let mut code = &line[..index];
    let indentation = Indentation::parse(&mut code);
    let before_marker = code;

    code = &line[index+END.len() .. ];
    skip_while(&mut code, |x| x==' ');
    let checksum = eat_while(&mut code, |x| char::is_ascii_hexdigit(&x));
    skip_while(&mut code, |x| x==' ');
    expect(&mut code, TAG_END)?;
    let after_marker = code;

    let marker = Marker{indentation, before_marker, after_marker};
    return Ok(Line::END_CODEGEN{marker, checksum});
  }else
  {
    return Ok(Line::CODE(line));
  }

}

fn parse_identifier<'a>(code: &mut &'a str) -> Result<&'a str>
{
  let index = code.rfind(">>").unwrap_or(code.len());
  let ident = &(*code)[..index].trim_end();
  let rest = &(*code)[ident.len()..];
  *code = rest;
  return Ok(ident);
}

fn expect(code: &mut &str, snippet: &'static str) -> Result<>
{
  if !code.starts_with(snippet)
  {
    return Err(Syntax_Error::EXPECTED_SNIPPET(snippet))
  }
  skip(code, snippet.len());
  return Ok(())
}

fn skip(code: &mut &str, len: usize)
{
  let _ = eat(code, len);
}

fn eat<'a>(code: &mut &'a str, len: usize) -> &'a str
{
  assert!(code.len() >= len);
  let x = &code[.. len];
  *code = &code[len..];
  return x;
}

fn skip_while<P: Fn(char)->bool>(code: &mut &str, pred: P)
{
  let _ = eat_while(code, pred);
}

fn eat_while<'a, P: Fn(char)->bool>(code: &mut &'a str, pred: P) -> &'a str
{
  let len = code.find(|x| !pred(x)).unwrap_or(code.len());
  return eat(code, len);
}

#[cfg(test)]
mod test
{
  use super::*;
  #[test]
  fn identifier()
  {
    fn parse(mut code: &str) -> Result<(&str, &str), Syntax_Error>
    {
      let ident = parse_identifier(&mut code)?;
      return Ok((ident, code));
    }

    assert_eq!(parse(""), Ok(("", "")));
    assert_eq!(parse(" "), Ok(("", " ")));
    assert_eq!(parse("! "), Ok(("!", " ")));
    assert_eq!(parse("x"), Ok(("x", "")));
    assert_eq!(parse("x "), Ok(("x", " ")));
    assert_eq!(parse("x, y"), Ok(("x, y", "")));
    assert_eq!(parse("x >>"), Ok(("x", " >>")));
    assert_eq!(parse("x >> y >>"), Ok(("x >> y", " >>")));
  }

  #[test]
  fn lines() -> Result
  {
    let indentation = Indentation(2);

    assert_eq!(parse_line("")?, Line::CODE(""));
    assert_eq!(parse_line("xyz")?, Line::CODE("xyz"));
    assert_eq!(parse_line("  // << codegen foo >> let's go!")?, Line::BEGIN_CODEGEN{identifier: "foo", marker: Marker{indentation, before_marker: "// ", after_marker: " let's go!"}});
    assert_eq!(parse_line("  // << /codegen f00baa >> nice!")?, Line::END_CODEGEN{checksum: "f00baa", marker: Marker{indentation, before_marker: "// ", after_marker: " nice!"}});
    assert_eq!(parse_line("  # << /codegen 0123465789abcdef00112233445566778899aabbccddeefffedcba9876543210 >>")?, Line::END_CODEGEN{checksum: "0123465789abcdef00112233445566778899aabbccddeefffedcba9876543210", marker: Marker{indentation, before_marker: "# ", after_marker: ""}});
    assert_eq!(parse_line("  // << /codegen >> nice!")?, Line::END_CODEGEN{checksum: "", marker: Marker{indentation, before_marker: "// ", after_marker: " nice!"}});
    assert_eq!(parse_line("  // << /codegen>> nice!")?, Line::END_CODEGEN{checksum: "", marker: Marker{indentation, before_marker: "// ", after_marker: " nice!"}});

    Ok(())
  }

  fn parse_line<'a>(code: &'a str) -> Result<Line<'a>>
  {
    return super::parse(code);
  }
}
