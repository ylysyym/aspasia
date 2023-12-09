use aspasia::{AssSubtitle, Error, SubRipSubtitle, Subtitle, TimedSubtitleFile, WebVttSubtitle};

fn main() -> Result<(), Error> {
    let sub = TimedSubtitleFile::new("/path/to/file.srt")?;

    // Get the file as its specific format
    let srt = SubRipSubtitle::from(sub);

    // You can use into() to convert the file
    let vtt: WebVttSubtitle = srt.into();

    // or from()
    let ass = AssSubtitle::from(vtt);

    ass.export("/path/to/converted.ass")?;

    Ok(())
}
