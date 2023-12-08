#![feature(slice_as_chunks)]
#![allow(non_camel_case_types)]

/*!

## Example

A silly example of using codebiber to mix autogenerated code with handwritten code.

Here, the original code contans handwritten lines
```c
void handwritten_line1();
void handwritten_line2();
```

```c
void handwritten_line3();
```

```c
void handwritten_line4();
```

```c
void handwritten_line5();
```

and some sections marked to be overwritten

```c
// << codegen foo >>
// << /codegen >>
```

```c
// << codegen bar >>
// << /codegen >>
```

and a section that was already generated by another function

```c
// << codegen baz >>
void generated_line_by_some_other_function();
// << /codegen >>
```

The function [`generate`]! accepts the input code, a [configuration](Config) and
most important the function which will generate the code.

In our example, the function will be called once for each section.
Each time it accepts the section name (in our case `foo`, `bar` or `baz`) and
returns the generated code for this section.
If the section is not its responsibility, it can also return Ok(None) and the
previous content will be used again.

In our example, the generator function `gen_code_lines` returns new code for the
sections `foo` and `bar` while leaving `baz` untouched.

This results in the  generated code section
```c
// << codegen foo >>
void autogen_line_foo();
// << /codegen aaa272 >>
```

```c
// << codegen bar >>
void autogen_line_bar1();
void autogen_line_bar2();
// << /codegen 00a214 >>
```

```c
// << codegen baz >>
void generated_line_by_some_other_function();
// << /codegen 810c07 >>
```

Note the hashsums. They protect against overwritting accidental modifications.
They are simply the first few bytes of a blake3 hahsum
(how many [can be configured](Config)).

```rust
extern crate codebiber;

const INPUT : &str = r"
void handwritten_line1();
void handwritten_line2();

// << codegen foo >>
// << /codegen >>

void handwritten_line3();

  // << codegen bar >>
  // << /codegen >>

void handwritten_line4();

// << codegen baz >>
void generated_line_by_some_other_function();
// << /codegen >>

void handwritten_line5();
";
              
fn main() -> codebiber::Result
{
  let cfg = codebiber::Config{
    // Anything checksum length other than 0 will catch unintended modifications
    // since the last modification.
    checksum_bytes_to_store: 3,
  };

  let actual_output = codebiber::generate(INPUT, cfg, gen_code_lines)?;

  assert_eq!(actual_output, Some(EXPECTED_OUTPUT.to_owned()));

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

const EXPECTED_OUTPUT : &str = r"
void handwritten_line1();
void handwritten_line2();

// << codegen foo >>
void autogen_line_foo();
// << /codegen aaa272 >>

void handwritten_line3();

  // << codegen bar >>
  void autogen_line_bar1();
  void autogen_line_bar2();
  // << /codegen 00a214 >>

void handwritten_line4();

// << codegen baz >>
void generated_line_by_some_other_function();
// << /codegen 810c07 >>

void handwritten_line5();
";
```

Please not how the generated code in the `bar` section is indented.
That's because the generated code inherits the indentation of the marker line
`// << codegen bar >>`

# Example 2

While in this example all marker lines start with `//`, marker lines can start
and with any characters as long as they don't end with `<` and don't contain
`<<`.

```rust
extern crate codebiber;

const INPUT : &str = r"
/* << codegen foo >> */
(* << /codegen >> *)

#if 0 // << codegen bar >>
#endif // << /codegen >>

  ## << codegen bar >> For python like languages
  -- << /codegen >> Note how you can also write stuff after the marker

    esoteric langugage using keywords << codegen bar >> for comments
    << /codegen >>
";
              
fn main() -> codebiber::Result
{
  let cfg = codebiber::Config{
    // Anything checksum length other than 0 will catch unintended modifications
    // since the last modification.
    checksum_bytes_to_store: 2,
  };

  let actual_output = codebiber::generate(INPUT, cfg, gen_code_lines)?;

  assert_eq!(actual_output, Some(EXPECTED_OUTPUT.to_owned()));

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

const EXPECTED_OUTPUT : &str = r"
/* << codegen foo >> */
void autogen_line_foo();
(* << /codegen aaa2 >> *)

#if 0 // << codegen bar >>
void autogen_line_bar1();
void autogen_line_bar2();
#endif // << /codegen 00a2 >>

  ## << codegen bar >> For python like languages
  void autogen_line_bar1();
  void autogen_line_bar2();
  -- << /codegen 00a2 >> Note how you can also write stuff after the marker

    esoteric langugage using keywords << codegen bar >> for comments
    void autogen_line_bar1();
    void autogen_line_bar2();
    << /codegen 00a2 >>
";
```

Also note how every `bar` section shares the same hashsum. Thats because the
hashsum is generated before indenting the code.

*/

pub mod parse_file;
pub mod indentation;
pub mod process;
pub mod gen;

pub use indentation::Indentation;
pub use gen::{generate, Config, Fmt_Result};
pub use process::{process_file, process_files, Process_Error as Error, Result};

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

#[cfg(test)]
extern crate unwrap_display;
#[cfg(test)]
use unwrap_display::UnwrapDisplay;