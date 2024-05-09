use crate::prelude::*;

use itertools::Itertools;
use std::collections::HashMap;

#[derive(Default)]
pub struct Headers<'a> {
    kv: HashMap<&'a str, &'a str>,
}

impl<'a> Headers<'a> {
    // TODO: override the hashing algorithm so that hashes are case-insensitive
    pub fn get(&self, key: &str) -> Option<&&'a str> {
        return self.kv.get(key);
    }
}

impl<'a> TryFrom<&[&'a str]> for Headers<'a> {
    type Error = Error;
    fn try_from(value: &[&'a str]) -> Result<Self> {
        let mut kv = HashMap::new();
        for header in value {
            let key_val = header.split(": ").collect_vec();
            if key_val.len() != 2 {
                return Err(Error::HttpBadHeaders);
            }
            kv.insert(key_val[0], key_val[1]);
        }

        return Ok(Self { kv });
    }
}
