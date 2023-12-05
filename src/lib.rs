#![feature(slice_as_chunks)]
#![allow(non_camel_case_types)]

/*!

```rust
extern crate codebiber;

fn main() -> codebiber::Result
{
  let cfg = codebiber::Config{
    // Anything checksum length other than 0 will catch unintended modifications
    // since the last modification.
    checksum_bytes_to_store: 3,
  };

  {
    let input = "void handwritten_line1();\n\
                 void handwritten_line2();\n\

                 // << codegen foo >>\n\
                 // << /codegen >>\n\

                 void handwritten_line3();\n\

                 // << codegen bar >>\n\
                 // << /codegen >>\n\

                 void handwritten_line4();\n\

                 // << codegen baz >>\n\
                 void generated_line_by_some_other_function();\n\
                 // << /codegen >>\n\

                 void handwritten_line5();\n\
                 ";
    let expected_output = "void handwritten_line1();\n\
                           void handwritten_line2();\n\

                           // << codegen foo >>\n\
                           void autogen_line_foo();\n\
                           // << /codegen aaa272 >>\n\

                           void handwritten_line3();\n\

                           // << codegen bar >>\n\
                           void autogen_line_bar1();\n\
                           void autogen_line_bar2();\n\
                           // << /codegen 00a214 >>\n\

                           void handwritten_line4();\n\

                           // << codegen baz >>\n\
                           void generated_line_by_some_other_function();\n\
                           // << /codegen 810c07 >>\n\

                           void handwritten_line5();\n\
                           ".to_owned();

    let actual_output = codebiber::generate(input, cfg, gen_code_lines)?;

    assert_eq!(actual_output, Some(expected_output));
  }

  Ok(())
}

fn gen_code_lines(name: &str) -> codebiber::Fmt_Result
{
  let generated = match name
  {
    "foo" => Some("void autogen_line_foo();".to_owned()),
    "bar" => Some("void autogen_line_bar1();\nvoid autogen_line_bar2();".to_owned()),
    _ => None,
  };
  Ok(generated)
}
```

*/

pub mod parse_file;
pub mod indentation;
pub mod process;
pub mod gen;

pub use indentation::Indentation;
pub use gen::{generate, Config, Fmt_Result};
pub use process::{process_file, process_files, Process_Error as Error, Result};

pub mod pretty_unwrap;
pub use pretty_unwrap::Pretty_Unwrap;

extern crate blake3;

extern crate arrayvec;
use arrayvec::ArrayVec;

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate smallvec;
use smallvec::SmallVec;

#[macro_use]
extern crate thiserror;