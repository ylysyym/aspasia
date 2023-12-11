use std::io::{BufRead, BufReader, Read};

use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, i64},
    combinator::{map, rest},
    sequence::{delimited, pair},
    IResult, Parser,
};

use crate::{timing::Frame, MicroDvdSubtitle};

use super::MicroDvdEvent;

fn parse_frame(input: &str) -> IResult<&str, Frame> {
    map(delimited(char('{'), i64, char('}')), Frame::from).parse(input)
}

fn parse_frame_interval(input: &str) -> IResult<&str, (Frame, Frame)> {
    pair(parse_frame, parse_frame).parse(input)
}

pub(crate) fn parse_microdvd_line(input: &str) -> IResult<&str, MicroDvdEvent> {
    map(
        pair(parse_frame_interval, alt((take_until("\n"), rest))),
        |((start, end), text)| MicroDvdEvent {
            start,
            end,
            text: text.to_string(),
        },
    )
    .parse(input)
}

pub(crate) fn parse_microdvd<T: Read>(reader: BufReader<T>) -> MicroDvdSubtitle {
    let mut events = Vec::new();
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };
        let Ok((_, event)) = parse_microdvd_line(line.as_str()) else {
            continue;
        };
        events.push(event);
    }

    MicroDvdSubtitle::from_events(events)
}
