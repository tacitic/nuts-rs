use crate::{util, Config};
use reqwest::header::HeaderMap;
use reqwest::{RequestBuilder, Response, Url};
use std::str::FromStr;

// TODO: docs
pub fn paginate(
    req: &RequestBuilder,
    from: u32,
    per_page: u32,
) -> Result<Vec<Response>, reqwest::Error> {
    let mut pager = Some(from);
    let mut out = vec![];

    while let Some(page) = pager {
        let res = req
            .try_clone()
            .unwrap()
            .query(&[("page", page), ("per_page", per_page)])
            .send()?;

        let lh = LinkHeaders::new(res.headers());
        match LinkHeaders::new(res.headers()) {
            Some(lh) => {
                if lh.should_next(page) {
                    pager = Some(lh.get_next(page))
                } else {
                    pager = None;
                }
            }
            None => pager = None,
        }

        out.push(res);
    }

    return Ok(out);
}

#[derive(Debug, PartialEq)]
pub(crate) enum LinkHeaderType {
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
pub(crate) struct LinkHeader {
    pub url: Url,
    pub page: u32,
    pub per_page: u32,
    pub rel: LinkHeaderType,
}

impl LinkHeader {
    fn new(s: &str) -> Result<Self, String> {
        util::parser::link_header(s)
    }
}

#[derive(Debug)]
struct LinkHeaders {
    next: Option<LinkHeader>,
    prev: Option<LinkHeader>,
    first: Option<LinkHeader>,
    last: Option<LinkHeader>,
}

impl LinkHeaders {
    fn new(headers: &HeaderMap) -> Option<Self> {
        match headers.get("Link") {
            Some(raw_link_header) => {
                let mut out = LinkHeaders {
                    next: None,
                    prev: None,
                    first: None,
                    last: None,
                };

                for link in raw_link_header.to_str().unwrap().split(",") {
                    let lh = util::parser::link_header(&link).unwrap();
                    match lh.rel {
                        LinkHeaderType::Next => out.next = Some(lh),
                        LinkHeaderType::Prev => out.prev = Some(lh),
                        LinkHeaderType::First => out.first = Some(lh),
                        LinkHeaderType::Last => out.last = Some(lh),
                        _ => unimplemented!(),
                    }
                }

                Some(out)
            }
            None => None,
        }
    }

    fn should_next(&self, page: u32) -> bool {
        if let Some(next) = &self.next {
            return next.page != page;
        } else {
            return false;
        }
    }

    fn get_next(&self, current: u32) -> u32 {
        if let Some(n) = &self.next {
            return n.page;
        }
        return current;
    }
}
