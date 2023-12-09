use std::str::FromStr;

use aspasia::{AssSubtitle, SubRipSubtitle, Subtitle, WebVttSubtitle};

#[test]
fn to_srt_formatting() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT

00:00:00.000 --> 00:00:02.000
<c.header>This <i>is</i> <b>completely</b><u> wrong</u>.</c>
",
    )
    .unwrap();
    let srt = SubRipSubtitle::from(&vtt);

    assert_eq!(srt.events().len(), 1);
    assert_eq!(
        srt.event(0).unwrap().text,
        "This <i>is</i> <b>completely</b><u> wrong</u>."
    );
}

#[test]
fn to_ass_formatting() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT

00:00:00.000 --> 00:00:02.000
<c.header>This <i>is</i> <b>completely</b><u> wrong</u>.</c>
And yet..
",
    )
    .unwrap();
    let ass = AssSubtitle::from(&vtt);

    assert_eq!(ass.events().len(), 1);
    assert_eq!(
        ass.event(0).unwrap().text,
        "This {\\i1}is{\\i0} {\\b1}completely{\\b0}{\\u1} wrong{\\u0}.\\NAnd yet.."
    );
}
