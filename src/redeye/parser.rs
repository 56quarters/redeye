//
//
//

use chrono::{DateTime, Utc};
use regex::{Captures, Regex};
use std::collections::HashMap;
use types::{RedeyeError, RedeyeResult};

pub trait LogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum LogFieldValue {
    Timestamp(DateTime<Utc>),
    Text(String),
    Int(u64),
}

#[derive(Debug, Default)]
pub struct LogEvent {
    values: HashMap<String, LogFieldValue>,
}

impl From<HashMap<String, LogFieldValue>> for LogEvent {
    fn from(values: HashMap<String, LogFieldValue>) -> Self {
        Self { values }
    }
}

pub struct CommonLogLineParser {
    regex: Regex,
}

impl CommonLogLineParser {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(concat!(
                r"^([^\s]+)\s+", // host
                r"([^\s]+)\s+",  // rfc931
                r"([^\s]+)\s+",  // username
                r"\[(.+)\]\s+",  // timestamp
                "\"",            // "
                r"([^\s]+)\s",   // method
                r"([^\s]+)\s",   // path
                r"([^\s]+)",     // protocol
                "\"\\s+",        // "
                r"([^\s]+)\s+",  // status
                r"([^\s]+)$",    // bytes
            )).unwrap(),
        }
    }
}

impl LogLineParser for CommonLogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent> {
        self.regex
            .captures(line.trim())
            .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
            .and_then(|matches| {
                let mut map = HashMap::with_capacity(matches.len());
                let remote_host = parse_text_value(&matches, 1, line)?;
                let rfc931 = parse_text_value(&matches, 2, line)?;
                let username = parse_text_value(&matches, 3, line)?;
                let timestamp = parse_text_value(&matches, 4, line)?;
                let method = parse_text_value(&matches, 5, line)?;
                let path = parse_text_value(&matches, 6, line)?;
                let protocol = parse_text_value(&matches, 7, line)?;
                let status = parse_int_value(&matches, 8, line)?;
                let bytes = parse_int_value(&matches, 9, line)?;

                map.insert("remote_host".to_string(), remote_host);
                map.insert("some_nonsense".to_string(), rfc931);
                map.insert("username".to_string(), username);
                map.insert("timestamp".to_string(), timestamp);
                map.insert("method".to_string(), method);
                map.insert("request_uri".to_string(), path);
                map.insert("protocol".to_string(), protocol);
                map.insert("status_code".to_string(), status);
                map.insert("bytes".to_string(), bytes);

                Ok(LogEvent::from(map))
            })
    }
}

fn parse_request(request: &str) -> () {}

fn parse_timestamp(timestamp: &str) -> () {}

fn parse_text_value(matches: &Captures, index: usize, line: &str) -> RedeyeResult<LogFieldValue> {
    let field_match = matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))?;

    Ok(LogFieldValue::Text(field_match.as_str().to_string()))
}

fn parse_int_value(matches: &Captures, index: usize, line: &str) -> RedeyeResult<LogFieldValue> {
    let field_match = matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))?;

    let val = field_match
        .as_str()
        .parse::<u64>()
        .map_err(|_| RedeyeError::ParseError(line.to_string()))?;

    Ok(LogFieldValue::Int(val))
}

pub struct ExtendedLogLineParser {}

#[cfg(test)]
mod tests {
    use super::{CommonLogLineParser, LogLineParser};

    #[test]
    fn test_common_log_line_parser() {
        let parser = CommonLogLineParser::new();
        println!("Res: {:?}", parser.parse("125.125.125.125 - dsmith [10/Oct/1999:21:15:05 +0500] \"GET /index.html HTTP/1.0\" 200 1043"));
    }
}
