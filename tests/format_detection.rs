use aspasia::{detect_format_from_str, Format};

#[test]
fn srt() {
    let detected = detect_format_from_str(
        "1
00:00:05,000 --> 00:00:09,200
Text",
    )
    .unwrap();

    assert_eq!(detected, Format::SubRip);
}

#[test]
fn blank_webvtt() {
    let detected = detect_format_from_str("WEBVTT").unwrap();

    assert_eq!(detected, Format::WebVtt);
}

#[test]
fn ass() {
    let detected = detect_format_from_str(
        "[Script Info]
ScriptType: v4.00+",
    )
    .unwrap();

    assert_eq!(detected, Format::Ass);
}

#[test]
fn microdvd() {
    let detected = detect_format_from_str("{0}{120}Help|me").unwrap();

    assert_eq!(detected, Format::MicroDvd);
}

#[test]
fn ssa() {
    let detected = detect_format_from_str(
        "[Script Info]
ScriptType: v4.00 ",
    )
    .unwrap();

    assert_eq!(detected, Format::Ssa);
}
