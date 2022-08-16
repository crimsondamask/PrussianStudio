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
pub fn parse_pattern(channel_pattern: &ChannelPattern, re: &Regex) -> Result<Vec<Channel>> {
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
    for pattern in &pattern_list {
        let range: Vec<&str> = pattern.split("-").collect();
        if range.len() == 2 {
            let mut begin_id = 0;
            let mut end_id = 0;
            let begin = &range[0];
            let end = &range[1];
            for cap in re.captures_iter(begin) {
                if let Ok(id) = cap[1].parse::<usize>() {
                    begin_id = id;
                }
            }
            for cap in re.captures_iter(end) {
                if let Ok(id) = cap[1].parse::<usize>() {
                    end_id = id;
                }
            }
            if begin_id > end_id {
                continue;
            }
            for id in begin_id..(end_id + 1) {
                channel_index_vec.push(id);
            }
        } else {
            continue;
        }
    }

    Ok(construct_channel_vec(channel_index_vec))
}

fn construct_channel_vec(channel_index_vec: Vec<usize>) -> Vec<Channel> {
    let mut channel_vec: Vec<Channel> = Vec::new();
    for id in channel_index_vec {
        let channel = Channel {
            id,
            ..Default::default()
        };

        channel_vec.push(channel);
    }
    channel_vec
}

#[cfg(test)]
mod tests {
    use super::parse_pattern;
    use super::ChannelPattern;
    use super::Regex;

    #[test]
    fn parse_pattern_test() {
        let mut channel_pattern = ChannelPattern {
            pattern: "CH4-CH7, CH1  -CH2, CH4-CH2, CH1-CH1".to_owned(),
        };
        let re = Regex::new(r"CH+(?:([0-9]+))").unwrap();
        let ids = parse_pattern(&mut channel_pattern, &re);
        let mut id_list = Vec::new();
        for id in &ids.unwrap() {
            id_list.push(id.id);
        }
        assert_eq!(id_list, [4, 5, 6, 7, 1, 2, 1]);
    }
}
