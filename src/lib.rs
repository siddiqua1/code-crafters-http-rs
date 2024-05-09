#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]
#![allow(unused)] // temporary

mod core;
pub use core::*;

mod error;
mod http;
mod prelude;
