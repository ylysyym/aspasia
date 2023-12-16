use std::str::FromStr;

use aspasia::{Subtitle, WebVttSubtitle};

#[test]
fn trailing_newlines() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT

1
00:00:00.000 --> 00:00:05.000
Line


",
    )
    .unwrap();

    assert_eq!(vtt.events().len(), 1);
    assert_eq!(vtt.event(0).unwrap().text, "Line");
}

#[test]
fn short_timestamps() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT

1
00:00.000 --> 00:05.000
Hello,

2
00:07.300 --> 00:10.100
dear friend

",
    )
    .unwrap();
    assert_eq!(vtt.events().len(), 2);
    assert_eq!(vtt.event(0).unwrap().start, 0.into());
    assert_eq!(vtt.event(0).unwrap().end, 5000.into());
    assert_eq!(vtt.event(1).unwrap().start, 7300.into());
}

#[test]
fn multiline_cues() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT

1
00:00:00.000 --> 00:00:05.000
What
a
great
day


00:00:07.300 --> 00:00:10.100
it
is!",
    )
    .unwrap();
    assert_eq!(vtt.events().len(), 2);
    assert_eq!(vtt.event(0).unwrap().text, "What\na\ngreat\nday");
    assert_eq!(vtt.event(1).unwrap().text, "it\nis!");
}

#[test]
fn invalid_block() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT
    
00:00:00.900 --> 00:00:03.350
Text

Invalid stuff

00:00:06.100 --> 00:00:09.800
More text
",
    )
    .unwrap();

    assert_eq!(vtt.events().len(), 2);
    assert_eq!(vtt.event(0).unwrap().text, "Text");
    assert_eq!(vtt.event(1).unwrap().text, "More text");
}
