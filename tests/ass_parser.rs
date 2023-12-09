use std::str::FromStr;

use aspasia::{AssSubtitle, Subtitle};

#[test]
fn dialogue() {
    let ass = AssSubtitle::from_str(
        "[Script Info]
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.40,Default,,0,0,0,,Oh, yeah.
",
    )
    .unwrap();
    assert_eq!(ass.events().len(), 1);
    assert_eq!(ass.event(0).unwrap().text, "Oh, yeah.");
}

#[test]
fn command() {
    let ass = AssSubtitle::from_str(
        "[Script Info]
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Command: 0,0:00:01.00,0:00:03.40,Default,,0,0,0,,ls
",
    )
    .unwrap();
    assert_eq!(ass.events().len(), 0);
    assert_eq!(ass.commands().len(), 1);
    assert_eq!(ass.command(0).unwrap().text, "ls");
}

#[test]
fn ignore_comments_and_unrecognised_lines() {
    let ass = AssSubtitle::from_str(
        "[Script Info]
[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:01.00,0:00:03.00,Default,,0,0,0,,Hi,
Comment: 0,0:00:02.00,0:00:05.00,Default,,0,0,0,,Firstly,
;there isn't


anymore

Dialogue: 0,0:00:03.20,0:00:05.40,Default,,0,0,0,,Mark.
",
    )
    .unwrap();
    assert_eq!(ass.events().len(), 2);
    assert_eq!(ass.event(0).unwrap().text, "Hi,");
    assert_eq!(ass.event(1).unwrap().text, "Mark.");
}
