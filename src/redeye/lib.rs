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

//! Supporting library for the Redeye log parser.

#![forbid(unsafe_code)]

extern crate chrono;
extern crate failure;
#[macro_use]
extern crate failure_derive;
extern crate regex;
extern crate serde;
extern crate serde_json;
extern crate tokio;

pub mod io;
pub mod parser;
pub mod types;
