use nom::{
    branch::alt,
    character::complete::{anychar, line_ending, multispace0},
    combinator::{eof, map},
    multi::many_till,
    sequence::pair,
    IResult, Parser,
};

use crate::Moment;

pub(crate) fn take_until_end_of_block(input: &str) -> IResult<&str, String> {
    map(
        many_till(
            anychar,
            alt((pair(line_ending, line_ending), pair(multispace0, eof))),
        ),
        |(s, _)| s.into_iter().collect(),
    )
    .parse(input)
}

pub(crate) fn as_moment(hours: i64, minutes: i64, seconds: i64, milliseconds: i64) -> Moment {
    (hours * 60 * 60 * 1000 + minutes * 60 * 1000 + seconds * 1000 + milliseconds).into()
}
