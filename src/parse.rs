use anyhow::bail;
use std::str::FromStr;

use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub struct ParsePattern {
    args: Vec<ArgumentPattern>,
}

impl ParsePattern {
    pub fn default_patterns() -> Vec<Self> {
        let patterns = [
            "{n} {a} - {t}",
            "{n} {a} — {t}",
            "{n}. {a} - {t}",
            "{n}. {a} — {t}",
            "{a} - {n} {t}",
            "{a} — {n} {t}",
            "{a} - {n}. {t}",
            "{a} — {n}. {t}",
            "{a} - {t}",
            "{a} — {t}",
            "{n} {t}",
            "{n}. {t}",
            "{t}",
        ];

        patterns
            .iter()
            .map(|x| ParsePattern::from_str(x))
            .filter_map(|x| x.ok())
            .collect()
    }

    pub fn try_pattern(&self, input: &str) -> anyhow::Result<Metadata> {
        todo!();

        bail!("Failed to parse given string using this pattern");
    }
}

impl FromStr for ParsePattern {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        todo!();
    }
}

#[derive(Debug, Clone)]
enum ArgumentPattern {
    Text(String),
    Token(Token),
}

/// Correspondence between `Token` values and actual values that are supplied
/// to the program:
/// Artist <-> {a}
/// Title  <-> {t}
/// Album  <-> {m}
/// Year   <-> {y}
/// Track  <-> {n}
#[derive(Debug, Clone)]
enum Token {
    Artist,
    Title,
    Album,
    Year,
    Track,
}
