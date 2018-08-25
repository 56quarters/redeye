// Redeye - Parse Apache-style access logs into Logstash JSON
//
// Copyright 2018 TSH Labs
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.
//

//! Parsers for various access log formats

use chrono::DateTime;
use regex::{Captures, Regex};
use std::collections::HashMap;
use std::rc::Rc;
use types::{LogEvent, LogFieldValue, RedeyeError, RedeyeResult};

const COMMON_LOG_TIMESTAMP: &str = "%d/%b/%Y:%T %z";
const OUTPUT_VERSION: &str = "1";

/// Parse a single log line of a pre-determined format into an object
/// suitable for being serialized into Logstash compatible JSON.
///
/// Implementations ignore leading and trailing whitespace and will
/// remove it before attempting to parse a line.
pub trait LogLineParser {
    /// Parse the given log line into a `LogEvent`.
    ///
    /// Return an error if the line does not match the expected format
    /// (implementation defined) or if a field in the line does not match
    /// the expected type (also implementation defined).
    ///
    /// The fields of the `LogEvent` object should match the names expected
    /// by [Logstash](https://github.com/logstash/logstash-logback-encoder#standard-fields).
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent>;
}

/// Implementation of a `LogLineParser` that parses access logs in the
/// NCSA Common Log Format into an object suitable for being serialized
/// into Logstash compatible JSON.
///
/// An example of the Common Log Format and the resulting fields that will
/// be parsed by this implementation are given below.
///
/// # Logs
///
/// An example of a log line in this format is given below.
///
/// ```text
/// 127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326
/// ```
///
/// In this log line, the fields of a parsed `LogEvent` object would be
/// (in JSON).
///
/// ```json
/// {
///   "remote_host": "127.0.0.1",
///   "remote_user": "frank",
///   "@timestamp": "2000-10-10T13:55:36-07:00",
///   "requested_url": "GET /index.html HTTP/1.0",
///   "method": "GET",
///   "requested_uri": "/index.html",
///   "protocol": "HTTP/1.0",
///   "status_code": 200,
///   "content_length": 2326,
///   "@version": "1",
///   "message": "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /index.html HTTP/1.0\" 200 2326"
/// }
/// ```
///
/// Some things to note about this example:
/// * The request portion of the log line has been parsed into method, path,
///   and protocol components.
/// * The second field (the "-" in the original log line) has been omitted
///   because the "-" represents a missing value.
/// * The timestamp field has a `@` prefix because it has special meaning
///   to Logstash.
/// * The field `@version` has been added and has special meaning to Logstash.
/// * The field `message` contains the entire original log line.
///
/// See the [Apache docs](https://httpd.apache.org/docs/current/logs.html#accesslog)
/// for the specifics of the log line format.
///
/// # Example
///
/// ```rust
/// use redeye::parser::{LogLineParser, CommonLogLineParser};
/// use redeye::types::LogFieldValue;
///
/// let parser = CommonLogLineParser::new();
/// let event = parser.parse("127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /index.html HTTP/1.0\" 200 2326").unwrap();
/// let fields = event.fields();
/// let request = fields.get("requested_url").unwrap();
/// let status = fields.get("status_code").unwrap();
///
/// assert_eq!(
///     &LogFieldValue::Text("GET /index.html HTTP/1.0".to_string()),
///     request
/// );
///
/// assert_eq!(&LogFieldValue::Int(200), status);
/// ```
#[derive(Debug, Clone)]
pub struct CommonLogLineParser {
    inner: ParserImpl,
}

