use chrono::prelude::*;
use std::collections::HashMap;
use types::RedeyeResult;

pub struct Enricher {}

pub struct Parser {}

pub fn from_plain_text(line: String) -> RedeyeResult<LogMessage> {
    unimplemented!();
}

pub fn from_json(line: String) -> RedeyeResult<LogMessage> {
    unimplemented!();
}

pub struct LogMessage {
    time: DateTime<Utc>,
    line: String,
    tags: HashMap<String, String>,
}
