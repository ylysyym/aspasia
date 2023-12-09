use aspasia::{Error, SubRipSubtitle, Subtitle, TimedEvent, TimedSubtitleFile, WebVttSubtitle};

fn main() -> Result<(), Error> {
    // We can directly specify the format to open a subtitle file
    let vtt = WebVttSubtitle::from_path("/path/to/some.vtt")?;

    // and then directly work with its data
    println!("{}", vtt.header().cloned().unwrap_or_default());

    // or we could use the more general interface to open (timed) subtitle files
    let sub = TimedSubtitleFile::new("/path/to/file.srt")?;

    // Move the underlying data out in order to access format-specific properties
    // Note that if the format doesn't match, this will perform a conversion instead of just moving the data
    let mut srt = SubRipSubtitle::from(sub);

    // Now we can access format-specific methods like SubRipSubtitle::renumber()
    srt.renumber();

    // Access and modify events
    for event in srt.events_mut() {
        event.shift(600.into());
    }

    // Write the modified subtitle to file
    srt.export("/path/to/output.srt")?;

    Ok(())
}
