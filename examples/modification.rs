use aspasia::{AssSubtitle, Error, Subtitle, TimeDelta, TimedEvent, TimedSubtitleFile};

fn main() -> Result<(), Error> {
    let sub = TimedSubtitleFile::new("/path/to/subtitle.ass")?;
    let mut ass = AssSubtitle::from(sub);

    println!("{}", ass.script_info());

    for event in ass.events_mut() {
        event.style = Some("Karaoke".to_string());

        if event.duration() > TimeDelta::from(500) {
            event.shift(2000.into());
        }
    }

    ass.export("/path/to/output.ass")
}
