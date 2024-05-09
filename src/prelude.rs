//! Main Crate Prelude

pub use crate::error::Error;
pub type Result<T> = core::result::Result<T, Error>;

// probably will end up using these components alot, hoist to global
pub use crate::http::headers::Headers;
pub use crate::http::method::Method;
pub use crate::http::path::Path;
pub use crate::http::request::Request;
pub use crate::http::version::Version;

pub use std::format as fmt;
