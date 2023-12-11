use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    combinator::{eof, map, rest, value},
    multi::many_till,
    sequence::delimited,
    IResult, Parser,
};

use crate::parsing::{bracket_tag, discard, html_tag};

fn convert_to_ass_tag(input: &str) -> IResult<&str, &str> {
    alt((
        convert_to_ssa_tag,
        value("{\\u1}", tag("<u>")),
        value("{\\u0}", tag("</u>")),
        value("{\\u1}", tag("{u}")),
        value("{\\u0}", tag("{/u}")),
    ))
    .parse(input)
}

fn convert_to_ssa_tag(input: &str) -> IResult<&str, &str> {
    alt((
        value("{\\b1}", tag("<b>")),
        value("{\\b0}", tag("</b>")),
        value("{\\b1}", tag("{b}")),
        value("{\\b0}", tag("{/b}")),
        value("{\\i1}", tag("<i>")),
        value("{\\i0}", tag("</i>")),
        value("{\\i1}", tag("{i}")),
        value("{\\i0}", tag("{/i}")),
    ))
    .parse(input)
}

fn convert_to_vtt_tag(input: &str) -> IResult<&str, &str> {
    alt((
        value("<b>", tag("{b}")),
        value("</b>", tag("{/b}")),
        value("<i>", tag("{i}")),
        value("</i>", tag("{/i}")),
        value("<u>", tag("{u}")),
        value("</u>", tag("{/u}")),
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

fn correct_colour_hex(input: &str) -> String {
    let s = format!("{input:06}");
    let mut result = String::new();
    result.push_str(&s[4..6]);
    result.push_str(&s[2..4]);
    result.push_str(&s[0..2]);

    result
}

// TODO this needs to handle color names, eg. white or magenta
fn replace_font_color_tag(input: &str) -> IResult<&str, String> {
    map(
        delimited(tag("<font color=\"#"), take_until("\">"), tag("\">")),
        |s| format!("{{\\c&H{}&}}", correct_colour_hex(s)),
    )
    .parse(input)
}

pub(crate) fn srt_to_ass_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                map(convert_to_ass_tag, std::string::ToString::to_string),
                replace_font_color_tag,
                map(discard(html_tag), std::string::ToString::to_string),
                map(discard(bracket_tag), std::string::ToString::to_string),
                map(take_while(|c| c != '<' && c != '{'), |s: &str| {
                    s.to_string()
                }),
                map(rest, |s: &str| s.to_string()),
            )),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

pub(crate) fn srt_to_ssa_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                map(convert_to_ssa_tag, std::string::ToString::to_string),
                replace_font_color_tag,
                map(discard(html_tag), std::string::ToString::to_string),
                map(discard(bracket_tag), std::string::ToString::to_string),
                map(take_while(|c| c != '<' && c != '{'), |s: &str| {
                    s.to_string()
                }),
                map(rest, |s: &str| s.to_string()),
            )),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}

pub(crate) fn srt_to_vtt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                convert_to_vtt_tag,
                keep_html_tags,
                discard(html_tag),
                discard(bracket_tag),
                take_while1(|c| c != '<' && c != '{'),
                rest,
            )),
            eof,
        ),
        |(v, _)| v.join(""),
    )
    .parse(input)
}
