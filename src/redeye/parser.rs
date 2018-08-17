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
    inner: InnerParser,
}

impl CommonLogLineParser {
    pub fn new() -> Self {
        Self {
            inner: InnerParser::new(Regex::new(concat!(
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
            )).unwrap())
        }
    }
}

impl LogLineParser for CommonLogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent> {
        let fields = self.inner.apply(line)?
            .add_text_field("remote_host", 1)?
            .add_text_field("ident", 2)?
            .add_text_field("username", 3)?
            .add_timestamp_field("@timestamp", 4, COMMON_LOG_TIMESTAMP)?
            .add_text_field("requested_url", 5)?
            .add_text_field("method", 6)?
            .add_text_field("requested_uri", 7)?
            .add_text_field("protocol", 8)?
            .add_int_field("status_code", 9)?
            .add_int_field("content_length", 10)?
            .build();

        Ok(LogEvent::from(fields))
    }
}

pub struct CombinedLogLineParser {
    inner: InnerParser,
}

impl CombinedLogLineParser {
    pub fn new() -> Self {
        Self {
            inner: InnerParser::new(Regex::new(concat!(
                r"^([^\s]+)\s+",    // host
                r"([^\s]+)\s+",     // rfc931 ident
                r"([^\s]+)\s+",     // username
                r"\[(.+)\]\s+",     // timestamp
                "\"(",              // open " and HTTP request
                r"([^\s]+)\s",      // method
                r"([^\s]+)\s",      // path
                r"([^\s]+)",        // protocol
                ")\"\\s+",          // close " and HTTP request
                r"([^\s]+)\s+",     // status
                r"([^\s]+)\s+",     // bytes
                "\"([^\"]+)\"\\s+", // "referer" [sic]
                "\"([^\"]+)\"$",    // "user agent"
            )).unwrap())
        }
    }
}

impl LogLineParser for CombinedLogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent> {

        let fields = self.inner.apply(line)?
            .add_text_field("remote_host", 1)?
            .add_text_field("ident", 2)?
            .add_text_field("username", 3)?
            .add_timestamp_field("@timestamp", 4, COMMON_LOG_TIMESTAMP)?
            .add_text_field("requested_url", 5)?
            .add_text_field("method", 6)?
            .add_text_field("requested_uri", 7)?
            .add_text_field("protocol", 8)?
            .add_int_field("status_code", 9)?
            .add_int_field("content_length", 10)?
            .add_mapping_field("request_headers")
            .add_text_field("referer", 11)?
            .add_text_field("user-agent", 12)?
            .build_mapping()
            .build();

        Ok(LogEvent::from(fields))
    }
}

struct InnerParser {
    regex: Regex,
}

impl InnerParser {
    fn new(regex: Regex) -> Self {
        Self { regex }
    }

    fn apply<'a, 'b>(&'a self, line: &'b str) -> RedeyeResult<InnerParseOutput>
    where
        'b: 'a
    {
        self.regex
            .captures(line.trim())
            .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
            .map(|matches| {
                InnerParseOutput::from(line, matches)
            })
    }
}

struct MappingBuilder<'a> {
    field: String,
    values: HashMap<String, LogFieldValue>,
    output: InnerParseOutput<'a>,
}

impl<'a> MappingBuilder<'a> {
    fn add_text_field<S>(mut self, field: S, index: usize) -> RedeyeResult<Self>
        where S: Into<String>
    {
        let res = parse_text_value(&self.output.captures, index, self.output.line)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)

    }
    fn add_timestamp_field<S>(mut self, field: S, index: usize, format: &str) -> RedeyeResult<Self>
        where S: Into<String>
    {
        let res = parse_timestamp(&self.output.captures, index, self.output.line, format)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }
    fn add_int_field<S>(mut self, field: S, index: usize) -> RedeyeResult<Self>
        where S: Into<String>
    {
        let res = parse_int_value(&self.output.captures, index, self.output.line)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }

    fn build_mapping(mut self) -> InnerParseOutput<'a> {
        if !self.values.is_empty() {
            self.output.values.insert(self.field, LogFieldValue::Mapping(self.values));
        }

        self.output
    }
}

struct InnerParseOutput<'a> {
    values: HashMap<String, LogFieldValue>,
    line: &'a str,
    captures: Captures<'a>,
}

impl<'a> InnerParseOutput<'a> {
    fn from(line: &'a str, captures: Captures<'a>) -> Self {
        let len = captures.len();
        Self {
            captures,
            line,
            values: HashMap::with_capacity(len),
        }
    }

    fn add_text_field<S>(mut self, field: S, index: usize) -> RedeyeResult<Self>
        where S: Into<String>
    {
        let res = parse_text_value(&self.captures, index, self.line)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)

    }
    fn add_timestamp_field<S>(mut self, field: S, index: usize, format: &str) -> RedeyeResult<Self>
        where S: Into<String>
    {
        let res = parse_timestamp(&self.captures, index, self.line, format)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }
    fn add_int_field<S>(mut self, field: S, index: usize) -> RedeyeResult<Self>
        where S: Into<String>
    {
        let res = parse_int_value(&self.captures, index, self.line)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }

    fn add_mapping_field<S>(self, field: S) -> MappingBuilder<'a>
    where
    S: Into<String>
    {
        MappingBuilder {
            field: field.into(),
            values: HashMap::new(),
            output: self,
        }
    }

    fn build(self) -> HashMap<String, LogFieldValue> {
        self.values
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
