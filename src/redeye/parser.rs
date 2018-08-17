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
                r"([^\s]+)\s+",  // rfc931 ident
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
        line_to_event(&self.regex, line, |matches, full_line, mut map| {
            add_field(&mut map, "remote_host", parse_text_value(&matches, 1, full_line))?;
            add_field(&mut map, "ident", parse_text_value(&matches, 2, full_line))?;
            add_field(&mut map, "username", parse_text_value(&matches, 3, full_line))?;
            add_field(&mut map, "@timestamp", parse_timestamp(&matches, 4, full_line, COMMON_LOG_TIMESTAMP))?;
            add_field(&mut map, "requested_url", parse_text_value(&matches, 5, full_line))?;
            add_field(&mut map, "method", parse_text_value(&matches, 6, full_line))?;
            add_field(&mut map, "requested_uri", parse_text_value(&matches, 7, full_line))?;
            add_field(&mut map, "protocol", parse_text_value(&matches, 8, full_line))?;
            add_field(&mut map, "status_code", parse_int_value(&matches, 9, full_line))?;
            add_field(&mut map, "content_length", parse_int_value(&matches, 10, full_line))?;
            Ok(())
        })
    }
}

pub struct CombinedLogLineParser {
    regex: Regex,
}

impl CombinedLogLineParser {
    pub fn new() -> Self {
        Self {
            regex: Regex::new(concat!(
                r"^([^\s]+)\s+",     // host
                r"([^\s]+)\s+",      // rfc931 ident
                r"([^\s]+)\s+",      // username
                r"\[(.+)\]\s+",      // timestamp
                "\"(",               // open " and HTTP request
                r"([^\s]+)\s",       // method
                r"([^\s]+)\s",       // path
                r"([^\s]+)",         // protocol
                ")\"\\s+",           // close " and HTTP request
                r"([^\s]+)\s+",      // status
                r"([^\s]+)\s+",      // bytes
                "\"([^\"]+)\"\\s+",  // "referer" [sic]
                "\"([^\"]+)\"$",     // "user agent"
            )).unwrap(),
        }
    }
}

impl LogLineParser for CombinedLogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent> {
        line_to_event(&self.regex, line, |matches, full_line, mut map| {
            add_field(&mut map, "remote_host", parse_text_value(&matches, 1, full_line))?;
            add_field(&mut map, "ident", parse_text_value(&matches, 2, full_line))?;
            add_field(&mut map, "username", parse_text_value(&matches, 3, full_line))?;
            add_field(&mut map, "@timestamp", parse_timestamp(&matches, 4, full_line, COMMON_LOG_TIMESTAMP))?;
            add_field(&mut map, "requested_url", parse_text_value(&matches, 5, full_line))?;
            add_field(&mut map, "method", parse_text_value(&matches, 6, full_line))?;
            add_field(&mut map, "requested_uri", parse_text_value(&matches, 7, full_line))?;
            add_field(&mut map, "protocol", parse_text_value(&matches, 8, full_line))?;
            add_field(&mut map, "status_code", parse_int_value(&matches, 9, full_line))?;
            add_field(&mut map, "content_length", parse_int_value(&matches, 10, full_line))?;

            let mut headers = HashMap::with_capacity(2);
            add_field(&mut headers, "referer" /* [sic] */, parse_text_value(&matches, 11, full_line))?;
            add_field(&mut headers, "user-agent", parse_text_value(&matches, 12, full_line))?;

            if !headers.is_empty() {
                map.insert("request_headers".to_string(), LogFieldValue::Mapping(headers));
            }

            Ok(())
        })
    }
}

fn line_to_event<F>(reg: &Regex, line: &str, func: F) -> RedeyeResult<LogEvent>
where
    F: FnOnce(&Captures, &str, &mut HashMap<String, LogFieldValue>) -> RedeyeResult<()>
{
    reg
        .captures(line.trim())
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .and_then(|matches| {
            let mut map = HashMap::with_capacity(matches.len());
            func(&matches, line, &mut map)?;
            Ok(LogEvent::from(map))
        })
}

fn add_field<S>(
    map: &mut HashMap<String, LogFieldValue>,
    field: S,
    res: RedeyeResult<Option<LogFieldValue>>,
) -> RedeyeResult<()>
    where
        S: Into<String>,
{
    res.map(|o| {
        if let Some(v) = o {
            map.insert(field.into(), v);
        }
    })
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
        Ok(Some(LogFieldValue::Timestamp(DateTime::parse_from_str(v, format)?)))
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
        .map(|s| if s == "-" { None } else { Some(LogFieldValue::Text(s.to_string()))})
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
        let val = v.parse::<u64>().map_err(|_| RedeyeError::ParseError(line.to_string()))?;
        Ok(Some(LogFieldValue::Int(val)))
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
