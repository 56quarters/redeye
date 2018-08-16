//
//
//

use chrono::DateTime;
use regex::{Captures, Regex};
use std::collections::HashMap;
use types::{LogEvent, LogFieldValue, RedeyeError, RedeyeResult};

const COMMON_LOG_TIMESTAMP: &str = "%d/%b/%Y:%T %z";

pub trait LogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent>;
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
                "\"(",           // open " and HTTP request
                r"([^\s]+)\s",   // method
                r"([^\s]+)\s",   // path
                r"([^\s]+)",     // protocol
                ")\"\\s+",       // close " and HTTP request
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
                let ident = parse_text_value(&matches, 2, line)?;
                let username = parse_text_value(&matches, 3, line)?;
                let timestamp = parse_timestamp(&matches, 4, line, COMMON_LOG_TIMESTAMP)?;
                let request = parse_text_value(&matches, 5, line)?;
                let method = parse_text_value(&matches, 6, line)?;
                let path = parse_text_value(&matches, 7, line)?;
                let protocol = parse_text_value(&matches, 8, line)?;
                let status = parse_int_value(&matches, 9, line)?;
                let bytes = parse_int_value(&matches, 10, line)?;

                if let Some(v) = remote_host {
                    map.insert("remote_host".to_string(), v);
                }

                if let Some(v) = ident {
                    map.insert("ident".to_string(), v);
                }

                if let Some(v) = username {
                    map.insert("username".to_string(), v);
                }

                if let Some(v) = timestamp {
                    map.insert("@timestamp".to_string(), v);
                }

                if let Some(v) = request {
                    map.insert("requested_url".to_string(), v);
                }

                if let Some(v) = method {
                    map.insert("method".to_string(), v);
                }

                if let Some(v) = path {
                    map.insert("requested_uri".to_string(), v);
                }

                if let Some(v) = protocol {
                    map.insert("protocol".to_string(), v);
                }

                if let Some(v) = status {
                    map.insert("status_code".to_string(), v);
                }

                if let Some(v) = bytes {
                    map.insert("content_length".to_string(), v);
                }

                Ok(LogEvent::from(map))
            })
    }
}

fn parse_timestamp(
    matches: &Captures,
    index: usize,
    line: &str,
    format: &str,
) -> RedeyeResult<Option<LogFieldValue>> {
    let field_match = matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .map(|m| m.as_str())
        .map(|s| if s == "-" { None } else { Some(s) })?;

    if let Some(v) = field_match {
        Ok(Some(LogFieldValue::Timestamp(DateTime::parse_from_str(
            v, format,
        )?)))
    } else {
        Ok(None)
    }
}

fn parse_text_value(
    matches: &Captures,
    index: usize,
    line: &str,
) -> RedeyeResult<Option<LogFieldValue>> {
    matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .map(|m| m.as_str())
        .map(|s| {
            if s == "-" {
                None
            } else {
                Some(LogFieldValue::Text(s.to_string()))
            }
        })
}

fn parse_int_value(
    matches: &Captures,
    index: usize,
    line: &str,
) -> RedeyeResult<Option<LogFieldValue>> {
    let field_match = matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .map(|m| m.as_str())
        .map(|s| if s == "-" { None } else { Some(s) })?;

    if let Some(v) = field_match {
        Ok(Some(LogFieldValue::Int(
            v.parse::<u64>()
                .map_err(|_| RedeyeError::ParseError(line.to_string()))?,
        )))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::{CommonLogLineParser, LogLineParser};

    #[test]
    fn test_common_log_line_parser() {
        let parser = CommonLogLineParser::new();
        println!("Res: {:?}", parser.parse("125.125.125.125 - dsmith [10/Oct/1999:21:15:05 +0500] \"GET /index.html HTTP/1.0\" 200 1043"));
    }
}
