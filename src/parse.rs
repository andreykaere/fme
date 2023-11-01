use anyhow::bail;
use std::str::FromStr;

use crate::metadata::Metadata;

const TOKEN_VALUES: [&str; 5] = ["{a}", "{t}", "{n}", "{m}", "{y}"];

#[derive(Debug, Clone, PartialEq)]
pub struct ParsePattern {
    args: Vec<ArgumentPattern>,
}

impl ParsePattern {
    fn new(args: Vec<ArgumentPattern>) -> Self {
        Self { args }
    }

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

fn keep_split<'a>(input: &'a str, token: &'a str) -> Vec<&'a str> {
    itertools::intersperse(input.split(token), token)
        .filter(|x| !x.is_empty())
        .collect()
}

impl FromStr for ParsePattern {
    type Err = anyhow::Error;

    // TODO: to add error support
    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split = vec![string];

        for token in TOKEN_VALUES {
            split = split.iter().fold(Vec::new(), |acc, x| {
                [acc, keep_split(x, token)].concat()
            });
        }

        println!("{:?}", split);

        let args = split
            .iter()
            .map(|x| ArgumentPattern::from_str(x))
            .filter_map(|x| x.ok())
            .collect();

        Ok(ParsePattern::new(args))
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ArgumentPattern {
    Text(String),
    Token(Token),
}

impl FromStr for ArgumentPattern {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match Token::from_str(string) {
            Ok(x) => Ok(ArgumentPattern::Token(x)),
            Err(_) => Ok(ArgumentPattern::Text(string.to_string())),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Artist,
    Title,
    Album,
    Year,
    Track,
}

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let token = match string {
            "{a}" => Token::Artist,
            "{t}" => Token::Title,
            "{m}" => Token::Album,
            "{y}" => Token::Year,
            "{n}" => Token::Track,
            _ => bail!("Uknown token"),
        };

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_from_str() {
        assert_eq!(
            ParsePattern::from_str("{n} {a} - {t}").unwrap(),
            ParsePattern::new(
                [
                    ArgumentPattern::Token(Token::Track),
                    ArgumentPattern::Text(" ".to_string()),
                    ArgumentPattern::Token(Token::Artist),
                    ArgumentPattern::Text(" - ".to_string()),
                    ArgumentPattern::Token(Token::Title)
                ]
                .to_vec()
            )
        );
        // ["{n}", " ", "{a}", " - ", "{t}"]

        // let patterns = [
        //     ,
        //     "{n} {a} — {t}",
        //     "{n}. {a} - {t}",
        //     "{n}. {a} — {t}",
        //     "{a} - {n} {t}",
        //     "{a} — {n} {t}",
        //     "{a} - {n}. {t}",
        //     "{a} — {n}. {t}",
        //     "{a} - {t}",
        //     "{a} — {t}",
        //     "{n} {t}",
        //     "{n}. {t}",
        //     "{t}",
        // ];
    }
}
