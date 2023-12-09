use std::str::FromStr;

use aspasia::{SubRipSubtitle, Subtitle, TextSubtitle};

#[test]
fn strip_nested_tags() {
    let mut srt = SubRipSubtitle::from_str(
        "1
00:00:00,000 --> 00:00:02,000
<b>Bolded <i>and italicised</i></b>
",
    )
    .unwrap();
    srt.strip_formatting();
    assert_eq!(srt.event(0).unwrap().text, "Bolded and italicised");
}

#[test]
fn strip_bracket_tags() {
    let mut srt = SubRipSubtitle::from_str(
        "1
00:00:00,000 --> 00:00:02,000
This {b}should not{/b} be {u}formatted{/u}",
    )
    .unwrap();
    srt.strip_formatting();

    assert_eq!(srt.event(0).unwrap().text, "This should not be formatted")
}

#[test]
fn mixed_tags() {
    let mut srt = SubRipSubtitle::from_str(
        "1
00:00:00,000 --> 00:00:02,000
<b>{i}Strip{/i}{u} this{/u}</b> away
",
    )
    .unwrap();
    srt.strip_formatting();

    assert_eq!(srt.event(0).unwrap().text, "Strip this away")
}

#[test]
fn terminating_bracket() {
    let mut srt = SubRipSubtitle::from_str(
        "1
00:00:00,000 --> 00:00:03,000
Some <>text<",
    )
    .unwrap();
    srt.strip_formatting();

    assert_eq!(srt.event(0).unwrap().text, "Some text<");
}
