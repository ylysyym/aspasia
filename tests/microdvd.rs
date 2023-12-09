use std::str::FromStr;

use aspasia::{MicroDvdSubtitle, Moment, Subtitle, TimedMicroDvdSubtitle};

const SUB_TEXT: &str = "{1}{450}One
{460}{510}Two
";

#[test]
fn parse_timed() {
    let sub = TimedMicroDvdSubtitle::from_str(SUB_TEXT).unwrap();

    assert_eq!(sub.events().len(), 2);
    assert_eq!(sub.event(0).unwrap().text, "One");
    assert_eq!(sub.event(0).unwrap().start, Moment::from(42));
    assert_eq!(sub.event(0).unwrap().end, Moment::from(18750));
    assert_eq!(sub.event(1).unwrap().text, "Two");
    assert_eq!(sub.event(1).unwrap().start, Moment::from(19167));
    assert_eq!(sub.event(1).unwrap().end, Moment::from(21250));
}

#[test]
fn parse_raw() {
    let sub = MicroDvdSubtitle::from_str(SUB_TEXT).unwrap();

    assert_eq!(sub.events().len(), 2);
    assert_eq!(sub.event(0).unwrap().text, "One");
    assert_eq!(sub.event(0).unwrap().start, 1.into());
    assert_eq!(sub.event(0).unwrap().end, 450.into());
    assert_eq!(sub.event(1).unwrap().text, "Two");
    assert_eq!(sub.event(1).unwrap().start, 460.into());
    assert_eq!(sub.event(1).unwrap().end, 510.into());
}

#[test]
fn lossless_conversion() {
    let sub = TimedMicroDvdSubtitle::from_str(SUB_TEXT).unwrap();
    let out = sub.to_string();

    assert_eq!(SUB_TEXT, out);
}

#[test]
fn framerate_modification() {
    let mut sub = TimedMicroDvdSubtitle::from_str(SUB_TEXT).unwrap();
    sub.set_framerate(100.0);

    assert_eq!(sub.event(0).unwrap().start, Moment::from(42));
    assert_eq!(sub.event(0).unwrap().end, Moment::from(18750));

    sub.update_framerate(75.0);

    assert_eq!(sub.event(0).unwrap().start, Moment::from(56));
    assert_eq!(sub.event(0).unwrap().end, Moment::from(25000));
}
