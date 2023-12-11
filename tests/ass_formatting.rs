use std::str::FromStr;

use aspasia::{AssSubtitle, Subtitle, TextEvent, TextSubtitle};

const SUB_TEXT: &str = "[Script Info]

[Events]
Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:27.92,0:00:30.26,*Default,NTP,0000,0000,0000,,- Oh\\N{\\fnArial}{\\b0}{\\fs14}{\\3c&H202020&}{\\shad1}- That's right
";

#[test]

fn strip_ass_tags() {
    let mut ass = AssSubtitle::from_str(SUB_TEXT).unwrap();
    ass.strip_formatting();

    assert_eq!(ass.events().len(), 1);
    assert_eq!(ass.event(0).unwrap().text, "- Oh\\N- That's right")
}

#[test]
fn convert_newlines() {
    let ass = AssSubtitle::from_str(SUB_TEXT).unwrap();

    assert_eq!(
        ass.event(0).unwrap().unformatted_text().to_string(),
        "- Oh\\N- That's right"
    );
    assert_eq!(
        ass.event(0).unwrap().as_plaintext().to_string(),
        "- Oh\n- That's right"
    );
}

#[test]
fn remove_drawing_spans() {
    let ass = AssSubtitle::from_str(
        "[Script Info]

[Events]
Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:06.00,0:00:10.30,*Default,NTP,0000,0000,0000,,Oh, {\\b1\\p9}yes.
Dialogue: 0,0:00:14.80,0:00:19.90,*Default,NTP,0000,0000,0000,,Well, {\\p1}yes, but {\\p0}no.
",
    )
    .unwrap();

    assert_eq!(ass.events().len(), 2);
    assert_eq!(ass.event(0).unwrap().unformatted_text().to_string(), "Oh, ");
    assert_eq!(
        ass.event(1).unwrap().unformatted_text().to_string(),
        "Well, no."
    );
}
