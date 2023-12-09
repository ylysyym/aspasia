use std::{borrow::Cow, fmt::Display, fs::File, io::BufReader, path::Path, str::FromStr};

use buildstructor::Builder;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{
    encoding::detect_file_encoding,
    errors::Error,
    plain::PlainSubtitle,
    subrip::convert::srt_to_ssa_formatting,
    substation::common::data::{SubStationEventKind, SubStationFont, SubStationGraphic},
    traits::TimedSubtitle,
    webvtt::convert::vtt_to_ass_formatting,
    AssSubtitle, Moment, SubRipSubtitle, Subtitle, TextEvent, TextEventInterface, TextSubtitle,
    TimedEvent, TimedEventInterface, TimedMicroDvdSubtitle, TimedSubtitleFile, WebVttSubtitle,
};

use super::{convert::strip_formatting_tags, parse::parse_ssa};

/// SubStation Alpha v4 (.ssa) subtitle
#[derive(Debug, Builder)]
pub struct SsaSubtitle {
    /// Script info
    script_info: SsaScriptInfo,
    // Store different event types separately so that we can return dialogue only without having to filter
    /// Dialogue events
    dialogue: Vec<SsaEvent>,
    /// Picture events
    pictures: Vec<SsaEvent>,
    /// Sound events
    sounds: Vec<SsaEvent>,
    /// Movie events
    movies: Vec<SsaEvent>,
    /// Command events
    commands: Vec<SsaEvent>,
    /// Styles
    styles: Vec<SsaStyle>,
    /// Embedded font data
    fonts: Vec<SubStationFont>,
    /// Embedded graphics data
    graphics: Vec<SubStationGraphic>,
}

/// Event in SubStation Alpha (.ssa) file
#[derive(Debug)]
pub struct SsaEvent {
    /// Kind of event, for example dialogue
    pub kind: SubStationEventKind,
    /// Whether event is shown as marked or not
    pub marked: bool,
    /// Start time of event
    pub start: Moment,
    /// End time of event
    pub end: Moment,
    /// Style name for event
    pub style: Option<String>,
    /// Name of speaker, if relevant
    pub name: Option<String>,
    /// Left margin
    pub margin_l: i64,
    /// Right margin
    pub margin_r: i64,
    /// Vertical margin
    pub margin_v: i64,
    /// Effect
    pub effect: Option<String>,
    /// Associated text. For dialogue events, this is the text that is shown on screen.
    /// For other events, this is the path to a media file or a command to run.
    pub text: String,
}

/// Information for the `[ScriptInfo]` section of an SubStation Alpha (.ssa) subtitle.
/// It should always be the first thing shown in an .ssa format subtitle.
#[derive(Debug, Builder)]
pub struct SsaScriptInfo {
    /// Title/description for the subtitle
    pub title: Option<String>,
    /// Original author of subtitle
    pub original_script: Option<String>,
    /// Original translator of subtitle
    pub original_translation: Option<String>,
    /// Original editor of subtitle
    pub original_editing: Option<String>,
    /// Describes who originally timed the subtitle
    pub original_timing: Option<String>,
    /// Where subtitle should start from
    pub synch_point: Option<String>,
    /// Describes who updated the subtitle apart from the original creator(s)
    pub script_updated_by: Option<String>,
    /// Used to describe the details of what was updated, if the subtitle was updated
    pub update_details: Option<String>,
    /// Version of SubStation Alpha subtitle.
    /// For SubStation Alpha (.ssa) format subtitles, the value should be V4.00+
    pub script_type: Option<String>,
    /// Rules for resolving collision issues between on-screen content
    pub collisions: Option<String>,
    /// Height of screen
    pub play_res_y: Option<String>,
    /// Width of screen
    pub play_res_x: Option<String>,
    /// Colour depth
    pub play_depth: Option<String>,
    /// Time scale, with 100 representing the original speed
    pub timer: Option<String>,
}