impl CommonLogLineParser {
    pub fn new() -> Self {
        Self {
            inner: ParserImpl::new(
                Regex::new(concat!(
                    r"^([^\s]+)\s+", // host
                    r"([^\s]+)\s+",  // rfc1413 ident
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
            ),
        }
    }
}

impl LogLineParser for CommonLogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent> {
        let line = line.trim();

        let fields = self
            .inner
            .apply(line)?
            .add_text_field("remote_host", 1)?
            .add_text_field("ident", 2)?
            .add_text_field("remote_user", 3)?
            .add_timestamp_field("@timestamp", 4, COMMON_LOG_TIMESTAMP)?
            .add_text_field("requested_url", 5)?
            .add_text_field("method", 6)?
            .add_text_field("requested_uri", 7)?
            .add_text_field("protocol", 8)?
            .add_int_field("status_code", 9)?
            .add_int_field("content_length", 10)?
            .add_fixed_value("@version", OUTPUT_VERSION)
            .add_fixed_value("message", line)
            .build();

        Ok(LogEvent::from(fields))
    }
}

/// Implementation of a `LogLineParser` that parses access logs in the
/// NCSA Combined Log Format into an object suitable for being serialized
/// into Logstash compatible JSON.
///
/// This format is nearly identical to the Common Log Format except for the
/// addition of two extra fields: The referrer (spelled as "referer") and
/// the user agent.
///
/// An example of the Common Log Format and the resulting fields that will
/// be parsed by this implementation are given below.
///
/// # Logs
///
/// An example of a log line in this format is given below.
///
/// ```text
/// 127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] "GET /index.html HTTP/1.0" 200 2326 "http://www.example.com/start.html" "Mozilla/4.08 [en] (Win98; I ;Nav)"
/// ```
///
/// In this log line, the fields of a parsed `LogEvent` object would be
/// (in JSON).
///
/// ```json
/// {
///   "remote_host": "127.0.0.1",
///   "remote_user": "frank",
///   "@timestamp": "2000-10-10T13:55:36-07:00",
///   "requested_url": "GET /index.html HTTP/1.0",
///   "method": "GET",
///   "requested_uri": "/index.html",
///   "protocol": "HTTP/1.0",
///   "status_code": 200,
///   "content_length": 2326,
///   "request_headers": {
///     "referer": "http://www.example.com/start.html",
///     "user_agent": "Mozilla/4.08 [en] (Win98; I ;Nav)"
///   },
///   "@version": "1",
///   "message": "127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /index.html HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\""
/// }
/// ```
///
/// Some things to note about this example:
/// * The request portion of the log line has been parsed into method, path,
///   and protocol components.
/// * The second field (the "-" in the original log line) has been omitted
///   because the "-" represents a missing value.
/// * The timestamp field has a `@` prefix because it has special meaning
///   to Logstash.
/// * The extra fields come from request headers and so are in a nested object.
/// * The field `@version` has been added and has special meaning to Logstash.
/// * The field `message` contains the entire original log line.
///
/// See the [Apache docs](https://httpd.apache.org/docs/current/logs.html#accesslog)
/// for the specifics of the log line format.
///
/// # Example
///
/// ```rust
/// use redeye::parser::{LogLineParser, CombinedLogLineParser};
/// use redeye::types::LogFieldValue;
///
/// let parser = CombinedLogLineParser::new();
/// let event = parser.parse("127.0.0.1 - frank [10/Oct/2000:13:55:36 -0700] \"GET /index.html HTTP/1.0\" 200 2326 \"http://www.example.com/start.html\" \"Mozilla/4.08 [en] (Win98; I ;Nav)\"").unwrap();
/// let fields = event.fields();
/// let request_headers = fields.get("request_headers").unwrap();
/// let headers = match request_headers {
///     LogFieldValue::Mapping(map) => map,
///     _ => { panic!("Should be a mapping!"); },
/// };
///
/// assert_eq!(
///     &LogFieldValue::Text("http://www.example.com/start.html".to_string()),
///     headers.get("referer").unwrap(),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct CombinedLogLineParser {
    inner: ParserImpl,
}

impl CombinedLogLineParser {
    pub fn new() -> Self {
        Self {
            inner: ParserImpl::new(
                Regex::new(concat!(
                    r"^([^\s]+)\s+",    // host
                    r"([^\s]+)\s+",     // rfc1413 ident
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
                )).unwrap(),
            ),
        }
    }
}

impl LogLineParser for CombinedLogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent> {
        let line = line.trim();

        let fields = self
            .inner
            .apply(line)?
            .add_text_field("remote_host", 1)?
            .add_text_field("ident", 2)?
            .add_text_field("remote_user", 3)?
            .add_timestamp_field("@timestamp", 4, COMMON_LOG_TIMESTAMP)?
            .add_text_field("requested_url", 5)?
            .add_text_field("method", 6)?
            .add_text_field("requested_uri", 7)?
            .add_text_field("protocol", 8)?
            .add_int_field("status_code", 9)?
            .add_int_field("content_length", 10)?
            .add_mapping_field("request_headers")
            .add_text_field("referer", 11)?
            .add_text_field("user_agent", 12)?
            .complete_mapping()
            .add_fixed_value("@version", OUTPUT_VERSION)
            .add_fixed_value("message", line)
            .build();

        Ok(LogEvent::from(fields))
    }
}

/// Regex-based parser for constructing logging events from an access log.
///
/// The provided regular expression is applied and log line and a builder is
/// returned that is used to parse captured values and build up a `HashMap`
/// of fields and values.
#[derive(Debug, Clone)]
struct ParserImpl {
    regex: Regex,
}

impl ParserImpl {
    fn new(regex: Regex) -> Self {
        Self { regex }
    }

    fn apply<'a>(&'a self, line: &'a str) -> RedeyeResult<FieldBuilder> {
        self.regex
            .captures(line)
            .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
            .map(|matches| FieldBuilder::root(line, matches))
    }
}

