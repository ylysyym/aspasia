use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::char,
    combinator::{eof, map, rest, value},
    multi::{many0, many_till},
    sequence::tuple,
    IResult, Parser,
};

fn discard_tag(input: &str) -> IResult<&str, &str> {
    value("", tuple((char('<'), take_until(">"), char('>')))).parse(input)
}

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

fn keep_html_tags(input: &str) -> IResult<&str, &str> {
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

pub(crate) fn convert_to_ass_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((convert_to_ass_tag, discard_tag, take_until("<"), rest)),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

pub(crate) fn convert_to_srt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((keep_html_tags, discard_tag, take_until("<"), rest)),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

fn discard_html_tag(input: &str) -> IResult<&str, &str> {
    value("", tuple((char('<'), take_until(">"), char('>')))).parse(input)
}

pub(crate) fn discard_html_tags(input: &str) -> IResult<&str, String> {
    map(
        many0(alt((
            discard_html_tag,
            alt((take_until("<"), take_while1(|_| true))),
        ))),
        |s| s.join("").to_string(),
    )
    .parse(input)
}
