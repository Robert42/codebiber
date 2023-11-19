#![allow(non_camel_case_types)]
#![cfg(test)]

mod find;
pub mod loc;

extern crate blake3;

extern crate pest;
#[macro_use]
extern crate pest_derive;

#[macro_use]
extern crate smallvec;
use smallvec::SmallVec;

#[macro_use]
extern crate thiserror;