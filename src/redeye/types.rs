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

//! Core types and errors of the library

use chrono::{format, DateTime, FixedOffset};
use serde::{Serialize, Serializer};
use serde_json::error::Error as SerdeError;
use std::collections::HashMap;
use std::io;

pub type RedeyeResult<T> = Result<T, RedeyeError>;

/// Possible error that may occur while parsing and emitting access logs.
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

/// Possible types of values for a single log field.
///
/// Values may be nested arbitrarily deep by using the `Mapping` variant.
/// This is typically used for groups of values like request or response
/// headers.
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

/// Holder for values parsed from a single log line.
///
/// Most of the values will correspond to a field parsed from the incoming
/// access log line. The names of the fields are picked to be compatible
/// with the format expected by Logstash consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogEvent {
    values: HashMap<String, LogFieldValue>,
}

impl LogEvent {
    pub fn fields(&self) -> &HashMap<String, LogFieldValue> {
        &self.values
    }
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
