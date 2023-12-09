use std::path::Path;

use crate::{
    detection::detect_format_with_encoding, encoding::detect_file_encoding, AssSubtitle, Error,
    SsaSubtitle, SubRipSubtitle, Subtitle, TimedMicroDvdSubtitle, WebVttSubtitle,
};

/// Convenience interface for interacting with time-based subtitle files in a generic manner.
///
/// For example, managing a collection of multiple subtitles of unknown or different formats.
///
/// For accessing or modifying format-specific data/methods, such as embedded fonts in SubStation Alpha files,
/// you should convert to the format-specific types using `from()` or `into()`
#[derive(Debug)]
pub enum TimedSubtitleFile {
    /// File in Advanced SubStation Alpha (V4+) (.ass) format
    Ass(AssSubtitle),
    /// Timed version of file in MicroDVD (.sub) format
    MicroDvd(TimedMicroDvdSubtitle),
    /// File in Substation Alpha (V4) (.ssa) format
    Ssa(SsaSubtitle),
    /// File in SubRip (.srt) format
    SubRip(SubRipSubtitle),
    /// File in WebVTT (.vtt) format
    WebVtt(WebVttSubtitle),
}

/// Supported file formats
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// Advanced SubStation Alpha (.ass) subtitle
    Ass,
    /// MicroDVD (.sub) subtitle
    MicroDvd,
    /// SubStation Alpha (.ssa) subtitle
    Ssa,
    /// SubRip (.srt) subtitle
    SubRip,
    /// WebVTT (.vtt) subtitle
    WebVtt,
}

impl TimedSubtitleFile {
    /// Automatically attempts to detect format using the file extension and file contents.
    ///
    /// Using the detected format, try to parse the given path and load its data
    ///
    /// # Errors
    ///
    /// - If an error is encountered while opening the file, returns [`Error::FileIoError`]
    /// - If the format cannot be successfully detected, returns [`Error::FormatUnknownError`]
    pub fn new(path: impl AsRef<Path>) -> Result<Self, Error> {
        let encoding = detect_file_encoding(path.as_ref(), None).ok();
        let format = detect_format_with_encoding(path.as_ref(), encoding)?;

        match format {
            Format::Ass => {
                AssSubtitle::from_path_with_encoding(path.as_ref(), encoding).map(Self::Ass)
            }
            Format::MicroDvd => {
                TimedMicroDvdSubtitle::from_path_with_encoding(path.as_ref(), encoding)
                    .map(Self::MicroDvd)
            }
            Format::Ssa => {
                SsaSubtitle::from_path_with_encoding(path.as_ref(), encoding).map(Self::Ssa)
            }
            Format::SubRip => {
                SubRipSubtitle::from_path_with_encoding(path.as_ref(), encoding).map(Self::SubRip)
            }
            Format::WebVtt => {
                WebVttSubtitle::from_path_with_encoding(path.as_ref(), encoding).map(Self::WebVtt)
            }
        }
    }

    /// Try to load and parse file as the given format
    ///
    /// # Errors
    ///
    /// If an error is encountered while opening the file, returns [`Error::FileIoError`]
    pub fn with_format(path: impl AsRef<Path>, format: Format) -> Result<Self, Error> {
        match format {
            Format::Ass => AssSubtitle::from_path(path.as_ref()).map(Self::Ass),
            Format::MicroDvd => TimedMicroDvdSubtitle::from_path(path.as_ref()).map(Self::MicroDvd),
            Format::Ssa => SsaSubtitle::from_path(path.as_ref()).map(Self::Ssa),
            Format::SubRip => SubRipSubtitle::from_path(path.as_ref()).map(Self::SubRip),
            Format::WebVtt => WebVttSubtitle::from_path(path.as_ref()).map(Self::WebVtt),
        }
    }

    /// Exports contents to file in the corresponding format
    ///
    /// # Errors
    ///
    /// If an error is encountered while creating the file, returns [`Error::FileIoError`]
    pub fn export(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        match self {
            Self::Ass(data) => data.export(path.as_ref()),
            Self::MicroDvd(data) => data.export(path.as_ref()),
            Self::Ssa(data) => data.export(path.as_ref()),
            Self::SubRip(data) => data.export(path.as_ref()),
            Self::WebVtt(data) => data.export(path.as_ref()),
        }
    }
}
