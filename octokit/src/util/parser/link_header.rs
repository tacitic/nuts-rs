use crate::util::pagination::LinkHeader;
use nom::bytes::complete::tag;
use nom::character::complete::alphanumeric1;
use reqwest::Url;
use std::str::{from_utf8, FromStr};

/// Parses a Link HTTP Header
/// Link headers look like this:
///     <https://api.github.com/x/y/releases?page=1&per_page=5>; rel="next"
pub(crate) fn link_header(s: &str) -> Result<LinkHeader, String> {
    match parse_link_header(s.as_bytes()) {
        Ok((_, l)) => Ok(l),
        Err(_) => Err("could not parse link-header".to_string()),
    }
}

named_attr!(#[doc="Parses an encapsulated <url> in a link-header"],
    parse_url<&[u8], Url>, do_parse!(
        ws!(tag!("<")) >>
        url: ws!(take_until!(">")) >>
        ws!(tag!(">")) >>
        ((Url::parse(from_utf8(url).unwrap()).unwrap()))
    ));

named_attr!(#[doc="Parses a single link-header argument"],
    parse_argument<&[u8], (&str, &str)>, do_parse!(
        key: alphanumeric1 >>
        tag!("=") >>
        val: delimited!(tag("\""), alphanumeric1, tag("\"")) >>
        ((from_utf8(key).unwrap(), from_utf8(val).unwrap()))
    ));

named_attr!(#[doc="Parser a link-header"],
    parse_link_header<&[u8], LinkHeader>, do_parse!(
        url: parse_url >>
        ws!(tag!(";")) >>
        args: separated_list!(ws!(tag(";")), parse_argument) >>
        ((LinkHeader {
            page: get_param(&url, "page").unwrap(),
            per_page: get_param(&url, "per_page").unwrap(),
            rel: get_arg(args, "rel").unwrap(),
            url: url,
        }))
    ));

fn get_param<T: FromStr>(url: &Url, key: &str) -> Option<T> {
    for (k, val) in url.query_pairs() {
        if key == k {
            return val.parse().ok();
        }
    }
    return None;
}

fn get_arg<T: FromStr>(args: Vec<(&str, &str)>, key: &str) -> Option<T> {
    for (k, val) in args {
        if key == k {
            return val.parse().ok();
        }
    }
    return None;
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::pagination::LinkHeaderType;

    #[test]
    fn test_parse_argument() {
        let arg = &b"rel=\"thing\""[..];
        let (_, res) = parse_argument(arg).unwrap();
        assert_eq!(res, ("rel", "thing"));
    }

    #[test]
    fn test_parse_url() {
        let link = &b"<https://api.github.com/repositories/x/releases?page=1&per_page=5>"[..];
        let (_, res) = parse_url(link).unwrap();
        assert_eq!(
            Url::parse("https://api.github.com/repositories/x/releases?page=1&per_page=5").unwrap(),
            res
        );
    }

    #[test]
    fn test_parse_header_link() {
        let links = vec![(
            "<https://api.github.com/repositories/x/releases?page=1&per_page=5>; rel=\"first\"",
            LinkHeader {
                url: Url::parse("https://api.github.com/repositories/x/releases?page=1&per_page=5")
                    .unwrap(),
                page: 1,
                per_page: 5,
                rel: LinkHeaderType::First,
            },
        ), (
            "<https://api.github.com/repositories/x/releases?page=2&per_page=5>; test=\"raar\"; rel=\"next\"",
            LinkHeader {
                url: Url::parse("https://api.github.com/repositories/x/releases?page=2&per_page=5")
                    .unwrap(),
                page: 2,
                per_page: 5,
                rel: LinkHeaderType::Next,
            }
            )];

        for (link, expect) in links {
            let (_, res) = parse_link_header(link.as_bytes()).unwrap();
            assert_eq!(res, expect);
        }
    }
}
