//
//
//

//!

use chrono::{format, DateTime, FixedOffset};
use serde::{Serialize, Serializer};
use serde_json::error::Error as SerdeError;
use std::collections::HashMap;
use std::io;

pub type RedeyeResult<T> = Result<T, RedeyeError>;

#[derive(Fail, Debug)]
pub enum RedeyeError {
    #[fail(display = "{}", _0)]
    IoError(#[cause] io::Error),

    #[fail(display = "{}", _0)]
    SerializationError(#[cause] SerdeError),

    #[fail(display = "{}", _0)]
    TimestampParseError(#[cause] format::ParseError),

    #[fail(display = "Could not parse: {}", _0)]
    ParseError(String),

    #[fail(display = "An unknown error occurred")]
    Unknown,
}

impl From<io::Error> for RedeyeError {
    fn from(e: io::Error) -> Self {
        RedeyeError::IoError(e)
    }
}

impl From<SerdeError> for RedeyeError {
    fn from(e: SerdeError) -> Self {
        RedeyeError::SerializationError(e)
    }
}

impl From<format::ParseError> for RedeyeError {
    fn from(e: format::ParseError) -> Self {
        RedeyeError::TimestampParseError(e)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogFieldValue {
    Mapping(HashMap<String, LogFieldValue>),
    Timestamp(DateTime<FixedOffset>),
    Text(String),
    Int(u64),
}

impl Serialize for LogFieldValue {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        match *self {
            LogFieldValue::Mapping(ref map) => map.serialize(serializer),
            LogFieldValue::Timestamp(ref val) => serializer.serialize_str(&val.to_rfc3339()),
            LogFieldValue::Text(ref val) => serializer.serialize_str(val),
            LogFieldValue::Int(val) => serializer.serialize_u64(val),
        }
    }
}

#[derive(Debug, Default)]
pub struct LogEvent {
    values: HashMap<String, LogFieldValue>,
}

impl Serialize for LogEvent {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        self.values.serialize(serializer)
    }
}

impl From<HashMap<String, LogFieldValue>> for LogEvent {
    fn from(values: HashMap<String, LogFieldValue>) -> Self {
        Self { values }
    }
}
