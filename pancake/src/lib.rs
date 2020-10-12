#![feature(proc_macro_hygiene)]

//#[macro_use] extern crate flamer;

pub mod arity;
pub mod eval;
pub mod ops;
pub mod parse;
pub mod types;
pub mod vm;

pub mod ast;
pub mod vm2;
pub mod inference_data;
pub mod typeck;

