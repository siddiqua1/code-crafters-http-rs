//! Crate Error
// Moving away from anyhow errors for internal errors

use derive_more::From;

#[derive(Debug, From, Clone)]
pub enum Error {
    #[from]
    Generic(String),
    #[from]
    Static(&'static str),

    // -- Http Errors
    #[from]
    HttpUnableToParse,
    HttpMalformedRequest,
    HttpUnsupportedVersion,
    HttpUnsupportedMethod,
    HttpPathParsing,
    HttpBadHeaders,
    HttpHeaderNoKey,
    HttpHeaderNoValue,
}
