use std::{borrow::Cow, fmt::Display, fs::File, io::BufReader, path::Path, str::FromStr};

use buildstructor::Builder;
use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{
    encoding::detect_file_encoding,
    errors::Error,
    plain::PlainSubtitle,
    substation::common::data::{SubStationEventKind, SubStationFont, SubStationGraphic},
    traits::TimedSubtitle,
    Moment, SsaSubtitle, SubRipSubtitle, Subtitle, TextEvent, TextEventInterface, TextSubtitle,
    TimedEvent, TimedEventInterface, TimedMicroDvdSubtitle, TimedSubtitleFile, WebVttSubtitle,
};

use super::{convert::strip_formatting_tags, parse::parse_ass};

/// Advanced SubStation Alpha v4+ (.ass) subtitle
#[derive(Debug, Builder)]
pub struct AssSubtitle {
    /// Script info
    script_info: AssScriptInfo,
    // Store different event types separately so that we can return dialogue only without having to filter
    /// Dialogue events
    dialogue: Vec<AssEvent>,
    /// Picture events
    pictures: Vec<AssEvent>,
    /// Sound events
    sounds: Vec<AssEvent>,
    /// Movie events
    movies: Vec<AssEvent>,
    /// Command events
    commands: Vec<AssEvent>,
    /// Styles
    styles: Vec<AssStyle>,
    /// Embedded font data
    fonts: Vec<SubStationFont>,
    /// Embedded graphics data
    graphics: Vec<SubStationGraphic>,
}

/// Advanced Substation Alpha (.ass) event
#[derive(Debug)]
pub struct AssEvent {
    /// Kind of event, for example dialogue
    pub kind: SubStationEventKind,
    /// Events with higher layers will be displayed over those on lower layers.
    /// Events that share the same layer will use different rules to resolve collisions.
    pub layer: i64,
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

/// Information for the `[ScriptInfo]` section of an Advanced SubStation Alpha (.ass) subtitle.
///
/// It should always be the first thing shown in an .ass format subtitle.
#[derive(Debug, Builder)]
pub struct AssScriptInfo {
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
    /// For Advanced SubStation Alpha (.ass) format subtitles, the value should be V4.00+
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
    /// Defines wrapping rules for text
    pub wrap_style: Option<String>,
}

/// Style in a .ass file
#[derive(Debug)]
pub struct AssStyle {
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
    pub outline_colour: String,
    /// Colour of text shadow or text outline
    pub back_colour: String,
    /// Whether text is bolded
    pub bold: bool,
    /// Whether text is italicised
    pub italic: bool,
    /// Whether text is underlined
    pub underline: bool,
    /// Whether text is stricken out
    pub strike_out: bool,
    /// Width scale of text
    pub scale_x: i64,
    /// Height scale of text
    pub scale_y: i64,
    /// Font spacing of text
    pub spacing: i64,
    /// Number of degrees to rotate text by
    pub angle: f64,
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
    /// Encoding of text represented as a number
    pub encoding: i64,
}

impl AssSubtitle {
    /// Get list of picture events as a slice
    #[must_use]
    pub fn pictures(&self) -> &[AssEvent] {
        self.pictures.as_slice()
    }

    /// Get list of picture events as a mutable slice
    pub fn pictures_mut(&mut self) -> &mut [AssEvent] {
        self.pictures.as_mut_slice()
    }

    /// Get picture event at given index
    #[must_use]
    pub fn picture(&self, index: usize) -> Option<&AssEvent> {
        self.pictures.get(index)
    }

    /// Get mutable picture event at given index
    pub fn picture_mut(&mut self, index: usize) -> Option<&mut AssEvent> {
        self.pictures.get_mut(index)
    }

    /// Get list of sound events as a slice
    #[must_use]
    pub fn sounds(&self) -> &[AssEvent] {
        self.sounds.as_slice()
    }

    /// Get list of sound events as a mutable slice
    pub fn sounds_mut(&mut self) -> &mut [AssEvent] {
        self.sounds.as_mut_slice()
    }

    /// Get sound event at specified index
    #[must_use]
    pub fn sound(&self, index: usize) -> Option<&AssEvent> {
        self.sounds.get(index)
    }

