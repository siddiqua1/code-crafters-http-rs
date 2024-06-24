use crate::prelude::Result;
pub type Response = Result<Vec<u8>>;

pub mod asynchronous;
pub mod synchronous;
