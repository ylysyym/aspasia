use aspasia::{SubRipSubtitle, Subtitle, TimedSubtitleFile};

#[test]
fn gbk_srt() {
    let sub = TimedSubtitleFile::new("./tests/data/gbk.srt").unwrap();
    let srt = SubRipSubtitle::from(sub);
    assert_eq!(srt.events().len(), 3);
    assert_eq!(srt.event(0).unwrap().text, "这是中文");
    assert_eq!(srt.event(1).unwrap().text, "还有更多的中文");
    assert_eq!(srt.event(2).unwrap().text, "再见");
}
