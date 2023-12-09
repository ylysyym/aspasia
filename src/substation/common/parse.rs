use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until},
    character::complete::{anychar, char, i64, multispace0, space0},
    combinator::{map, value},
    multi::{many0, many_till},
    sequence::{delimited, preceded, separated_pair, terminated, tuple},
    IResult, Parser,
};

use crate::{Format, Moment};

use super::data::{SubStationFont, SubStationGraphic};

#[derive(Clone, Debug)]
pub(crate) enum SubStationSection {
    ScriptInfo,
    Styles,
    Events,
    Fonts,
    Graphics,
}

pub(crate) fn parse_script_info_heading(input: &str) -> IResult<&str, &str> {
    delimited(char('['), tag_no_case("Script Info"), char(']')).parse(input)
}

fn parse_script_type(input: &str) -> IResult<&str, Format> {
    map(
        preceded(
            tuple((space0, tag_no_case("ScriptType"), space0, char(':'), space0)),
            terminated(
                alt((tag_no_case("v4.00+"), tag_no_case("v4.00"))),
                multispace0,
            ),
        ),
        |s: &str| match s.to_ascii_lowercase().as_str() {
            "v4.00" => Format::Ssa,
            _ => Format::Ass,
        },
    )
    .parse(input)
}

pub(crate) fn parse_format(input: &str) -> IResult<&str, Format> {
    map(many_till(anychar, parse_script_type), |(_, format)| format).parse(input)
}

fn parse_script_info_header(input: &str) -> IResult<&str, SubStationSection> {
    value(SubStationSection::ScriptInfo, tag_no_case("Script Info")).parse(input)
}

fn parse_events_header(input: &str) -> IResult<&str, SubStationSection> {
    value(SubStationSection::Events, tag_no_case("Events")).parse(input)
}

fn parse_graphics_header(input: &str) -> IResult<&str, SubStationSection> {
    value(SubStationSection::Graphics, tag_no_case("Graphics")).parse(input)
}

fn parse_fonts_header(input: &str) -> IResult<&str, SubStationSection> {
    value(SubStationSection::Fonts, tag_no_case("Fonts")).parse(input)
}

fn parse_styles_header(input: &str) -> IResult<&str, SubStationSection> {
    value(
        SubStationSection::Styles,
        alt((tag_no_case("V4 Styles"), tag_no_case("V4+ Styles"))),
    )
    .parse(input)
}

pub(crate) fn parse_category_header(input: &str) -> IResult<&str, SubStationSection> {
    delimited(
        multispace0,
        delimited(
            char('['),
            alt((
                parse_events_header,
                parse_styles_header,
                parse_fonts_header,
                parse_graphics_header,
                parse_script_info_header,
            )),
            char(']'),
        ),
        multispace0,
    )
    .parse(input)
}

pub(crate) fn parse_reverse_bool(input: &str) -> IResult<&str, bool> {
    alt((value(true, tag("-1")), value(false, char('0')))).parse(input)
}

fn parse_fontname(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((multispace0, tag("fontname:"), space0)),
        take_until("\n"),
    )
    .parse(input)
}

fn parse_font(input: &str) -> IResult<&str, SubStationFont> {
    map(
        separated_pair(parse_fontname, multispace0, take_until("\n\n")),
        |(fontname, data)| SubStationFont {
            fontname: fontname.to_string(),
            data: data.to_string(),
        },
    )
    .parse(input)
}

pub(crate) fn parse_fonts(input: &str) -> IResult<&str, Vec<SubStationFont>> {
    many0(parse_font).parse(input)
}

fn parse_filename(input: &str) -> IResult<&str, &str> {
    preceded(
        tuple((multispace0, tag("filename:"), space0)),
        take_until("\n"),
    )
    .parse(input)
}

fn parse_graphic(input: &str) -> IResult<&str, SubStationGraphic> {
    map(
        separated_pair(parse_filename, multispace0, take_until("\n\n")),
        |(filename, data)| SubStationGraphic {
            filename: filename.to_string(),
            data: data.to_string(),
        },
    )
    .parse(input)
}

pub(crate) fn parse_graphics(input: &str) -> IResult<&str, Vec<SubStationGraphic>> {
    many0(parse_graphic).parse(input)
}

pub(crate) fn parse_timestamp(input: &str) -> IResult<&str, Moment> {
    map(
        delimited(
            space0,
            tuple((
                terminated(i64, char(':')),
                i64,
                delimited(char(':'), i64, char('.')),
                i64,
            )),
            space0,
        ),
        |(h, m, s, cs)| Moment::from_timestamp(h, m, s, cs * 10),
    )
    .parse(input)
}