/// Builder for constructing a `HashMap` of fields and values based
/// on the results of parsing log values from the provided `Captures`
/// object.
#[derive(Debug)]
struct FieldBuilder<'a> {
    line: &'a str,
    captures: Rc<Captures<'a>>,
    field: Option<String>,
    parent: Option<Box<FieldBuilder<'a>>>,
    values: HashMap<String, LogFieldValue>,
}

impl<'a> FieldBuilder<'a> {
    /// Create a new root field builder for parsing fields from the given
    /// `regex::Captures` object.
    fn root(line: &'a str, captures: Captures<'a>) -> Self {
        let len = captures.len();

        FieldBuilder {
            line,
            captures: Rc::new(captures),
            field: None,
            parent: None,
            values: HashMap::with_capacity(len),
        }
    }

    /// Create a nested field builder object for parsing fields from the
    /// given `regex::Captures` object and parent builder that control will
    /// be returned to when `.complete_mapping()` is called.
    fn leaf(line: &'a str, captures: Rc<Captures<'a>>, field: String, parent: Box<FieldBuilder<'a>>) -> Self {
        FieldBuilder {
            line,
            captures,
            field: Some(field),
            parent: Some(parent),
            values: HashMap::new(),
        }
    }

    /// Parse the text value in position `index` and output the field
    /// using the given name. Return an error if the value could not be
    /// parsed.
    fn add_text_field<S>(mut self, field: S, index: usize) -> RedeyeResult<Self>
    where
        S: Into<String>,
    {
        let res = parse_text_value(&self.captures, index, self.line)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }

    /// Parse the timestamp value in position `index` and output the field
    /// using the given name. Return an error if the value could not be parsed.
    fn add_timestamp_field<S>(mut self, field: S, index: usize, format: &str) -> RedeyeResult<Self>
    where
        S: Into<String>,
    {
        let res = parse_timestamp(&self.captures, index, self.line, format)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }

    /// Parse the integer value in position `index` and output the field
    /// using the given name. Return an error if the value could not be parsed.
    fn add_int_field<S>(mut self, field: S, index: usize) -> RedeyeResult<Self>
    where
        S: Into<String>,
    {
        let res = parse_int_value(&self.captures, index, self.line)?;
        if let Some(v) = res {
            self.values.insert(field.into(), v);
        }

        Ok(self)
    }

    /// Add a literal string value and output the field using the given name.
    fn add_fixed_value<K, V>(mut self, field: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.values.insert(field.into(), LogFieldValue::Text(value.into()));
        self
    }

    /// Return a new `FieldBuilder` that will be used to construct a nested
    /// mapping value and will be output using the given name. Note that callers
    /// must also make a corresponding call to `.complete_mapping()` after adding
    /// all desired values to the nested mapping.
    fn add_mapping_field<S>(self, field: S) -> Self
    where
        S: Into<String>,
    {
        let parent = Box::new(self);
        FieldBuilder::leaf(parent.line, parent.captures.clone(), field.into(), parent)
    }

    /// Complete adding fields to a nested mapping value and return the original
    /// `FieldBuilder` instance to continue working on the previous set of fields.
    fn complete_mapping(self) -> Self {
        // Unwraps are OK here because if we're calling this method when not building
        // a nested mapping, that's a bug completely within our control and panicking
        // is the most obvious way to handle it.
        let mut parent = self.parent.unwrap();
        if !self.values.is_empty() {
            parent
                .values
                .insert(self.field.unwrap(), LogFieldValue::Mapping(self.values));
        }

        *parent
    }

    /// Complete parsing and build fields and return a `HashMap` of the values.
    fn build(self) -> HashMap<String, LogFieldValue> {
        self.values
    }
}

/// Parse the regex capture identified by `index into a timestamp with
/// a fixed offset.
///
/// Return an error if the capture was missing (the field didn't exist
/// at all, which is not the same as being empty, aka `-`) or the field
/// could not be parsed into a timestamp. Return `Ok(None)` if the field
/// exists but contains an empty value (`-`).
fn parse_timestamp(matches: &Captures, index: usize, line: &str, format: &str) -> RedeyeResult<Option<LogFieldValue>> {
    let field_match = matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .map(|m| m.as_str())
        .map(empty_field)?;

    if let Some(v) = field_match {
        Ok(Some(LogFieldValue::Timestamp(DateTime::parse_from_str(v, format)?)))
    } else {
        Ok(None)
    }
}

/// Parse the regex capture identified by `index` into a string value.
///
/// Return an error if the capture was missing (the field didn't exist
/// at all, which is not the same as being empty, aka `-`). Return
/// `Ok(None)` if the field exists but contains an empty value (`-`).
fn parse_text_value(matches: &Captures, index: usize, line: &str) -> RedeyeResult<Option<LogFieldValue>> {
    matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .map(|m| m.as_str())
        .map(empty_field)
        .map(|o| o.map(|s| LogFieldValue::Text(s.to_string())))
}

