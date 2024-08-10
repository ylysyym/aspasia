use std::{borrow::Cow, fmt::Display, fs::File, io::BufReader, path::Path, str::FromStr};

use buildstructor::Builder;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{
    encoding::detect_file_encoding,
    errors::Error,
    plain::PlainSubtitle,
    subrip::convert::srt_to_vtt_formatting,
    substation::{
        ass::convert::ass_to_vtt_formatting, common::convert::split_formatting_tags,
        ssa::convert::ssa_to_vtt_formatting,
    },
    traits::TimedSubtitle,
    AssSubtitle, Moment, SsaSubtitle, SubRipSubtitle, Subtitle, TextEvent, TextEventInterface,
    TextSubtitle, TimedEvent, TimedEventInterface, TimedMicroDvdSubtitle, TimedSubtitleFile,
};

use super::{convert::strip_html_tags, parse::parse_vtt};

/// WebVTT (.vtt) subtitle data
#[derive(Clone, Debug, Builder)]
pub struct WebVttSubtitle {
    /// Header
    header: Option<String>,
    /// List of cues
    cues: Vec<WebVttCue>,
    /// List of styles (strings)
    styles: Vec<String>,
    /// List of regions (strings)
    regions: Vec<String>,
}

/// WebVTT subtitle cue (event)
#[derive(Clone, Debug)]
pub struct WebVttCue {
    /// WebVTT identifier
    pub identifier: Option<String>,
    /// Text content of cue
    pub text: String,
    /// Cue settings
    pub settings: Option<String>,
    /// Start time of cue
    pub start: Moment,
    /// End time of cue
    pub end: Moment,
}

impl WebVttSubtitle {
    /// Get header
    #[must_use]
    pub fn header(&self) -> Option<&String> {
        self.header.as_ref()
    }

    /// Change header to specified option
    pub fn set_header(&mut self, header: impl Into<Option<String>>) {
        self.header = header.into();
    }

    /// Get list of styles as a slice
    #[must_use]
    pub fn styles(&self) -> &[String] {
        self.styles.as_slice()
    }

    /// Get list of styles as a mutable slice
    pub fn styles_mut(&mut self) -> &mut [String] {
        self.styles.as_mut_slice()
    }

    /// Get style at index
    #[must_use]
    pub fn style(&self, index: usize) -> Option<&String> {
        self.styles.get(index)
    }

    /// Get mutable style at index
    pub fn style_mut(&mut self, index: usize) -> Option<&mut String> {
        self.styles.get_mut(index)
    }

    /// Get list of regions as a slice
    #[must_use]
    pub fn regions(&self) -> &[String] {
        self.regions.as_slice()
    }

    /// Get list of regions as a mutable slice
    pub fn regions_mut(&mut self) -> &mut [String] {
        self.regions.as_mut_slice()
    }

    /// Get region at index
    #[must_use]
    pub fn region(&self, index: usize) -> Option<&String> {
        self.regions.get(index)
    }

    /// Get mutable region at index
    pub fn region_mut(&mut self, index: usize) -> Option<&mut String> {
        self.regions.get_mut(index)
    }

    fn open_file_with_encoding(
        path: &Path,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        let file = File::open(path)?;
        let transcoded = DecodeReaderBytesBuilder::new()
            .encoding(encoding)
            .build(file);
        let reader = BufReader::new(transcoded);

        Ok(parse_vtt(reader))
    }
}

impl Subtitle for WebVttSubtitle {
    type Event = WebVttCue;

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

    fn events(&self) -> &[WebVttCue] {
        self.cues.as_slice()
    }

    fn events_mut(&mut self) -> &mut [WebVttCue] {
        self.cues.as_mut_slice()
    }
}

impl TextSubtitle for WebVttSubtitle {}

impl TimedSubtitle for WebVttSubtitle {}

impl Display for WebVttSubtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "WEBVTT")?;
        if let Some(header) = &self.header {
            writeln!(f, " - {header}")?;
        }
        if !self.styles.is_empty() {
            writeln!(f)?;
            for style in &self.styles {
                writeln!(f, "STYLE\n{style}")?;
            }
        }
        if !self.regions.is_empty() {
            writeln!(f)?;
            for region in &self.regions {
                writeln!(f, "REGION\n{region}")?;
            }
        }
        if !self.cues.is_empty() {
            writeln!(f)?;
            for line in &self.cues {
                writeln!(f)?;
                writeln!(f, "{line}")?;
            }
        }

        Ok(())
    }
}

impl FromStr for WebVttSubtitle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());

        Ok(parse_vtt(reader))
    }
}

impl Default for WebVttSubtitle {
    fn default() -> Self {
        Self::builder().build()
    }
}

