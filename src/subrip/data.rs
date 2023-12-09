use std::{borrow::Cow, fmt::Display, fs::File, io::BufReader, path::Path, str::FromStr};

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{
    encoding::detect_file_encoding,
    errors::Error,
    plain::PlainSubtitle,
    substation::{
        ass::convert::ass_to_srt_formatting, common::convert::split_formatting_tags,
        ssa::convert::ssa_to_srt_formatting,
    },
    traits::TimedSubtitle,
    webvtt::convert::vtt_to_srt_formatting,
    AssSubtitle, Moment, SsaSubtitle, Subtitle, TextEvent, TextEventInterface, TextSubtitle,
    TimedEvent, TimedEventInterface, TimedMicroDvdSubtitle, TimedSubtitleFile, WebVttSubtitle,
};

use super::parse::{parse_srt, strip_srt_formatting};

/// SubRip (.srt) subtitle data, containing only a list of events.
#[derive(Clone, Debug)]
pub struct SubRipSubtitle {
    events: Vec<SubRipEvent>,
}

/// SubRip subtitle event
#[derive(Debug, Clone)]
pub struct SubRipEvent {
    /// Line number for the event. Generally this should be a sequential range of numbers, from 1 to however many lines there are.
    pub line_number: usize,
    /// Textual content of the event.
    pub text: String,
    /// Start time of event
    pub start: Moment,
    /// End time of event
    pub end: Moment,
    /// Coordinates for positioning of subtitle text
    pub coordinates: Option<String>,
}

impl SubRipSubtitle {
    /// Creates a new SubRip (.srt) subtitle from an already existing list of `SubRipEvent`s
    #[must_use]
    pub fn from_events(events: Vec<SubRipEvent>) -> Self {
        Self { events }
    }

    /// Renumbers all events according to the order they are stored in.
    ///
    /// This modifies the line number of all events,
    /// starting from 1 for the first event and incrementing by 1 for each subsequent event.
    pub fn renumber(&mut self) {
        for (i, event) in self.events.iter_mut().enumerate() {
            event.line_number = i + 1;
        }
    }

    fn try_from_path_with_encoding(
        path: &Path,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        let file = File::open(path)?;
        let transcoded = DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(file);
        let reader = BufReader::new(transcoded);

        Ok(parse_srt(reader))
    }
}

impl TextSubtitle for SubRipSubtitle {}

impl TimedSubtitle for SubRipSubtitle {}

impl Subtitle for SubRipSubtitle {
    type Event = SubRipEvent;

    fn from_path_with_encoding(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        // Try to detect encoding quickly by only looking at first few lines of the file
        // If an error results, then try detecting encoding using the entire file
        // TODO: look at whether this is worth it (benchmark encoding detection using very large file)
        let mut enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), Some(30)).ok());
        let mut result = Self::try_from_path_with_encoding(path.as_ref(), enc);

        if encoding.is_none() && result.is_err() {
            enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), None).ok());
            result = Self::try_from_path_with_encoding(path.as_ref(), enc);
        }

        result
    }

    fn events(&self) -> &[SubRipEvent] {
        self.events.as_slice()
    }

    fn events_mut(&mut self) -> &mut [SubRipEvent] {
        self.events.as_mut_slice()
    }
}

impl Display for SubRipSubtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, event) in self.events.iter().enumerate() {
            writeln!(f, "{}{}", if i > 0 { "\n" } else { "" }, event,)?;
        }

        Ok(())
    }
}

impl FromStr for SubRipSubtitle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());

        Ok(parse_srt(reader))
    }
}

impl From<&AssSubtitle> for SubRipSubtitle {
    /// Convert Advanced SubStation Alpha (.ass) subtitles to .srt format
    ///
    /// Replaces SubStation newline indicators (\N) with actual newlines.
    /// Additionally, converts .ass style formatting tags to .srt formatting tags.
    /// Currently, bolded, italicised, underlined, or coloured text will be converted to their .srt counterparts.
    /// All other tags will be discarded.
    fn from(value: &AssSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .enumerate()
                .map(|(i, dialogue)| {
                    let mut text = dialogue.text.replace("\\N", "\n");
                    if let Ok((_, separated)) = split_formatting_tags(text.as_str()) {
                        if let Ok((_, converted)) = ass_to_srt_formatting(separated.as_str()) {
                            text = converted;
                        }
                    }

                    SubRipEvent {
                        line_number: i + 1,
                        text,
                        start: dialogue.start,
                        end: dialogue.end,
                        coordinates: None,
                    }
                })
                .collect(),
        }
    }
}

