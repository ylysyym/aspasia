use std::io::{BufRead, BufReader, Read};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while1},
    character::complete::{char, i64, line_ending, multispace0, space0, u32},
    combinator::{eof, map, opt, rest, verify},
    multi::{many_till, separated_list1},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult, Parser,
};

use crate::{
    parsing::{bracket_tag, discard, html_tag, take_until_end_of_block},
    Moment, SubRipSubtitle,
};

use super::SubRipEvent;

#[derive(Debug)]
pub(crate) enum SubRipBlock {
    NewLine(SubRipEvent),
    LineContinuation(String),
}

fn parse_timestamp(input: &str) -> IResult<&str, Moment> {
    map(
        delimited(
            space0,
            tuple((
                terminated(i64, char(':')),
                i64,
                delimited(char(':'), i64, char(',')),
                i64,
            )),
            space0,
        ),
        |(h, m, s, ms)| Moment::from_timestamp(h, m, s, ms),
    )
    .parse(input)
}

fn parse_timing(input: &str) -> IResult<&str, ((Moment, Moment), Option<&str>)> {
    pair(
        separated_pair(parse_timestamp, tag("-->"), parse_timestamp),
        terminated(opt(take_until("\n")), line_ending),
    )
    .parse(input)
}

fn parse_line_number(input: &str) -> IResult<&str, u32> {
    terminated(u32, line_ending).parse(input)
}

pub(crate) fn parse_new_line(input: &str) -> IResult<&str, SubRipBlock> {
    map(
        preceded(
            multispace0,
            tuple((parse_line_number, parse_timing, take_until_end_of_block)),
        ),
        |(line_number, ((start, end), coordinates), text)| {
            SubRipBlock::NewLine(SubRipEvent {
                line_number: line_number as usize,
                text,
                start,
                end,
                coordinates: coordinates.map(std::string::ToString::to_string),
            })
        },
    )
    .parse(input)
}

fn parse_continuation(input: &str) -> IResult<&str, SubRipBlock> {
    map(
        verify(take_until_end_of_block, |s: &String| !s.is_empty()),
        SubRipBlock::LineContinuation,
    )
    .parse(input)
}

fn parse_block(input: &str) -> IResult<&str, SubRipBlock> {
    alt((parse_new_line, parse_continuation)).parse(input)
}

pub(crate) fn parse_blocks(input: &str) -> IResult<&str, Vec<SubRipBlock>> {
    separated_list1(tag("\n\n"), parse_block).parse(input)
}

pub(crate) fn parse_srt<T: Read>(reader: BufReader<T>) -> SubRipSubtitle {
    let mut lines = reader.lines();

    let mut queue = String::new();
    let mut events = Vec::new();
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
                SubRipBlock::NewLine(line) => {
                    events.push(line);
                }
                SubRipBlock::LineContinuation(content) => {
                    if let Some(line) = events.last_mut() {
                        line.text.push('\n');
                        line.text.push('\n');
                        line.text.push_str(content.as_str());
                    }
                }
            }
        }

        queue = unparsed.to_string();
    }

    SubRipSubtitle::from_events(events)
}

pub(crate) fn strip_srt_formatting(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            alt((
                discard(html_tag),
                discard(bracket_tag),
                take_while1(|c| c != '<' && c != '{'),
                rest,
            )),
            eof,
        ),
        |(s, _)| s.join("").to_string(),
    )
    .parse(input)
}
