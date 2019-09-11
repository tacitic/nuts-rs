use nom::branch::alt;
use nom::bytes::complete::{tag, take_until};
use nom::character::complete::anychar;
use nom::character::complete::{alphanumeric0, alphanumeric1};
use nom::multi::separated_list;
use nom::multi::{many0, many1};
use nom::sequence::{delimited, separated_pair};
use nom::IResult;

/// Parses a LinkHeader in the format of <https://api.github.com/repositories/x/releases?page=1&per_page=5>; rel="first">"
/// TODO(rharink): trim whitespaces between everything
pub fn link_header(b: &[u8]) -> IResult<&[u8], &str> {
    let (rem, _) = tag("<")(b)?;
    let (rem, url) = take_until(">")(rem)?;
    let (rem, _) = tag(">; ")(rem)?;
    let (rem, args) = separated_list(tag(";"), rel)(rem)?;
    let (rem, _) = tag(">")(rem)?;
    println!("{:?}, {:?}", args, std::str::from_utf8(rem).unwrap());

    Ok((rem, "raar"))
}

pub fn link_header2(b: &[u8]) -> IResult<&[u8], &str> {
    let (rem, items) = separated_list(alt((tag(";"), tag("; "))), take_until(";"))(b)?;
    println!("{:?}, {:?}", items, std::str::from_utf8(rem).unwrap());
    Ok((rem, "poep"))
}

pub fn rel(b: &[u8]) -> IResult<&[u8], (&str, &str)> {
    let (res, (k, v)) = separated_pair(
        alphanumeric1,
        tag("="),
        delimited(tag("\""), alphanumeric1, tag("\"")),
    )(b)?;
    Ok((
        res,
        (
            std::str::from_utf8(k).unwrap(),
            std::str::from_utf8(v).unwrap(),
        ),
    ))
}

#[cfg(test)]
mod test {
    use super::*;
    use nom::AsBytes;

    #[test]
    fn test_link_header() {
        let link =
            b"<https://api.github.com/repositories/x/releases?page=1&per_page=5>; rel=\"first\"; bla=\"something\"";

        let (_, res) = link_header2(link).unwrap();
        println!("{:?}", res);
    }
}
