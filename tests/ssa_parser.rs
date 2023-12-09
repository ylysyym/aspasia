use std::str::FromStr;

use aspasia::{SsaSubtitle, Subtitle};

#[test]
fn parse() {
    let ssa = SsaSubtitle::from_str("[Script Info]

[V4 Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, TertiaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, AlphaLevel, Encoding
Style: Zombie,Courier New,32,16777215,65535,65535,-2147483640,-1,0,1,3,0,2,0,0,40,0,0

[Events]
Format: Marked, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: Marked=0,0:01:19.00,0:01:26.50,Zombie,,0000,0000,0000,,{\\a10}Graahhh...
Dialogue: Marked=0,0:01:31.50,0:01:37.00,Zombie,,0000,0000,0100,,{\\a2}Brains...").unwrap();

    assert_eq!(ssa.events().len(), 2);
    assert_eq!(ssa.styles().len(), 1);
}
