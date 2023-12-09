use std::str::FromStr;

use aspasia::{AssSubtitle, SubRipSubtitle, Subtitle, WebVttSubtitle};

#[test]
fn to_vtt_formatting() {
    let srt = SubRipSubtitle::from_str(
        "1
00:00:00,000 --> 00:00:02,000
<b>{i}Some{/i}{u} mixed{/u}</b> formatting
",
    )
    .unwrap();
    let vtt = WebVttSubtitle::from(&srt);

    assert_eq!(vtt.events().len(), 1);
    assert_eq!(
        vtt.event(0).unwrap().text,
        "<b><i>Some</i><u> mixed</u></b> formatting"
    );
}

#[test]
fn to_ass_formatting() {
    let srt = SubRipSubtitle::from_str(
        "1
00:00:00,000 --> 00:00:02,000
<b>{i}Some{/i}{u} mixed{/u}</b> formatting
<font color=\"#ff0000\">and more</font>",
    )
    .unwrap();
    let ass = AssSubtitle::from(&srt);

    assert_eq!(ass.events().len(), 1);
    assert_eq!(
        ass.event(0).unwrap().text,
        "{\\b1}{\\i1}Some{\\i0}{\\u1} mixed{\\u0}{\\b0} formatting\\N{\\c&H0000ff&}and more"
    );
}
