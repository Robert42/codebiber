#![feature(slice_as_chunks)]
#![allow(non_camel_case_types)]

pub mod parse_file;
pub mod indentation;
pub mod process;
pub mod gen;

pub use gen::{Codegen_Usage, Codegen_Result};
pub use process::{process_file, process_files, Process_Error as Error, Result};

pub mod pretty_unwrap;
pub use pretty_unwrap::Pretty_Unwrap;

extern crate blake3;

extern crate arrayvec;
use arrayvec::ArrayVec;

#[cfg(test)]
extern crate proptest;

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate smallvec;
use smallvec::SmallVec;

#[macro_use]
extern crate thiserror;