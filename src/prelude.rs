//! Main Crate Prelude

pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

// probably will end up using these components alot, hoist to global
pub use crate::callback::Response;
pub use crate::http::Headers;
pub use crate::http::Method;
pub use crate::http::Path;
pub use crate::http::Request;
pub use crate::http::Version;

pub use std::format as fmt;
