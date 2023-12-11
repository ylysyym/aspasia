use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{anychar, char, line_ending, multispace0},
    combinator::{eof, map, value},
    error::ParseError,
    multi::many_till,
    sequence::{delimited, pair},
    IResult, Parser,
};

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

pub(crate) fn discard<'a, I, O2, E: ParseError<I>, F>(
    parser: F,
) -> impl FnMut(I) -> IResult<I, &'a str, E>
where
    F: Parser<I, O2, E>,
{
    value("", parser)
}

pub(crate) fn bracket_tag(input: &str) -> IResult<&str, &str> {
    delimited(char('{'), take_until("}"), char('}')).parse(input)
}

pub(crate) fn html_tag(input: &str) -> IResult<&str, &str> {
    delimited(char('<'), take_until(">"), char('>')).parse(input)
}
