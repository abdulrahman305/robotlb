use std::{collections::BTreeMap, str::FromStr};

use crate::error::LBTrackerError;

#[derive(Debug, Clone)]
enum Rule {
    Equal(String, String),
    NotEqual(String, String),
    Exists(String),
    DoesNotExist(String),
}

#[derive(Debug, Clone, Default)]
pub struct LabelFilter {
    rules: Vec<Rule>,
}

impl LabelFilter {
    #[must_use] pub fn check(&self, labels: &BTreeMap<String, String>) -> bool {
        for rule in &self.rules {
            match rule {
                Rule::Equal(key, value) => {
                    if labels.get(key) != Some(value) {
                        return false;
                    }
                }
                Rule::NotEqual(key, value) => {
                    if labels.get(key) == Some(value) {
                        return false;
                    }
                }
                Rule::Exists(key) => {
                    if labels.get(key).is_none() {
                        return false;
                    }
                }
                Rule::DoesNotExist(key) => {
                    if labels.get(key).is_some() {
                        return false;
                    }
                }
            }
        }
        true
    }
}

impl FromStr for LabelFilter {
    type Err = LBTrackerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut rules = Vec::new();
        for rule in s.split(',') {
            let parts = rule.split('=').collect::<Vec<_>>();
            match *parts.as_slice() {
                [key] => {
                    if key.starts_with('!') {
                        rules.push(Rule::DoesNotExist(
                            key.strip_prefix('!').unwrap().to_string(),
                        ));
                        continue;
                    }
                    rules.push(Rule::Exists(key.to_string()));
                }
                [key, value] => {
                    if key.ends_with('!') {
                        rules.push(Rule::NotEqual(
                            key.strip_suffix('!').unwrap().to_string(),
                            value.to_string(),
                        ));
                        continue;
                    }
                    rules.push(Rule::Equal(key.to_string(), value.to_string()));
                }
                _ => return Err(LBTrackerError::InvalidNodeFilter(rule.to_string())),
            }
        }
        Ok(Self { rules })
    }
}