/// Style in a .ssa file
#[derive(Debug)]
pub struct SsaStyle {
    /// Name of style
    pub name: String,
    /// Name of font used to display text
    pub fontname: String,
    /// Font size of text
    pub fontsize: i64,
    /// Colour that text will be rendered as
    pub primary_colour: String,
    /// Colour that text will be rendered as in the case it is moved due to a collision
    pub secondary_colour: String,
    /// Colour that text will be rendered as in the case it is moved due to a collision with text that appears in the secondary colour
    pub tertiary_colour: String,
    /// Colour of text shadow or text outline
    pub back_colour: String,
    /// Whether text is bolded
    pub bold: bool,
    /// Whether text is italicised
    pub italic: bool,
    /// Style of text border
    pub border_style: i64,
    /// Width of text outline
    pub outline: i64,
    /// Depth of text shadow
    pub shadow: i64,
    /// Alignment of text on screen
    pub alignment: i64,
    /// Left margin in pixels
    pub margin_l: i64,
    /// Right margin in pixels
    pub margin_r: i64,
    /// Vertical margin in pixels
    pub margin_v: i64,
    /// Transparency of text
    pub alpha_level: i64,
    /// Encoding of text represented as a number
    pub encoding: i64,
}

impl SsaSubtitle {
    /// Get list of picture events as a slice
    #[must_use]
    pub fn pictures(&self) -> &[SsaEvent] {
        self.pictures.as_slice()
    }

    /// Get list of picture events as a mutable slice
    pub fn pictures_mut(&mut self) -> &mut [SsaEvent] {
        self.pictures.as_mut_slice()
    }

    /// Get picture event at given index
    #[must_use]
    pub fn picture(&self, index: usize) -> Option<&SsaEvent> {
        self.pictures.get(index)
    }

    /// Get mutable picture event at given index
    pub fn picture_mut(&mut self, index: usize) -> Option<&mut SsaEvent> {
        self.pictures.get_mut(index)
    }

    /// Get list of sound events as a slice
    #[must_use]
    pub fn sounds(&self) -> &[SsaEvent] {
        self.sounds.as_slice()
    }

    /// Get list of sound events as a mutable slice
    pub fn sounds_mut(&mut self) -> &mut [SsaEvent] {
        self.sounds.as_mut_slice()
    }

    /// Get sound event at specified index
    #[must_use]
    pub fn sound(&self, index: usize) -> Option<&SsaEvent> {
        self.sounds.get(index)
    }

    /// Get mutable sound event at specified index
    pub fn sound_mut(&mut self, index: usize) -> Option<&mut SsaEvent> {
        self.sounds.get_mut(index)
    }

    /// Get list of movie events as a slice
    #[must_use]
    pub fn movies(&self) -> &[SsaEvent] {
        self.movies.as_slice()
    }

    /// Get list of movie events as a mutable slice
    pub fn movies_mut(&mut self) -> &mut [SsaEvent] {
        self.movies.as_mut_slice()
    }

    /// Get movie event at specified index
    #[must_use]
    pub fn movie(&self, index: usize) -> Option<&SsaEvent> {
        self.movies.get(index)
    }

    /// Get mutable movie event at specified index
    pub fn movie_mut(&mut self, index: usize) -> Option<&mut SsaEvent> {
        self.movies.get_mut(index)
    }

    /// Get list of command events as a slice
    #[must_use]
    pub fn commands(&self) -> &[SsaEvent] {
        self.commands.as_slice()
    }

    /// Get list of command events as a mutable slice
    pub fn commands_mut(&mut self) -> &mut [SsaEvent] {
        self.commands.as_mut_slice()
    }

    /// Get command event at given index
    #[must_use]
    pub fn command(&self, index: usize) -> Option<&SsaEvent> {
        self.commands.get(index)
    }

    /// Get mutable command event at given index
    pub fn command_mut(&mut self, index: usize) -> Option<&mut SsaEvent> {
        self.commands.get_mut(index)
    }

