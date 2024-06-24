#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(unused)] // temporary

// mod core;
// pub use core::*;

pub mod callback;
pub mod error;
pub mod http;
pub mod matching;
mod prelude;