/// Parse the regex capture identified by `index` into an integer value.
///
/// Return an error if the capture was missing (the field didn't exist
/// at all, which is not the same as being empty, aka `-`) or the field
/// could not be parsed into an integer. Return `Ok(None)` if the field
/// exists but contains an empty value (`-`).
fn parse_int_value(matches: &Captures, index: usize, line: &str) -> RedeyeResult<Option<LogFieldValue>> {
    let field_match = matches
        .get(index)
        .ok_or_else(|| RedeyeError::ParseError(line.to_string()))
        .map(|m| m.as_str())
        .map(empty_field)?;

    if let Some(v) = field_match {
        let val = v
            .parse::<u64>()
            .map_err(|_| RedeyeError::ParseError(line.to_string()))?;
        Ok(Some(LogFieldValue::Int(val)))
    } else {
        Ok(None)
    }
}

/// Convert the "-" character that represents empty fields
fn empty_field(val: &str) -> Option<&str> {
    if val == "-" {
        None
    } else {
        Some(val)
    }
}

#[cfg(test)]
mod tests {

    use super::{parse_int_value, parse_text_value, parse_timestamp, COMMON_LOG_TIMESTAMP};
    use chrono::{Datelike, Timelike};
    use regex::{Captures, Regex};
    use types::{LogFieldValue, RedeyeError};

    fn single_val_capture<'a>(line: &'a str) -> Captures<'a> {
        let r = Regex::new(r"^(.+)$").unwrap();
        r.captures(line).unwrap()
    }

    #[test]
    fn test_parse_timestamp_missing() {
        let line = "127.0.0.1";
        let c = single_val_capture(line);
        let res = parse_timestamp(&c, 2 /* shouldn't exist */, line, COMMON_LOG_TIMESTAMP);

        match res {
            Err(RedeyeError::ParseError(_)) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_timestamp_empty_field() {
        let line = "-";
        let c = single_val_capture(line);
        let res = parse_timestamp(&c, 1, line, COMMON_LOG_TIMESTAMP);

        match res {
            Ok(None) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_timestamp_bad_format() {
        let line = "asdf";
        let c = single_val_capture(line);
        let res = parse_timestamp(&c, 1, line, COMMON_LOG_TIMESTAMP);

        match res {
            Err(RedeyeError::TimestampParseError(_)) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_timestamp_success() {
        let line = "11/Oct/2000:13:55:36 -0700";
        let c = single_val_capture(line);
        let res = parse_timestamp(&c, 1, line, COMMON_LOG_TIMESTAMP);

        match res {
            Ok(Some(LogFieldValue::Timestamp(ts))) => {
                assert_eq!(2000, ts.year());
                assert_eq!(10, ts.month());
                assert_eq!(11, ts.day());
                assert_eq!(13, ts.hour());
                assert_eq!(55, ts.minute());
                assert_eq!(36, ts.second());
                assert_eq!(-7 * 3600, ts.offset().local_minus_utc());
            }
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_text_value_missing() {
        let line = "127.0.0.1";
        let c = single_val_capture(line);
        let res = parse_text_value(&c, 2 /* shouldn't exist */, line);

        match res {
            Err(RedeyeError::ParseError(_)) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_text_value_empty_field() {
        let line = "-";
        let c = single_val_capture(line);
        let res = parse_text_value(&c, 1, line);

        match res {
            Ok(None) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_text_value_success() {
        let line = "127.0.0.1";
        let c = single_val_capture(line);
        let res = parse_text_value(&c, 1, line);

        match res {
            Ok(Some(LogFieldValue::Text(s))) => {
                assert_eq!("127.0.0.1".to_owned(), s);
            }
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_int_value_missing() {
        let line = "200";
        let c = single_val_capture(line);
        let res = parse_int_value(&c, 2 /* shouldn't exist */, line);

        match res {
            Err(RedeyeError::ParseError(_)) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_int_value_empty_field() {
        let line = "-";
        let c = single_val_capture(line);
        let res = parse_int_value(&c, 1, line);

        match res {
            Ok(None) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_int_value_bad_format() {
        let line = "asdf";
        let c = single_val_capture(line);
        let res = parse_int_value(&c, 1, line);

        match res {
            Err(RedeyeError::ParseError(_)) => (),
            v => panic!("Unexpected result: {:?}", v),
        }
    }

    #[test]
    fn test_parse_int_value_success() {
        let line = "404";
        let c = single_val_capture(line);
        let res = parse_int_value(&c, 1, line);

        match res {
            Ok(Some(LogFieldValue::Int(v))) => {
                assert_eq!(404, v);
            }
            v => panic!("Unexpected result: {:?}", v),
        }
    }
}
