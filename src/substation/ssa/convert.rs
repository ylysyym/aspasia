use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{eof, map, rest, value},
    multi::many_till,
    IResult, Parser,
};

use crate::{
    parsing::{bracket_tag, discard},
    substation::common::convert::convert_font_color_tag,
};

fn convert_to_html_tag(input: &str) -> IResult<&str, &str> {
    alt((
        value("<b>", tag("{\\b1}")),
        value("</b>", tag("{\\b0}")),
        value("<i>", tag("{\\i1}")),
        value("</i>", tag("{\\i0}")),
    ))
    .parse(input)
}

pub(crate) fn ssa_to_srt_formatting(input: &str) -> IResult<&str, String> {
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

pub(crate) fn ssa_to_vtt_formatting(input: &str) -> IResult<&str, String> {
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

pub(crate) fn strip_formatting_tags(input: &str) -> IResult<&str, String> {
    map(
        many_till(alt((discard(bracket_tag), take_until("{"), rest)), eof),
        |(s, _)| s.concat(),
    )
    .parse(input)
}
