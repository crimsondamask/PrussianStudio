use anyhow::{bail, Result};
use lib_device::*;
use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct ChannelPattern {
    pub pattern: String,
}

impl ChannelPattern {
    pub fn from_str(pattern: String) -> Self {
        Self { pattern }
    }
}

impl Default for ChannelPattern {
    fn default() -> Self {
        Self {
            pattern: String::new(),
        }
    }
}
pub fn parse_pattern(
    channel_pattern: &ChannelPattern,
    re: (&Regex, &Regex),
) -> Result<Vec<LoggerChannel>> {
    // Remove all whitespaces.
    let pattern: &str = &channel_pattern.pattern;

    if pattern == "" {
        bail!("Pattern string is empty!")
    }
    // Now we split the string on commas.
    let pattern_list: Vec<&str> = channel_pattern.pattern.split(",").collect();
    if pattern_list.is_empty() {
        bail!("No channel pattern found!")
    }

    let mut channel_index_vec: Vec<usize> = Vec::new();
    let mut calc_index_vec: Vec<usize> = Vec::new();
    for pattern in &pattern_list {
        let range: Vec<&str> = pattern.split("-").collect();
        if range.len() == 2 {
            if (range[0].replace(" ", "").is_empty()) || (range[1].replace(" ", "").is_empty()) {
                continue;
            }
            let mut begin_id = None;
            let mut end_id = None;
            let begin = &range[0];
            let end = &range[1];
            for cap in re.0.captures_iter(begin) {
                if let Ok(id) = cap[1].parse::<usize>() {
                    begin_id = Some(id);
                }
            }
            for cap in re.0.captures_iter(end) {
                if let Ok(id) = cap[1].parse::<usize>() {
                    end_id = Some(id);
                }
            }
            if let (Some(begin_id), Some(end_id)) = (begin_id, end_id) {
                if begin_id > end_id {
                    continue;
                }
                for id in begin_id..(end_id + 1) {
                    channel_index_vec.push(id);
                }
            }
        } else {
            continue;
        }
    }
    for pattern in &pattern_list {
        let range: Vec<&str> = pattern.split("-").collect();
        if range.len() == 2 {
            if (range[0].replace(" ", "").is_empty()) || (range[1].replace(" ", "").is_empty()) {
                continue;
            }
            let mut begin_id = None;
            let mut end_id = None;
            let begin = &range[0];
            let end = &range[1];
            for cap in re.1.captures_iter(begin) {
                if let Ok(id) = cap[1].parse::<usize>() {
                    begin_id = Some(id);
                }
            }
            for cap in re.1.captures_iter(end) {
                if let Ok(id) = cap[1].parse::<usize>() {
                    end_id = Some(id);
                }
            }
            if let (Some(begin_id), Some(end_id)) = (begin_id, end_id) {
                if begin_id > end_id {
                    continue;
                }
                for id in begin_id..(end_id + 1) {
                    calc_index_vec.push(id);
                }
            }
        } else {
            continue;
        }
    }

    let mut channel_vec = construct_channel_vec(channel_index_vec);
    let mut calc_vec = construct_calc_vec(calc_index_vec);
    channel_vec.append(&mut calc_vec);
    Ok(channel_vec)
}

fn construct_channel_vec(channel_index_vec: Vec<usize>) -> Vec<LoggerChannel> {
    let mut channel_vec: Vec<LoggerChannel> = Vec::new();
    for id in channel_index_vec {
        let channel = Channel {
            id,
            ..Default::default()
        };

        channel_vec.push(LoggerChannel::Channel(channel));
    }
    channel_vec
}
fn construct_calc_vec(calc_index_vec: Vec<usize>) -> Vec<LoggerChannel> {
    let mut calc_vec: Vec<LoggerChannel> = Vec::new();
    for id in calc_index_vec {
        let calculation = Calculation {
            id,
            ..Default::default()
        };

        calc_vec.push(LoggerChannel::Calculation(calculation));
    }
    calc_vec
}

#[cfg(test)]
mod tests {
    use lib_device::LoggerChannel;

    use super::parse_pattern;
    use super::ChannelPattern;
    use super::Regex;

    #[test]
    fn parse_pattern_test() {
        let mut channel_pattern = ChannelPattern {
            pattern: "CH4-CH7, CH1  -CH2, CH4-CH2, CH1-CH1, CH1-CH2, EVAL1-     CH4 , EVAL1-EVAL3"
                .to_owned(),
        };
        let re_channel = Regex::new(r"CH+(?:([0-9]+))").unwrap();
        let re_calc = Regex::new(r"EVAL+(?:([0-9]+))").unwrap();
        let ids = parse_pattern(&mut channel_pattern, (&re_channel, &re_calc));
        let mut id_list = Vec::new();
        for id in &ids.unwrap() {
            match id {
                LoggerChannel::Channel(channel) => id_list.push(channel.id),
                LoggerChannel::Calculation(calculation) => id_list.push(calculation.id),
            }
        }
        assert_eq!(id_list, [4, 5, 6, 7, 1, 2, 1, 1, 2, 1, 2, 3]);
    }
}
