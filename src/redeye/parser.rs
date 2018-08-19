//
//
//

//!

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
    /// using the given name. Return an error if the value could be parsed.
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