    /// Get script info struct
    #[must_use]
    pub fn script_info(&self) -> &SsaScriptInfo {
        &self.script_info
    }

    /// Get mutable script info struct
    pub fn script_info_mut(&mut self) -> &mut SsaScriptInfo {
        &mut self.script_info
    }

    /// Get list of styles as a slice
    #[must_use]
    pub fn styles(&self) -> &[SsaStyle] {
        self.styles.as_slice()
    }

    /// Get list of styles as a mutable slice
    pub fn styles_mut(&mut self) -> &mut [SsaStyle] {
        self.styles.as_mut_slice()
    }

    /// Get list of fonts as a slice
    #[must_use]
    pub fn fonts(&self) -> &[SubStationFont] {
        self.fonts.as_slice()
    }

    /// Get list of fonts as a mutable slice
    pub fn fonts_mut(&mut self) -> &mut [SubStationFont] {
        self.fonts.as_mut_slice()
    }

    /// Get list of graphics as a slice
    #[must_use]
    pub fn graphics(&self) -> &[SubStationGraphic] {
        self.graphics.as_slice()
    }

    /// Get list of graphics as a mutable slice
    pub fn graphics_mut(&mut self) -> &mut [SubStationGraphic] {
        self.graphics.as_mut_slice()
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

        Ok(parse_ssa(reader))
    }
}

impl TimedSubtitle for SsaSubtitle {}

impl Subtitle for SsaSubtitle {
    type Event = SsaEvent;

    fn from_path_with_encoding(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error> {
        let mut enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), Some(40)).ok());
        let mut result = Self::open_file_with_encoding(path.as_ref(), enc);

        if encoding.is_none() && result.is_err() {
            enc = encoding.or_else(|| detect_file_encoding(path.as_ref(), None).ok());
            result = Self::open_file_with_encoding(path.as_ref(), enc);
        }

        result
    }

    fn events(&self) -> &[SsaEvent] {
        self.dialogue.as_slice()
    }

    fn events_mut(&mut self) -> &mut [SsaEvent] {
        self.dialogue.as_mut_slice()
    }
}

impl TextSubtitle for SsaSubtitle {
    /// Strip formatting tags from lines in addition to deleting styles
    fn strip_formatting(&mut self) {
        for event in self.events_mut() {
            event.strip_formatting();
        }
        self.styles.clear();
    }
}

impl Display for SsaSubtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.script_info)?;
        writeln!(f)?;
        if !self.styles.is_empty() {
            writeln!(f, "[V4 Styles]")?;
            writeln!(f, "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, TertiaryColour, BackColour, Bold, Italic, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, AlphaLevel, Encoding")?;
            for style in &self.styles {
                writeln!(f, "{style}")?;
            }
            writeln!(f)?;
        }
        if !self.fonts.is_empty() {
            writeln!(f)?;
            for font in &self.fonts {
                writeln!(f, "{font}")?;
            }
        }
        if !self.graphics.is_empty() {
            writeln!(f)?;
            for graphic in &self.graphics {
                writeln!(f, "{graphic}")?;
            }
        }
        if !self.dialogue.is_empty() {
            writeln!(f, "[Events]")?;
            writeln!(
                f,
                "Format: Marked, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"
            )?;
            for event in &self.dialogue {
                writeln!(f, "{event}")?;
            }
            for event in &self.pictures {
                writeln!(f, "{event}")?;
            }
            for event in &self.sounds {
                writeln!(f, "{event}")?;
            }
            for event in &self.movies {
                writeln!(f, "{event}")?;
            }
            for event in &self.commands {
                writeln!(f, "{event}")?;
            }
        }

        Ok(())
    }
}

impl FromStr for SsaSubtitle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());

        Ok(parse_ssa(reader))
    }
}

impl Default for SsaSubtitle {
    fn default() -> Self {
        Self::builder()
            .script_info(SsaScriptInfo::default())
            .build()
    }
}

