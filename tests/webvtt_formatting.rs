use std::str::FromStr;

use aspasia::{Subtitle, TextSubtitle, WebVttSubtitle};

#[test]
fn strip_vtt_format() {
    let mut vtt = WebVttSubtitle::from_str(
        "WEBVTT

00:00:00.000 --> 00:00:05.000
<v James><b>Had</b> <c.person>it <i>up</i> to</c><u> here</u>


",
    )
    .unwrap();
    vtt.strip_formatting();

    assert_eq!(vtt.events().len(), 1);
    assert_eq!(vtt.event(0).unwrap().text, "Had it up to here");
}

#[test]
fn basic_output() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT
    
00:00:00.650 --> 00:00:01.200
something

NOTE ignore this please


00:00:04.000 --> 00:00:06.000
anything


00:00:10.100 --> 00:00:18.999
everything
",
    );

    assert_eq!(
        vtt.unwrap().to_string(),
        "WEBVTT

00:00:00.650 --> 00:00:01.200
something

00:00:04.000 --> 00:00:06.000
anything

00:00:10.100 --> 00:00:18.999
everything
"
    );
}

#[test]
fn identifiers() {
    let vtt = WebVttSubtitle::from_str(
        "WEBVTT

1
00:00:00.650 --> 00:00:01.200
foo

2
00:00:04.000 --> 00:00:06.000
bar

3
00:00:10.100 --> 00:00:18.999
baz
",
    );

    assert_eq!(
        vtt.unwrap().to_string(),
        "WEBVTT

1
00:00:00.650 --> 00:00:01.200
foo

2
00:00:04.000 --> 00:00:06.000
bar

3
00:00:10.100 --> 00:00:18.999
baz
"
    );
}
