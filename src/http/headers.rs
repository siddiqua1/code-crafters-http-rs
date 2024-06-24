use crate::prelude::*;

use itertools::Itertools;
use std::{collections::HashMap, ops::Index};

#[derive(Default, Debug, PartialEq, Clone)]
pub struct Headers<'a> {
    kv: HashMap<&'a str, &'a str>,
}

impl<'a> Headers<'a> {
    // TODO: override the hashing algorithm so that hashes are case-insensitive
    pub fn get(&self, key: &str) -> Option<&&'a str> {
        return self.kv.get(key);
    }

    pub fn is_empty(&self) -> bool {
        return self.kv.is_empty();
    }
}

impl<'a> TryFrom<&[&'a str]> for Headers<'a> {
    type Error = Error;
    fn try_from(value: &[&'a str]) -> Result<Self> {
        let mut kv = HashMap::new();
        for header in value {
            /* From the spec we have that
            Each field line consists of a case-insensitive field name followed by a colon (":"),
            optional leading whitespace, the field line value, and optional trailing whitespace.
             */
            let Some(split_idx) = header.find(':') else {
                return Err(Error::HttpBadHeaders);
            };
            if split_idx == 0 {
                return Err(Error::HttpHeaderNoKey);
            }
            let key = &header[0..split_idx];
            if key.ends_with(' ') {
                return Err(Error::HttpBadHeaders);
            }
            if split_idx == header.len() - 1 {
                return Err(Error::HttpHeaderNoValue);
            }
            let value = &header[split_idx + 1..];
            let start = if value.starts_with(' ') { 1 } else { 0 };
            let end = if value.ends_with(' ') {
                value.len() - 1
            } else {
                value.len()
            };
            if start > end {
                return Err(Error::HttpHeaderNoValue);
            }
            let value = &value[start..end];
            // TODO: RFC2616 states that if multiple keys we need to create a comma seperated array?
            kv.insert(key, value);
        }

        return Ok(Self { kv });
    }
}
