use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{eof, map, rest, value},
    multi::many_till,
    IResult, Parser,
};

use crate::parsing::{discard, html_tag};

fn convert_to_ass_tag(input: &str) -> IResult<&str, &str> {
    alt((
        value("{\\b1}", tag("<b>")),
        value("{\\b0}", tag("</b>")),
        value("{\\i1}", tag("<i>")),
        value("{\\i0}", tag("</i>")),
        value("{\\u1}", tag("<u>")),
        value("{\\u0}", tag("</u>")),
    ))
    .parse(input)
}

fn parse_webvtt_tags(input: &str) -> IResult<&str, &str> {
    alt((
        tag("<b>"),
        tag("</b>"),
        tag("<i>"),
        tag("</i>"),
        tag("<u>"),
        tag("</u>"),
    ))
    .parse(input)
}

pub(crate) fn vtt_to_ass_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((convert_to_ass_tag, discard(html_tag), take_until("<"), rest)),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

pub(crate) fn vtt_to_srt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((parse_webvtt_tags, discard(html_tag), take_until("<"), rest)),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

pub(crate) fn strip_html_tags(input: &str) -> IResult<&str, String> {
    map(
        many_till(alt((discard(html_tag), take_until("<"), rest)), eof),
        |(s, _)| s.join("").to_string(),
    )
    .parse(input)
}
