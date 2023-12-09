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
