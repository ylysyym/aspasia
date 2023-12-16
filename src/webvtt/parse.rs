use std::io::{BufRead, BufReader, Read};

use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_until},
    character::complete::{char, i64, line_ending, multispace0, space0, space1},
    combinator::{map, opt, rest, value},
    multi::separated_list1,
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult, Parser,
};

use crate::{parsing::take_until_end_of_block, Moment, WebVttSubtitle};

use super::WebVttCue;

#[derive(Debug)]
enum WebVttBlock<'a> {
    Cue(WebVttCue),
    Note(String),
    Style(String),
    Region(String),
    Invalid(&'a str),
}

pub(crate) fn parse_header(input: &str) -> IResult<&str, Option<&str>> {
    preceded(
        tuple((tag_no_case("WEBVTT"), space0, opt(tag("-")), space0)),
        opt(take_until("\n")),
    )
    .parse(input)
}

fn parse_timestamp(input: &str) -> IResult<&str, Moment> {
    alt((
        map(
            tuple((
                delimited(space0, i64, char(':')),
                terminated(i64, char(':')),
                terminated(i64, char('.')),
                terminated(i64, space0),
            )),
            |(h, m, s, ms)| Moment::from_timestamp(h, m, s, ms),
        ),
        map(
            tuple((
                delimited(space0, i64, char(':')),
                terminated(i64, char('.')),
                terminated(i64, space0),
            )),
            |(m, s, ms)| Moment::from_timestamp(0, m, s, ms),
        ),
    ))
    .parse(input)
}

fn parse_cue_timing(input: &str) -> IResult<&str, ((Moment, Moment), Option<&str>)> {
    terminated(
        tuple((
            separated_pair(parse_timestamp, tag("-->"), parse_timestamp),
            opt(take_until("\n")),
        )),
        line_ending,
    )
    .parse(input)
}

fn parse_cue_identifier(input: &str) -> IResult<&str, &str> {
    terminated(take_until("\n"), line_ending).parse(input)
}

fn parse_cue(input: &str) -> IResult<&str, WebVttBlock> {
    map(
        tuple((
            alt((
                pair(opt(parse_cue_identifier), parse_cue_timing),
                pair(opt(space0), parse_cue_timing),
            )),
            take_until_end_of_block,
        )),
        |((identifier, ((start, end), settings)), text)| {
            WebVttBlock::Cue(WebVttCue {
                identifier: identifier.map(std::string::ToString::to_string),
                text,
                settings: settings.map(std::string::ToString::to_string),
                start,
                end,
            })
        },
    )
    .parse(input)
}

fn parse_style(input: &str) -> IResult<&str, WebVttBlock> {
    map(
        preceded(pair(tag("STYLE"), line_ending), take_until_end_of_block),
        WebVttBlock::Style,
    )
    .parse(input)
}

fn parse_note(input: &str) -> IResult<&str, WebVttBlock> {
    map(
        preceded(
            pair(tag("NOTE"), alt((space1, line_ending))),
            take_until_end_of_block,
        ),
        WebVttBlock::Note,
    )
    .parse(input)
}

fn parse_region(input: &str) -> IResult<&str, WebVttBlock> {
    map(
        preceded(pair(tag("REGION"), line_ending), take_until_end_of_block),
        WebVttBlock::Region,
    )
    .parse(input)
}

fn parse_invalid(input: &str) -> IResult<&str, WebVttBlock> {
    map(alt((take_until("\n\n"), rest)), WebVttBlock::Invalid).parse(input)
}

fn parse_block(input: &str) -> IResult<&str, WebVttBlock> {
    preceded(
        multispace0,
        alt((
            parse_cue,
            parse_style,
            parse_note,
            parse_region,
            parse_invalid,
        )),
    )
    .parse(input)
}

fn parse_blocks(input: &str) -> IResult<&str, Vec<WebVttBlock>> {
    separated_list1(parse_double_newline, parse_block).parse(input)
}

fn parse_double_newline(input: &str) -> IResult<&str, &str> {
    value("", pair(line_ending, line_ending)).parse(input)
}

pub(crate) fn parse_vtt<T: Read>(reader: BufReader<T>) -> WebVttSubtitle {
    let mut lines = reader.lines();

    let mut header = None;
    if let Some(Ok(first_line)) = lines.next() {
        if let Ok((_, parsed_header)) = parse_header(first_line.as_str()) {
            header = parsed_header.map(std::string::ToString::to_string);
        }
    }

    let mut queue = String::new();
    let mut cues = Vec::new();
    let mut styles = Vec::new();
    let mut regions = Vec::new();
    let mut is_streaming = true;
    while is_streaming {
        if let Some(has_line) = lines.next() {
            let Ok(line) = has_line else {
                continue;
            };
            queue.push_str(line.as_str());
            queue.push('\n');
            if !line.is_empty() {
                // While streaming, only parse after double newlines (where line is empty)
                continue;
            }
        } else {
            is_streaming = false;
        }

        let Ok((unparsed, blocks)) = parse_blocks(queue.as_str()) else {
            continue;
        };
        for block in blocks {
            match block {
                WebVttBlock::Cue(cue) => cues.push(cue),
                WebVttBlock::Style(style) => styles.push(style),
                WebVttBlock::Region(region) => regions.push(region),
                _ => {}
            }
        }

        queue = unparsed.to_string();
    }

    WebVttSubtitle::builder()
        .and_header(header)
        .cues(cues)
        .styles(styles)
        .regions(regions)
        .build()
}
