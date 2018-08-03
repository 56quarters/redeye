//
//
//

use types::RedeyeResult;

pub trait LogLineParser {
    fn parse(&self, line: &str) -> RedeyeResult<LogEvent>;
}

pub struct LogEvent {}

pub struct CommonLogLineParser {}

pub struct ExtendedLogLineParser {}