impl From<&AssSubtitle> for WebVttSubtitle {
    fn from(value: &AssSubtitle) -> Self {
        Self {
            cues: value
                .events()
                .iter()
                .map(|dialogue| {
                    let mut text = dialogue.text.replace("\\N", "\n");
                    if let Ok((_, separated)) = split_formatting_tags(text.as_str()) {
                        if let Ok((_, converted)) = ass_to_vtt_formatting(separated.as_str()) {
                            text = converted;
                        }
                    }

                    WebVttCue {
                        identifier: None,
                        text,
                        settings: None,
                        start: dialogue.start,
                        end: dialogue.end,
                    }
                })
                .collect(),
            header: value.script_info().title.clone(),
            styles: Vec::new(),
            regions: Vec::new(),
        }
    }
}

impl From<&TimedMicroDvdSubtitle> for WebVttSubtitle {
    fn from(value: &TimedMicroDvdSubtitle) -> Self {
        Self {
            cues: value
                .events()
                .iter()
                .map(|line| WebVttCue {
                    identifier: None,
                    text: line.text.replace('|', "\n"),
                    settings: None,
                    start: line.start,
                    end: line.end,
                })
                .collect(),
            header: None,
            styles: Vec::new(),
            regions: Vec::new(),
        }
    }
}

impl From<&SsaSubtitle> for WebVttSubtitle {
    fn from(value: &SsaSubtitle) -> Self {
        Self {
            cues: value
                .events()
                .iter()
                .map(|dialogue| {
                    let mut text = dialogue.text.replace("\\N", "\n");
                    if let Ok((_, separated)) = split_formatting_tags(text.as_str()) {
                        if let Ok((_, converted)) = ssa_to_vtt_formatting(separated.as_str()) {
                            text = converted;
                        }
                    }
                    WebVttCue {
                        identifier: None,
                        text,
                        settings: None,
                        start: dialogue.start,
                        end: dialogue.end,
                    }
                })
                .collect(),
            header: None,
            styles: Vec::new(),
            regions: Vec::new(),
        }
    }
}

impl From<&SubRipSubtitle> for WebVttSubtitle {
    /// Convert a SubRip (.srt) subtitle to WebVTT format.
    ///
    /// This sets the line number as the cue identifier.
    ///
    /// The supported tags are `<b>`, `<i>`, and `<u>`. Any other tags will be stripped from the text.
    ///
    /// Bracket tags (of the form `{b}`) will be converted to HTML tags (`<b>`).
    fn from(value: &SubRipSubtitle) -> Self {
        Self {
            cues: value
                .events()
                .iter()
                .map(|line| {
                    let result = srt_to_vtt_formatting(line.text.as_str());
                    let text = result.map_or_else(|_| line.text.clone(), |(_, s)| s);

                    WebVttCue {
                        identifier: Some(line.line_number.to_string()),
                        text,
                        settings: None,
                        start: line.start,
                        end: line.end,
                    }
                })
                .collect(),
            header: None,
            styles: Vec::new(),
            regions: Vec::new(),
        }
    }
}

impl From<AssSubtitle> for WebVttSubtitle {
    fn from(value: AssSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<TimedMicroDvdSubtitle> for WebVttSubtitle {
    fn from(value: TimedMicroDvdSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<SsaSubtitle> for WebVttSubtitle {
    fn from(value: SsaSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<SubRipSubtitle> for WebVttSubtitle {
    fn from(value: SubRipSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<TimedSubtitleFile> for WebVttSubtitle {
    fn from(value: TimedSubtitleFile) -> Self {
        match value {
            TimedSubtitleFile::WebVtt(data) => data,
            TimedSubtitleFile::Ass(data) => data.into(),
            TimedSubtitleFile::MicroDvd(data) => data.into(),
            TimedSubtitleFile::Ssa(data) => data.into(),
            TimedSubtitleFile::SubRip(data) => data.into(),
        }
    }
}

impl From<PlainSubtitle> for WebVttSubtitle {
    fn from(value: PlainSubtitle) -> Self {
        Self {
            cues: value
                .events()
                .iter()
                .map(|event| WebVttCue {
                    identifier: None,
                    text: event.text.clone(),
                    settings: None,
                    start: event.start,
                    end: event.end,
                })
                .collect(),
            header: None,
            styles: Vec::new(),
            regions: Vec::new(),
        }
    }
}

impl TextEvent for WebVttCue {
    fn unformatted_text(&self) -> Cow<'_, String> {
        let Ok((_, stripped)) = strip_html_tags(self.text.as_str()) else {
            return Cow::Borrowed(&self.text);
        };

        Cow::Owned(stripped)
    }
}

impl TimedEvent for WebVttCue {}

impl TextEventInterface for WebVttCue {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl TimedEventInterface for WebVttCue {
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

impl Display for WebVttCue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{} --> {}{}\n{}",
            self.identifier.clone().unwrap_or_default(),
            if self.identifier.is_some() { "\n" } else { "" },
            self.start.as_vtt_timestamp(),
            self.end.as_vtt_timestamp(),
            self.settings.as_deref().unwrap_or_default(),
            self.text,
        )
    }
}
