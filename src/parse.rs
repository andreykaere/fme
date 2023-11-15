use anyhow::{bail, Context};
use regex::Regex;
use std::str::FromStr;

use crate::metadata::Metadata;

const TOKEN_VALUES: [&str; 5] = ["{a}", "{t}", "{d}", "{m}", "{y}"];

#[derive(Debug, Clone, PartialEq)]
pub struct ParsePattern {
    items: Vec<ItemPattern>,
}

impl ParsePattern {
    fn new(items: Vec<ItemPattern>) -> Self {
        Self { items }
    }

    pub fn default_patterns() -> Vec<Self> {
        let patterns = [
            "{d} {a} - {t}",
            "{d} {a} — {t}",
            "{d}. {a} - {t}",
            "{d}. {a} — {t}",
            "{a} - {d} {t}",
            "{a} — {d} {t}",
            "{a} - {d}. {t}",
            "{a} — {d}. {t}",
            "{a} - {t}",
            "{a} — {t}",
            "{d} {t}",
            "{d}. {t}",
            "{t}",
        ];

        patterns
            .iter()
            .map(|x| ParsePattern::from_str(x).unwrap())
            .collect()
    }

    pub fn try_pattern(&self, input: &str) -> anyhow::Result<Metadata> {
        let mut metadata = Metadata::default();

        let mut regex_str = self
            .items
            .iter()
            .map(|x| match x {
                ItemPattern::Text(s) => regex::escape(s),
                ItemPattern::Token(t) => t.token_to_regex_repr(),
            })
            .collect::<String>();

        regex_str.push('$');
        regex_str.insert(0, '^');

        let regex = Regex::new(&regex_str).unwrap();
        let groups = regex
            .captures(input)
            .context("Failed to parse given string using this pattern")?;

        let tokens: Vec<_> = self
            .items
            .iter()
            .filter_map(|x| {
                if let ItemPattern::Token(token) = x {
                    Some(token)
                } else {
                    None
                }
            })
            .collect();

        for (i, token) in tokens.iter().enumerate() {
            let value = groups.get(i + 1).unwrap().as_str();
            token.apply_token(value, &mut metadata)?;
        }

        Ok(metadata)
    }
}

impl FromStr for ParsePattern {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let mut split = vec![string];

        for token in TOKEN_VALUES {
            split = split.iter().fold(Vec::new(), |mut acc, x| {
                acc.append(&mut keep_split(x, token));
                acc
            });
        }

        let mut items = Vec::new();

        for s in split {
            match ItemPattern::from_str(s) {
                Ok(s) => items.push(s),
                Err(e) => bail!("{e}"),
            }
        }

        Ok(ParsePattern::new(items))
    }
}

fn keep_split<'a>(input: &'a str, token: &'a str) -> Vec<&'a str> {
    itertools::intersperse(input.split(token), token)
        .filter(|x| !x.is_empty())
        .collect()
}


#[derive(Debug, Clone, PartialEq)]
enum ItemPattern {
    Text(String),
    Token(Token),
}

impl FromStr for ItemPattern {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        match Token::from_str(string) {
            Ok(x) => Ok(ItemPattern::Token(x)),
            Err(_) => Ok(ItemPattern::Text(string.to_string())),
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

impl Token {
    fn token_to_regex_repr(&self) -> String {
        // let regex_text = r"([a-zA-Z0-9&'\s\{\}\[\]\(\)_\.,\*]+)";
        let regex_text = r"(.+?)";
        let regex_num = r"([0-9]+)";

        let regex_repr = match self {
            Token::Artist | Token::Title | Token::Album => regex_text,
            Token::Year | Token::Track => regex_num,
        };

        regex_repr.to_string()
    }

    fn apply_token(
        &self,
        value: &str,
        metadata: &mut Metadata,
    ) -> anyhow::Result<()> {
        // println!("token: {:?} and value: {value}", self);

        match self {
            Token::Artist => metadata.artist = Some(value.parse()?),
            Token::Title => metadata.title = Some(value.parse()?),
            Token::Album => metadata.album_title = Some(value.parse()?),
            Token::Year => metadata.year = Some(value.parse()?),
            Token::Track => metadata.track_number = Some(value.parse()?),
        }

        Ok(())
    }
}

impl FromStr for Token {
    type Err = anyhow::Error;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        let token = match string {
            "{a}" => Token::Artist,
            "{t}" => Token::Title,
            "{m}" => Token::Album,
            "{y}" => Token::Year,
            "{d}" => Token::Track,
            _ => bail!("Unknown token"),
        };

        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_from_str() {
        assert_eq!(
            ParsePattern::from_str("{d} {a} - {t}").unwrap(),
            ParsePattern::new(
                [
                    ItemPattern::Token(Token::Track),
                    ItemPattern::Text(" ".to_string()),
                    ItemPattern::Token(Token::Artist),
                    ItemPattern::Text(" - ".to_string()),
                    ItemPattern::Token(Token::Title)
                ]
                .to_vec()
            )
        );

        assert_eq!(
            ParsePattern::from_str("{d}. {a} — {t}").unwrap(),
            ParsePattern::new(
                [
                    ItemPattern::Token(Token::Track),
                    ItemPattern::Text(". ".to_string()),
                    ItemPattern::Token(Token::Artist),
                    ItemPattern::Text(" — ".to_string()),
                    ItemPattern::Token(Token::Title)
                ]
                .to_vec()
            )
        );

        assert_eq!(
            ParsePattern::from_str("{a}{a} - {t}").unwrap(),
            ParsePattern::new(
                [
                    ItemPattern::Token(Token::Artist),
                    ItemPattern::Token(Token::Artist),
                    ItemPattern::Text(" - ".to_string()),
                    ItemPattern::Token(Token::Title)
                ]
                .to_vec()
            )
        );

        assert_eq!(
            ParsePattern::from_str("{t}").unwrap(),
            ParsePattern::new([ItemPattern::Token(Token::Title)].to_vec())
        );
    }

    #[test]
    fn test_patterns() {
        let pattern1 = ParsePattern::from_str("{d}. {a} - {t}").unwrap();
        let input1 = "12. Foo - Bar";
        let input1_ = "12. Foo & Bazz & Quuz vs Booz & Tooz - Bar (no {quuz})";
        // println!("{:?}", pattern1.try_pattern(input1));
        assert!(pattern1.try_pattern(input1).is_ok());
        assert!(pattern1.try_pattern(input1_).is_ok());

        // println!(
        //     "{}",
        //     regex::escape(r"([0-9]+)\. ([a-zA-Z0-9&']+) \- ([a-zA-Z0-9&']+)$")
        // );

        let pattern2 = ParsePattern::from_str("{d}. {t}").unwrap();
        let input2 = "12. Foo - Bar";
        // println!("{:?}", pattern2.try_pattern(input2));
        assert!(pattern2.try_pattern(input2).is_ok());
    }
}
