use std::{borrow::Cow, fmt::Display, fs::File, io::BufReader, path::Path, str::FromStr};

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{
    encoding::detect_file_encoding,
    timing::{frame_to_moment, moment_to_frame, Frame},
    traits::TimedSubtitle,
    AssSubtitle, Error, Moment, SsaSubtitle, SubRipSubtitle, Subtitle, TextEvent,
    TextEventInterface, TextSubtitle, TimedEvent, TimedEventInterface, WebVttSubtitle,
};

use super::parse::parse_microdvd;

type FrameRate = f32;

/// Timed version of MicroDVD (.sub) subtitle, using user-supplied framerate to calculate timings
///
/// When initialised without framerate, the default framerate is 24.
#[derive(Debug)]
pub struct TimedMicroDvdSubtitle {
    events: Vec<TimedMicroDvdEvent>,
    framerate: FrameRate,
}

/// Timed MicroDVD event
#[derive(Debug)]
pub struct TimedMicroDvdEvent {
    /// Start time of event
    pub start: Moment,
    /// End time of event
    pub end: Moment,
    /// Text to display during event
    pub text: String,
}

/// Unmodified MicroDVD subtitle, with events timed in terms of frames.
///
/// This is not well supported, so things like conversion are not implemented for this type.
/// If possible, use of [`MicroDvdSubtitle`], which represents subtitle events using actual timestamps, is better supported.
#[derive(Debug)]
pub struct MicroDvdSubtitle {
    events: Vec<MicroDvdEvent>,
}

/// Unmodified MicroDVD event, timed in terms of frames
#[derive(Debug)]
pub struct MicroDvdEvent {
    /// Frame at which event starts
    pub start: Frame,
    /// Frame at which event ends
    pub end: Frame,
    /// Text to display during event
    pub text: String,
}

impl TimedMicroDvdSubtitle {
    fn open_file_with_encoding(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        let file = File::open(path)?;
        let transcoded = DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(file);
        let reader = BufReader::new(transcoded);

        Ok(Self::from_raw(&parse_microdvd(reader), Some(24.0)))
    }

    fn open_file_with_encoding_and_framerate(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
        framerate: FrameRate,
    ) -> Result<Self, Error> {
        let file = File::open(path)?;
        let transcoded = DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(file);
        let reader = BufReader::new(transcoded);

        Ok(Self::from_raw(&parse_microdvd(reader), Some(framerate)))
    }

    /// Convert raw MicroDVD subtitle data to timed MicroDVD data, given the framerate the subtitles were created for.
    /// If no framerate is given, the default of 24 is used.
    #[must_use]
    pub fn from_raw(raw: &MicroDvdSubtitle, framerate: Option<FrameRate>) -> Self {
        let framerate = framerate.unwrap_or(24.0);
        let events = raw
            .events
            .iter()
            .map(|event| TimedMicroDvdEvent {
                start: frame_to_moment(event.start, framerate),
                end: frame_to_moment(event.end, framerate),
                text: event.text.clone(),
            })
            .collect();

        Self { events, framerate }
    }

    /// Create MicroDVD from path given and calculate its timings using the given framerate.
    ///
    /// # Errors
    ///
    /// Returns [`Error::FileIoError`] if an error occurs while opening the file
    pub fn with_framerate(path: impl AsRef<Path>, framerate: FrameRate) -> Result<Self, Error> {
        let mut enc = detect_file_encoding(path.as_ref(), Some(30)).ok();
        let mut result = Self::open_file_with_encoding_and_framerate(path.as_ref(), enc, framerate);

        if result.is_err() {
            enc = detect_file_encoding(path.as_ref(), None).ok();
            result = Self::open_file_with_encoding_and_framerate(path.as_ref(), enc, framerate);
        }

        result
    }

    /// Get framerate used to create timings
    #[must_use]
    pub fn framerate(&self) -> FrameRate {
        self.framerate
    }

    /// Modify framerate associated with subtitle.
    /// Does not modify event timings at all.
    pub fn set_framerate(&mut self, framerate: FrameRate) {
        self.framerate = framerate;
    }

    /// Modify framerate associated with subtitle
    ///
    /// This will also recalculate and update all event timings to match the new framerate.
    pub fn update_framerate(&mut self, framerate: FrameRate) {
        let ratio = self.framerate / framerate;
        for event in &mut self.events {
            event.start =
                Moment::from(((i64::from(event.start) as FrameRate) * ratio).round() as i64);
            event.end = Moment::from(((i64::from(event.end) as FrameRate) * ratio).round() as i64);
        }
        self.framerate = framerate;
    }
}

impl Subtitle for TimedMicroDvdSubtitle {
    type Event = TimedMicroDvdEvent;

    fn from_path_with_encoding(
        path: impl AsRef<std::path::Path>,
        encoding: Option<&'static encoding_rs::Encoding>,
    ) -> Result<Self, crate::Error> {
        let mut enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), Some(30)).ok());
        let mut result = Self::open_file_with_encoding(path.as_ref(), enc);

        if encoding.is_none() && result.is_err() {
            enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), None).ok());
            result = Self::open_file_with_encoding(path.as_ref(), enc);
        }

        result
    }

    fn events(&self) -> &[Self::Event] {
        self.events.as_slice()
    }

    fn events_mut(&mut self) -> &mut [Self::Event] {
        self.events.as_mut_slice()
    }
}