    /// Get mutable sound event at specified index
    pub fn sound_mut(&mut self, index: usize) -> Option<&mut AssEvent> {
        self.sounds.get_mut(index)
    }

    /// Get list of movie events as a slice
    #[must_use]
    pub fn movies(&self) -> &[AssEvent] {
        self.movies.as_slice()
    }

    /// Get list of movie events as a mutable slice
    pub fn movies_mut(&mut self) -> &mut [AssEvent] {
        self.movies.as_mut_slice()
    }

    /// Get movie event at specified index
    #[must_use]
    pub fn movie(&self, index: usize) -> Option<&AssEvent> {
        self.movies.get(index)
    }

    /// Get mutable movie event at specified index
    pub fn movie_mut(&mut self, index: usize) -> Option<&mut AssEvent> {
        self.movies.get_mut(index)
    }

    /// Get list of command events as a slice
    #[must_use]
    pub fn commands(&self) -> &[AssEvent] {
        self.commands.as_slice()
    }

    /// Get list of command events as a mutable slice
    pub fn commands_mut(&mut self) -> &mut [AssEvent] {
        self.commands.as_mut_slice()
    }

    /// Get command event at given index
    #[must_use]
    pub fn command(&self, index: usize) -> Option<&AssEvent> {
        self.commands.get(index)
    }

    /// Get mutable command event at given index
    pub fn command_mut(&mut self, index: usize) -> Option<&mut AssEvent> {
        self.commands.get_mut(index)
    }

    /// Get script info struct
    #[must_use]
    pub fn script_info(&self) -> &AssScriptInfo {
        &self.script_info
    }

    /// Get mutable script info struct
    pub fn script_info_mut(&mut self) -> &mut AssScriptInfo {
        &mut self.script_info
    }

    /// Get list of styles as a slice
    #[must_use]
    pub fn styles(&self) -> &[AssStyle] {
        self.styles.as_slice()
    }

    /// Get list of styles as a mutable slice
    pub fn styles_mut(&mut self) -> &mut [AssStyle] {
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

        Ok(parse_ass(reader))
    }
}

impl Subtitle for AssSubtitle {
    type Event = AssEvent;

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

    fn events(&self) -> &[AssEvent] {
        self.dialogue.as_slice()
    }

    fn events_mut(&mut self) -> &mut [AssEvent] {
        self.dialogue.as_mut_slice()
    }
}

impl TextSubtitle for AssSubtitle {
    /// Strip formatting tags from lines in addition to deleting styles
    fn strip_formatting(&mut self) {
        for event in self.events_mut() {
            event.strip_formatting();
        }
        self.styles.clear();
    }
}

impl TimedSubtitle for AssSubtitle {}

impl Display for AssSubtitle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.script_info)?;
        writeln!(f)?;
        if !self.styles.is_empty() {
            writeln!(f, "[V4+ Styles]")?;
            writeln!(f, "Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding")?;
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
                "Format: Layer, Start, End, Style, Actor, MarginL, MarginR, MarginV, Effect, Text"
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

impl FromStr for AssSubtitle {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let reader = BufReader::new(s.as_bytes());

        Ok(parse_ass(reader))
    }
}

impl Default for AssSubtitle {
    fn default() -> Self {
        Self::builder()
            .script_info(AssScriptInfo::default())
            .build()
    }
}

