mod channel_pattern;
use regex::Regex;
use std::{fmt::Display, path::PathBuf};

pub use channel_pattern::{parse_pattern, ChannelPattern};
use chrono::prelude::*;

use lib_device::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub enum LoggerType {
    DataBase,
    TextFile,
}
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Logger {
    pub name: String,
    pub logger_type: LoggerType,
    pub channels: Vec<LoggerChannel>,
    pub path: PathBuf,
    pub log_rate: usize,
    pub is_logging: bool,
}

impl Default for LoggerType {
    fn default() -> Self {
        LoggerType::TextFile
    }
}
impl Display for LoggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerType::DataBase => write!(f, "Database"),
            LoggerType::TextFile => write!(f, "Text File"),
        }
    }
}
impl Logger {
    pub fn new(
        name: String,
        logger_type: LoggerType,
        channel_pattern: &mut ChannelPattern,
        path: PathBuf,
        log_rate: usize,
        is_logging: bool,
        re: (&Regex, &Regex),
    ) -> anyhow::Result<Self> {
        let channels = parse_pattern(channel_pattern, re)?;

        let logger = Self {
            name,
            logger_type,
            channels,
            path,
            log_rate,
            is_logging,
        };

        Ok(logger)
    }
}
