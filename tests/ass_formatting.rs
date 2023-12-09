use std::str::FromStr;

use aspasia::{AssSubtitle, Subtitle, TextSubtitle};

#[test]

fn strip_ass_tags() {
    let mut ass = AssSubtitle::from_str(
        "[Script Info]

[Events]
Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:27.92,0:00:30.26,*Default,NTP,0000,0000,0000,,- Oh\\N{\\fnArial}{\\b0}{\\fs14}{\\3c&H202020&}{\\shad1}- That's right
",
    )
    .unwrap();
    ass.strip_formatting();

    assert_eq!(ass.events().len(), 1);
    assert_eq!(ass.event(0).unwrap().text, "- Oh\\N- That's right")
}
