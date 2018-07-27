use chrono::prelude::*;
use serde_json;
use std::collections::HashMap;
use types::RedeyeResult;

const TIMESTAMP_KEY: &str = "@timesamp";
const LOG_LINE_KEY: &str = "message";

pub struct Enricher {}

pub struct LineParser;

impl LineParser {
    fn from_plain_text(line: String) -> RedeyeResult<LogMessage> {
        let msg = LogMessage::new(line);
        Ok(msg)
    }

    fn from_json(line: String) -> RedeyeResult<LogMessage> {
        unimplemented!();
    }

    fn is_json_message(line: &str) -> bool {
        line.starts_with('{') && line.ends_with('}')
    }

    pub fn parse_line(&self, line: String) -> RedeyeResult<LogMessage> {
        if Self::is_json_message(&line) {
            Self::from_json(line)
        } else {
            Self::from_plain_text(line)
        }
    }
}

impl Default for LineParser {
    fn default() -> Self {
        LineParser
    }
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    time: DateTime<Utc>,
    line: String,
}

impl LogMessage {
    fn new(line: String) -> Self {
        LogMessage {
            time: Utc::now(),
            line,
        }
    }
}
