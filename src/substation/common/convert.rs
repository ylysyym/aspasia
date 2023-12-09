use std::fmt::Write;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::char,
    combinator::{eof, map, rest, value},
    multi::{many1, many_till},
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

// Split a sequence of {\b1\i1} into {\b1}{\i1}
fn split_tag(input: &str) -> IResult<&str, String> {
    delimited(char('{'), take_until("}"), char('}'))
        .and_then(map(
            many1(preceded(char('\\'), alt((take_until("\\"), rest)))),
            |v| {
                v.iter().fold(String::new(), |mut res, s| {
                    write!(res, "{{\\{s}}}").unwrap();
                    res
                })
            },
        ))
        .parse(input)
}

pub(crate) fn split_formatting_tags(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                split_tag,
                map(take_until("{"), |s: &str| s.to_string()),
                map(rest, |s: &str| s.to_string()),
            )),
            eof,
        ),
        |(s, _)| s.join(""),
    )
    .parse(input)
}

pub(crate) fn discard_tag(input: &str) -> IResult<&str, &str> {
    value("", tuple((char('{'), take_until("}"), char('}')))).parse(input)
}

fn convert_hex(input: &str) -> String {
    let s = format!("{input:06}");
    let mut result = String::new();
    result.push_str(&s[4..6]);
    result.push_str(&s[2..4]);
    result.push_str(&s[0..2]);

    result
}

pub(crate) fn convert_font_color_tag(input: &str) -> IResult<&str, String> {
    alt((
        map(
            delimited(tag("{\\1c&H"), take_until("&}"), tag("&}")),
            |s| format!("<font color=\"#{}\">", convert_hex(s)),
        ),
        map(delimited(tag("{\\c&H"), take_until("&}"), tag("&}")), |s| {
            format!("<font color=\"#{}\">", convert_hex(s))
        }),
    ))
    .parse(input)
}