impl From<&TimedMicroDvdSubtitle> for AssSubtitle {
    fn from(value: &TimedMicroDvdSubtitle) -> Self {
        AssSubtitle::builder()
            .script_info(AssScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|line| AssEvent {
                        kind: SubStationEventKind::Dialogue,
                        layer: 0,
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

// TODO: convert styles, etc
impl From<&SsaSubtitle> for AssSubtitle {
    fn from(value: &SsaSubtitle) -> Self {
        AssSubtitle::builder()
            .script_info(AssScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|event| AssEvent {
                        kind: SubStationEventKind::Dialogue,
                        layer: 0,
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

impl From<&SubRipSubtitle> for AssSubtitle {
    /// Convert SubRip (.srt) subtitle to .ass format.
    ///
    /// This will replace newlines and convert HTML formatting tags (`<b>`, `<i>`, etc.) into .ass style formatting tags
    fn from(value: &SubRipSubtitle) -> Self {
        AssSubtitle::builder()
            .script_info(AssScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|line| {
                        let mut text = line.text.replace('\n', "\\N");
                        if let Ok((_, converted)) =
                            crate::subrip::convert::convert_to_ass_formatting(text.as_str())
                        {
                            text = converted;
                        }
                        AssEvent {
                            kind: SubStationEventKind::Dialogue,
                            layer: 0,
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

impl From<&WebVttSubtitle> for AssSubtitle {
    /// Convert WebVTT (.vtt) format subtitle to .ass format.
    ///
    /// For each line, this will convert newlines into the appropriate representation,
    /// and it will also convert the basic HTML formatting tags (`<b>`, `<i>`, and `<u>`).
    ///
    /// All other tags and styles are discarded.
    fn from(value: &WebVttSubtitle) -> Self {
        AssSubtitle::builder()
            .script_info(AssScriptInfo::builder().and_title(value.header()).build())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|cue| {
                        let mut text = cue.text.replace('\n', "\\N");
                        if let Ok((_, converted)) =
                            crate::webvtt::convert::convert_to_ass_formatting(text.as_str())
                        {
                            text = converted;
                        }
                        AssEvent {
                            kind: SubStationEventKind::Dialogue,
                            layer: 0,
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

impl From<TimedMicroDvdSubtitle> for AssSubtitle {
    fn from(value: TimedMicroDvdSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<SsaSubtitle> for AssSubtitle {
    fn from(value: SsaSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<SubRipSubtitle> for AssSubtitle {
    fn from(value: SubRipSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<WebVttSubtitle> for AssSubtitle {
    fn from(value: WebVttSubtitle) -> Self {
        Self::from(&value)
    }
}

impl From<TimedSubtitleFile> for AssSubtitle {
    fn from(value: TimedSubtitleFile) -> Self {
        match value {
            TimedSubtitleFile::Ass(data) => data,
            TimedSubtitleFile::MicroDvd(data) => data.into(),
            TimedSubtitleFile::Ssa(data) => data.into(),
            TimedSubtitleFile::SubRip(data) => data.into(),
            TimedSubtitleFile::WebVtt(data) => data.into(),
        }
    }
}

impl From<PlainSubtitle> for AssSubtitle {
    fn from(value: PlainSubtitle) -> Self {
        AssSubtitle::builder()
            .script_info(AssScriptInfo::default())
            .dialogue(
                value
                    .events()
                    .iter()
                    .map(|event| AssEvent {
                        kind: SubStationEventKind::Dialogue,
                        layer: 0,
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

impl TextEvent for AssEvent {
    fn unformatted_text(&self) -> Cow<'_, String> {
        let Ok((_, stripped)) = strip_formatting_tags(self.text.as_str()) else {
            return Cow::Borrowed(&self.text);
        };

        Cow::Owned(stripped)
    }
}

impl TimedEvent for AssEvent {}

impl TextEventInterface for AssEvent {
    fn text(&self) -> String {
        self.text.clone()
    }

    fn set_text(&mut self, text: String) {
        self.text = text;
    }
}

impl TimedEventInterface for AssEvent {
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

impl Display for AssEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {},{},{},{},{},{},{},{},{},{}",
            self.kind,
            self.layer,
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

impl Display for AssScriptInfo {
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
        if let Some(wrap_style) = &self.wrap_style {
            write!(f, "\nWrapStyle: {wrap_style}")?;
        }

        Ok(())
    }
}

impl Default for AssScriptInfo {
    fn default() -> Self {
        AssScriptInfo::builder().script_type("v4.00+").build()
    }
}

impl Display for AssStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            self.name,
            self.fontname,
            self.fontsize,
            self.primary_colour,
            self.secondary_colour,
            self.outline_colour,
            self.back_colour,
            self.bold,
            self.italic,
            self.underline,
            self.strike_out,
            self.scale_x,
            self.scale_y,
            self.spacing,
            self.angle,
            self.border_style,
            self.outline,
            self.shadow,
            self.alignment,
            self.margin_l,
            self.margin_r,
            self.margin_v,
            self.encoding
        )
    }
}
