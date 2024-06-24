use crate::prelude::*;

use std::collections::HashMap;

#[derive(Default)]
pub struct Identifiers<'a> {
    path_values: HashMap<&'static str, &'a str>,
}
