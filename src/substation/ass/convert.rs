use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{anychar, one_of},
    combinator::{eof, map, rest, value},
    multi::many_till,
    sequence::{pair, tuple},
    IResult, Parser,
};

use crate::{
    parsing::{bracket_tag, discard},
    substation::common::convert::convert_font_color_tag,
};

pub(crate) fn ass_to_srt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                map(convert_to_html_tag, std::string::ToString::to_string),
                convert_font_color_tag,
                map(discard(bracket_tag), std::string::ToString::to_string),
                map(take_until("{"), |s: &str| s.to_string()),
                map(rest, |s: &str| s.to_string()),
            )),
            eof,
        ),
        |(v, _)| v.concat(),
    )
    .parse(input)
}

pub(crate) fn ass_to_vtt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                convert_to_html_tag,
                discard(bracket_tag),
                take_until("{"),
                rest,
            )),
            eof,
        ),
        |(v, _)| v.concat(),
    )
    .parse(input)
}

fn parse_drawing_end_tag(input: &str) -> IResult<&str, &str> {
    bracket_tag.and_then(take_until("\\p0")).parse(input)
}

fn discard_drawing_span(input: &str) -> IResult<&str, &str> {
    discard(pair(
        bracket_tag.and_then(tuple((take_until("\\p"), tag("\\p"), one_of("123456789")))),
        alt((discard(many_till(anychar, parse_drawing_end_tag)), rest)),
    ))
    .parse(input)
}

pub(crate) fn strip_formatting_tags(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                discard_drawing_span,
                discard(bracket_tag),
                take_until("{"),
                rest,
            )),
            eof,
        ),
        |(s, _)| s.concat(),
    )
    .parse(input)
}

fn convert_to_html_tag(input: &str) -> IResult<&str, &str> {
    alt((
        value("<b>", tag("{\\b1}")),
        value("</b>", tag("{\\b0}")),
        value("<i>", tag("{\\i1}")),
        value("</i>", tag("{\\i0}")),
        value("<u>", tag("{\\u1}")),
        value("</u>", tag("{\\u0}")),
    ))
    .parse(input)
}
