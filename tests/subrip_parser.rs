use std::str::FromStr;

use aspasia::{SubRipSubtitle, Subtitle};

const MISNUMBERED_SUB: &str = "10
00:00:01,000 --> 00:00:02,500
First line

5
00:00:03,000 --> 00:00:06,000
Second line
";

#[test]
fn blank_text() {
    let srt = SubRipSubtitle::from_str(
        "1
00:00:01,000 --> 00:00:02,500



2
00:00:03,000 --> 00:00:06,000
",
    )
    .unwrap();

    assert_eq!(srt.events().len(), 2);
    assert_eq!(srt.event(0).unwrap().text, "");
}

#[test]
fn blank_lines_in_text() {
    let srt = SubRipSubtitle::from_str(
        "1
00:00:01,000 --> 00:00:02,500
This

has a blank line in the text

2
00:00:03,000 --> 00:00:06,000
Fin.

",
    )
    .unwrap();

    assert_eq!(srt.events().len(), 2);
    assert_eq!(
        srt.event(0).unwrap().text,
        "This\n\nhas a blank line in the text"
    );
    assert_eq!(srt.event(1).unwrap().text, "Fin.");
}

#[test]
fn improper_spacing_between_lines() {
    let srt = SubRipSubtitle::from_str(
        "1
00:00:01,000 --> 00:00:02,500
2
00:00:03,000 --> 00:00:06,000
Some text",
    )
    .unwrap();

    assert_eq!(srt.events().len(), 1);
}

#[test]
fn crlf() {
    let srt = SubRipSubtitle::from_str(
        "1\r
00:00:01,000 --> 00:00:02,500\r
Some text\r
\r
2\r
00:00:03,000 --> 00:00:06,000\r
More text\r
",
    )
    .unwrap();

    assert_eq!(srt.events().len(), 2);
    assert_eq!(srt.event(0).unwrap().text, "Some text");
    assert_eq!(srt.event(1).unwrap().text, "More text");
}

#[test]
fn misnumbered_lines() {
    let srt = SubRipSubtitle::from_str(MISNUMBERED_SUB).unwrap();

    assert_eq!(srt.events().len(), 2);
    assert_eq!(srt.event(0).unwrap().line_number, 10);
    assert_eq!(srt.event(0).unwrap().text, "First line");
    assert_eq!(srt.event(1).unwrap().line_number, 5);
    assert_eq!(srt.event(1).unwrap().text, "Second line");
}

#[test]
fn renumbering() {
    let mut srt = SubRipSubtitle::from_str(MISNUMBERED_SUB).unwrap();
    srt.renumber();

    assert_eq!(srt.event(0).unwrap().line_number, 1);
    assert_eq!(srt.event(1).unwrap().line_number, 2);
}