// TODO convert styles etc
impl From<&AssSubtitle> for SsaSubtitle {
    fn from(value: &AssSubtitle) -> Self {
        SsaSubtitle::builder()
            .script_info(SsaScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|event| SsaEvent {
                        kind: SubStationEventKind::Dialogue,
                        marked: false,
                        start: event.start,
                        end: event.end,
                        style: event.style.clone(),
                        name: event.name.clone(),
                        margin_l: event.margin_l,
                        margin_r: event.margin_r,
                        margin_v: event.margin_v,
                        effect: event.effect.clone(),
                        text: event.text.clone(),
                    })
                    .collect(),
            )
            .build()
    }
}

impl From<&TimedMicroDvdSubtitle> for SsaSubtitle {
    fn from(value: &TimedMicroDvdSubtitle) -> Self {
        SsaSubtitle::builder()
            .script_info(SsaScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|line| SsaEvent {
                        kind: SubStationEventKind::Dialogue,
                        marked: false,
                        start: line.start,
                        end: line.end,
                        style: None,
                        name: None,
                        margin_l: 0,
                        margin_r: 0,
                        margin_v: 0,
                        effect: None,
                        text: line.text.replace('|', "\\N"),
                    })
                    .collect(),
            )
            .build()
    }
}

impl From<&SubRipSubtitle> for SsaSubtitle {
    /// Convert SubRip (.srt) subtitle to .ssa format.
    ///
    /// This will replace newlines and convert HTML formatting tags (`<b>`, `<i>`, etc.) into .ssa style formatting tags
    fn from(value: &SubRipSubtitle) -> Self {
        SsaSubtitle::builder()
            .script_info(SsaScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|line| {
                        let mut text = line.text.replace('\n', "\\N");
                        if let Ok((_, converted)) = srt_to_ssa_formatting(text.as_str()) {
                            text = converted;
                        }
                        SsaEvent {
                            kind: SubStationEventKind::Dialogue,
                            marked: false,
                            start: line.start,
                            end: line.end,
                            style: None,
                            name: None,
                            margin_l: 0,
                            margin_r: 0,
                            margin_v: 0,
                            effect: None,
                            text,
                        }
                    })
                    .collect(),
            )
            .build()
    }
}

impl From<&WebVttSubtitle> for SsaSubtitle {
    /// Convert WebVTT (.vtt) format subtitle to .ssa format.
    ///
    /// For each line, this will convert newlines into the appropriate representation,
    /// and it will also convert the basic HTML formatting tags (`<b>`, `<i>`, and `<u>`).
    ///
    /// All other tags and styles are discarded.
    fn from(value: &WebVttSubtitle) -> Self {
        SsaSubtitle::builder()
            .script_info(SsaScriptInfo::builder().and_title(value.header()).build())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|cue| {
                        let mut text = cue.text.replace('\n', "\\N");
                        if let Ok((_, converted)) = vtt_to_ass_formatting(text.as_str()) {
                            text = converted;
                        }
                        SsaEvent {
                            kind: SubStationEventKind::Dialogue,
                            marked: false,
                            start: cue.start,
                            end: cue.end,
                            style: None,
                            name: None,
                            margin_l: 0,
                            margin_r: 0,
                            margin_v: 0,
                            effect: None,
                            text,
                        }
                    })
                    .collect(),
            )
            .build()
    }
}

