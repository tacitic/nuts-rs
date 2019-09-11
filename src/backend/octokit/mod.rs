use reqwest::Url;
use std::str::FromStr;

mod parser;

#[derive(Debug, PartialEq)]
pub enum LinkHeaderType {
    Prev,
    Next,
    First,
    Last,
    Unknown,
}

impl From<String> for LinkHeaderType {
    fn from(s: String) -> Self {
        match s.as_str() {
            "prev" => LinkHeaderType::Prev,
            "next" => LinkHeaderType::Next,
            "first" => LinkHeaderType::First,
            "last" => LinkHeaderType::Last,
            _ => LinkHeaderType::Unknown,
        }
    }
}

impl FromStr for LinkHeaderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(LinkHeaderType::from(s.to_string()))
    }
}

#[derive(Debug, PartialEq)]
pub struct LinkHeader {
    pub url: Url,
    pub page: u32,
    pub per_page: u32,
    pub rel: LinkHeaderType,
}

impl LinkHeader {
    fn parse(input: &str) -> Option<Self> {
        let mut s = input.clone();
        if s.starts_with("<") && s.ends_with(">") {
            s = &s[1..];
            s = &s[0..s.len() - 1];
        }

        unimplemented!()
    }
}
