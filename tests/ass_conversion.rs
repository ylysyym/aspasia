use std::str::FromStr;

use aspasia::{AssSubtitle, SubRipSubtitle, Subtitle, WebVttSubtitle};

#[test]

fn ass_to_srt_format() {
    let ass = AssSubtitle::from_str(
        "[Script Info]

[Events]
Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:27.92,0:00:30.26,*Default,NTP,0000,0000,0000,,- Oh\\N{\\b1\\i1}{\\fs14\\1c&HFF2022&}{\\shad1}- That's right
",
    )
    .unwrap();
    let srt = SubRipSubtitle::from(ass);

    assert_eq!(srt.events().len(), 1);
    assert_eq!(
        srt.event(0).unwrap().text,
        "- Oh\n<b><i><font color=\"#2220FF\">- That's right"
    )
}

#[test]

fn ass_to_vtt_format() {
    let ass = AssSubtitle::from_str(
        "[Script Info]

[Events]
Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:27.92,0:00:30.26,*Default,NTP,0000,0000,0000,,- Oh\\N{\\b1\\i1}{\\fs14\\1c&HFF2022&}{\\shad1}- That's right
",
    )
    .unwrap();
    let vtt = WebVttSubtitle::from(&ass);

    assert_eq!(vtt.events().len(), 1);
    assert_eq!(vtt.event(0).unwrap().text, "- Oh\n<b><i>- That's right")
}