impl From<AssSubtitle> for SsaSubtitle {
    fn from(value: AssSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<TimedMicroDvdSubtitle> for SsaSubtitle {
    fn from(value: TimedMicroDvdSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<SubRipSubtitle> for SsaSubtitle {
    fn from(value: SubRipSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<WebVttSubtitle> for SsaSubtitle {
    fn from(value: WebVttSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<TimedSubtitleFile> for SsaSubtitle {
    fn from(value: TimedSubtitleFile) -> Self {
        match value {
            TimedSubtitleFile::Ssa(data) => data,
            TimedSubtitleFile::Ass(data) => data.into(),
            TimedSubtitleFile::MicroDvd(data) => data.into(),
            TimedSubtitleFile::SubRip(data) => data.into(),
            TimedSubtitleFile::WebVtt(data) => data.into(),
        }
    }
}

impl From<PlainSubtitle> for SsaSubtitle {
    fn from(value: PlainSubtitle) -> Self {
        SsaSubtitle::builder()
            .script_info(SsaScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|event| SsaEvent {
                        kind: SubStationEventKind::Dialogue,
                        marked: false,
                        start: event.start,
                        end: event.end,
                        style: None,
                        name: None,
                        margin_l: 0,
                        margin_r: 0,
                        margin_v: 0,
                        effect: None,
                        text: event.text.replace('\n', "\\N"),
                    })
                    .collect(),
            )
            .build()
    }
}

impl TextEvent for SsaEvent {
    fn unformatted_text(&self) -> Cow<'_, String> {
        let Ok((_, stripped)) = strip_formatting_tags(self.text.as_str()) else {
            return Cow::Borrowed(&self.text);
        };

        Cow::Owned(stripped)
    }

    fn as_plaintext(&self) -> Cow<'_, String> {
        Cow::Owned(self.unformatted_text().replace("\\N", "\n"))
    }
}

impl TimedEvent for SsaEvent {}

impl TextEventInterface for SsaEvent {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl TimedEventInterface for SsaEvent {
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

impl Display for SsaEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: Marked={},{},{},{},{},{},{},{},{},{}",
            self.kind,
            i32::from(self.marked),
            self.start.as_substation_timestamp(),
            self.end.as_substation_timestamp(),
            self.style.as_deref().unwrap_or_default(),
            self.name.as_deref().unwrap_or_default(),
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.effect.as_deref().unwrap_or_default(),
            self.text
        )
    }
}

impl Display for SsaScriptInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Script Info]")?;
        if let Some(title) = &self.title {
            write!(f, "\nTitle: {title}")?;
        }
        if let Some(original_script) = &self.original_script {
            write!(f, "\nOriginal Script: {original_script}")?;
        }
        if let Some(original_translation) = &self.original_translation {
            write!(f, "\nOriginal Translation: {original_translation}")?;
        }
        if let Some(original_editing) = &self.original_editing {
            write!(f, "\nOriginal Editing: {original_editing}")?;
        }
        if let Some(original_timing) = &self.original_timing {
            write!(f, "\nOriginal Timing: {original_timing}")?;
        }
        if let Some(synch_point) = &self.synch_point {
            write!(f, "\nSynch Point: {synch_point}")?;
        }
        if let Some(script_updated_by) = &self.script_updated_by {
            write!(f, "\nScript Updated By: {script_updated_by}")?;
        }
        if let Some(update_details) = &self.update_details {
            write!(f, "\nUpdate Details: {update_details}")?;
        }
        if let Some(script_type) = &self.script_type {
            write!(f, "\nScript Type: {script_type}")?;
        }
        if let Some(collisions) = &self.collisions {
            write!(f, "\nCollisions: {collisions}")?;
        }
        if let Some(play_res_y) = &self.play_res_y {
            write!(f, "\nPlayResY: {play_res_y}")?;
        }
        if let Some(play_res_x) = &self.play_res_x {
            write!(f, "\nPlayResX: {play_res_x}")?;
        }
        if let Some(play_depth) = &self.play_depth {
            write!(f, "\nPlayDepth: {play_depth}")?;
        }
        if let Some(timer) = &self.timer {
            write!(f, "\nTimer: {timer}")?;
        }

        Ok(())
    }
}

impl Default for SsaScriptInfo {
    fn default() -> Self {
        SsaScriptInfo::builder().script_type("v4.00").build()
    }
}

impl Display for SsaStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.name,
            self.fontname,
            self.fontsize,
            self.primary_colour,
            self.secondary_colour,
            self.tertiary_colour,
            self.back_colour,
            self.bold,
            self.italic,
            self.border_style,
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.alpha_level,
            self.encoding
        )
    }
}
