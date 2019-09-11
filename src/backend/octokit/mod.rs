mod parser;

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

pub struct LinkHeader {
    url: String,
    page: u32,
    per_page: u32,
    rel: LinkHeaderType,
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