impl TextSubtitle for TimedMicroDvdSubtitle {}

impl TimedSubtitle for TimedMicroDvdSubtitle {}

impl Display for TimedMicroDvdSubtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for event in &self.events {
            writeln!(
                f,
                "{{{}}}{{{}}}{}",
                moment_to_frame(event.start, self.framerate),
                moment_to_frame(event.end, self.framerate),
                event.text
            )?;
        }

        Ok(())
    }
}

impl FromStr for TimedMicroDvdSubtitle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());

        Ok(Self::from_raw(&parse_microdvd(reader), None))
    }
}

impl From<&AssSubtitle> for TimedMicroDvdSubtitle {
    fn from(value: &AssSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .map(|line| TimedMicroDvdEvent {
                    text: line.unformatted_text().replace('\n', "|"),
                    start: line.start,
                    end: line.end,
                })
                .collect(),
            framerate: 24.0,
        }
    }
}

impl From<&SsaSubtitle> for TimedMicroDvdSubtitle {
    fn from(value: &SsaSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .map(|line| TimedMicroDvdEvent {
                    text: line.unformatted_text().replace('\n', "|"),
                    start: line.start,
                    end: line.end,
                })
                .collect(),
            framerate: 24.0,
        }
    }
}

impl From<&SubRipSubtitle> for TimedMicroDvdSubtitle {
    fn from(value: &SubRipSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .map(|line| TimedMicroDvdEvent {
                    text: line.unformatted_text().replace('\n', "|"),
                    start: line.start,
                    end: line.end,
                })
                .collect(),
            framerate: 24.0,
        }
    }
}

impl From<&WebVttSubtitle> for TimedMicroDvdSubtitle {
    fn from(value: &WebVttSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .map(|line| TimedMicroDvdEvent {
                    text: line.unformatted_text().replace('\n', "|"),
                    start: line.start,
                    end: line.end,
                })
                .collect(),
            framerate: 24.0,
        }
    }
}

impl TextEvent for TimedMicroDvdEvent {
    fn unformatted_text(&self) -> Cow<'_, String> {
        Cow::Owned(self.text.replace('|', "\n"))
    }
}

impl TextEventInterface for TimedMicroDvdEvent {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl TimedEvent for TimedMicroDvdEvent {}

impl TimedEventInterface for TimedMicroDvdEvent {
    fn start(&self) -> Moment {
        self.start
    }

    fn end(&self) -> Moment {
        self.end
    }

    fn set_start(&mut self, moment: Moment) {
        self.start = moment;
    }

    fn set_end(&mut self, moment: Moment) {
        self.end = moment;
    }
}

impl From<&TimedMicroDvdSubtitle> for MicroDvdSubtitle {
    fn from(value: &TimedMicroDvdSubtitle) -> Self {
        Self {
            events: value
                .events
                .iter()
                .map(|event| MicroDvdEvent {
                    start: moment_to_frame(event.start, value.framerate),
                    end: moment_to_frame(event.end, value.framerate),
                    text: event.text.clone(),
                })
                .collect(),
        }
    }
}

impl MicroDvdSubtitle {
    /// Create new instance from already existing list of `MicroDvdEvent`s.
    #[must_use]
    pub fn from_events(events: Vec<MicroDvdEvent>) -> Self {
        Self { events }
    }

    fn open_file_with_encoding(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        let file = File::open(path)?;
        let transcoded = DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(file);
        let reader = BufReader::new(transcoded);

        Ok(parse_microdvd(reader))
    }
}

impl Subtitle for MicroDvdSubtitle {
    type Event = MicroDvdEvent;

    fn from_path_with_encoding(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        let mut enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), Some(30)).ok());
        let mut result = Self::open_file_with_encoding(path.as_ref(), enc);

        if encoding.is_none() && result.is_err() {
            enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), None).ok());
            result = Self::open_file_with_encoding(path.as_ref(), enc);
        }

        result
    }

    fn events(&self) -> &[Self::Event] {
        self.events.as_slice()
    }

    fn events_mut(&mut self) -> &mut [Self::Event] {
        self.events.as_mut_slice()
    }
}

impl TextSubtitle for MicroDvdSubtitle {}

impl Display for MicroDvdSubtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for event in &self.events {
            writeln!(f, "{{{}}}{{{}}}{}", event.start, event.end, event.text)?;
        }

        Ok(())
    }
}

impl FromStr for MicroDvdSubtitle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());

        Ok(parse_microdvd(reader))
    }
}

impl TextEvent for MicroDvdEvent {
    fn unformatted_text(&self) -> Cow<'_, String> {
        Cow::Owned(self.text.replace('|', "\n"))
    }
}

impl TextEventInterface for MicroDvdEvent {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}