impl From<&TimedMicroDvdSubtitle> for SubRipSubtitle {
    fn from(value: &TimedMicroDvdSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .enumerate()
                .map(|(i, line)| SubRipEvent {
                    line_number: i + 1,
                    text: line.text.replace('|', "\n"),
                    start: line.start,
                    end: line.end,
                    coordinates: None,
                })
                .collect(),
        }
    }
}

impl From<&SsaSubtitle> for SubRipSubtitle {
    /// Convert SubStation Alpha (.ssa) subtitles to .srt format
    ///
    /// Replaces SubStation newline indicators (\N) with actual newlines.
    /// Additionally, converts .ssa style formatting tags to .srt formatting tags.
    /// Currently, bolded, italicised, or coloured text will be converted to their .srt counterparts.
    /// All other tags will be discarded.
    fn from(value: &SsaSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .enumerate()
                .map(|(i, dialogue)| {
                    let mut text = dialogue.text.replace("\\N", "\n");
                    if let Ok((_, separated)) = split_formatting_tags(text.as_str()) {
                        if let Ok((_, converted)) = ssa_to_srt_formatting(separated.as_str()) {
                            text = converted;
                        }
                    }
                    SubRipEvent {
                        line_number: i + 1,
                        text,
                        start: dialogue.start,
                        end: dialogue.end,
                        coordinates: None,
                    }
                })
                .collect(),
        }
    }
}

impl From<&WebVttSubtitle> for SubRipSubtitle {
    /// Convert WebVTT (.vtt) subtitle to .srt format
    ///
    /// Cue identifiers that are numbers will be treated as line numbers for the corresponding event in the .srt output.
    fn from(value: &WebVttSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .enumerate()
                .map(|(i, cue)| {
                    let mut text = cue.text.clone();
                    if let Ok((_, converted)) = vtt_to_srt_formatting(text.as_str()) {
                        text = converted;
                    }

                    let identifier = cue
                        .identifier
                        .as_ref()
                        .and_then(|s| s.parse::<usize>().ok());
                    SubRipEvent {
                        line_number: identifier.unwrap_or(i + 1),
                        text,
                        start: cue.start,
                        end: cue.end,
                        coordinates: None,
                    }
                })
                .collect(),
        }
    }
}

impl From<AssSubtitle> for SubRipSubtitle {
    fn from(value: AssSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<TimedMicroDvdSubtitle> for SubRipSubtitle {
    fn from(value: TimedMicroDvdSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<SsaSubtitle> for SubRipSubtitle {
    fn from(value: SsaSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<WebVttSubtitle> for SubRipSubtitle {
    fn from(value: WebVttSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<PlainSubtitle> for SubRipSubtitle {
    fn from(value: PlainSubtitle) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .enumerate()
                .map(|(i, l)| SubRipEvent {
                    line_number: i + 1,
                    text: l.text.clone(),
                    start: l.start,
                    end: l.end,
                    coordinates: None,
                })
                .collect(),
        }
    }
}

impl From<TimedSubtitleFile> for SubRipSubtitle {
    fn from(value: TimedSubtitleFile) -> Self {
        match value {
            TimedSubtitleFile::SubRip(data) => data,
            TimedSubtitleFile::MicroDvd(data) => data.into(),
            TimedSubtitleFile::Ass(data) => data.into(),
            TimedSubtitleFile::WebVtt(data) => data.into(),
            TimedSubtitleFile::Ssa(data) => data.into(),
        }
    }
}

impl TextEvent for SubRipEvent {
    fn unformatted_text(&self) -> Cow<'_, String> {
        let Ok((_, stripped)) = strip_srt_formatting(self.text.as_str()) else {
            return Cow::Borrowed(&self.text);
        };

        Cow::Owned(stripped)
    }
}

impl TimedEvent for SubRipEvent {}

impl TextEventInterface for SubRipEvent {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl TimedEventInterface for SubRipEvent {
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

impl Display for SubRipEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}\n{} --> {}{}{}\n{}",
            self.line_number,
            self.start.as_srt_timestamp(),
            self.end.as_srt_timestamp(),
            if self.coordinates.is_some() { " " } else { "" },
            self.coordinates.as_deref().unwrap_or_default(),
            self.text,
        )
    }
}
