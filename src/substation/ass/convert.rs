use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{anychar, char, one_of},
    combinator::{eof, map, rest, value},
    multi::{many0, many_till},
    sequence::{delimited, preceded, terminated, tuple},
    IResult, Parser,
};

use crate::substation::common::convert::{convert_font_color_tag, discard_tag};

pub(crate) fn convert_to_srt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                map(convert_to_html_tag, std::string::ToString::to_string),
                convert_font_color_tag,
                map(discard_tag, std::string::ToString::to_string),
                map(take_until("{"), |s: &str| s.to_string()),
                map(rest, |s: &str| s.to_string()),
            )),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

pub(crate) fn convert_to_vtt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((convert_to_html_tag, discard_tag, take_until("{"), rest)),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

fn parse_drawing_end_tag(input: &str) -> IResult<&str, &str> {
    let ass_tag = preceded(char('{'), take_until("}")).parse(input)?;
    value("", take_until("\\p0")).parse(ass_tag.1)?;

    terminated(take_until("}"), char('}')).parse(input)
}

fn discard_drawing_span(input: &str) -> IResult<&str, &str> {
    let ass_tag = delimited(char('{'), take_until("}"), char('}')).parse(input)?;
    value(
        "",
        tuple((take_until("\\p"), tag("\\p"), one_of("123456789"))),
    )
    .parse(ass_tag.1)?;

    value(
        "",
        alt((value("", many_till(anychar, parse_drawing_end_tag)), rest)),
    )
    .parse(input)
}

pub(crate) fn strip_formatting_tags(input: &str) -> IResult<&str, String> {
    map(
        many0(alt((
            discard_drawing_span,
            discard_tag,
            take_until("{"),
            take_while1(|_| true),
        ))),
        |s| s.join("").to_string(),
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
