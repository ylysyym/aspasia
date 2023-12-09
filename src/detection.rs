use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use encoding_rs::Encoding;
use encoding_rs_io::DecodeReaderBytesBuilder;

use crate::{
    encoding::detect_file_encoding,
    microdvd::parse::parse_microdvd_line,
    subrip::parse::parse_new_line,
    substation::common::parse::{parse_format, parse_script_info_heading},
    webvtt::parse::parse_header,
    Error, Format,
};

/// Attempt to detect subtitle format from its extension first, then by file contents if that fails
///
/// # Errors
///
/// - Returns [`Error::FileIoError`] if an error results while attempting to open the file for encoding detection or content detection.
/// - Returns [`Error::FormatUnknownError`] if unable to conclusively determine a single format.
pub fn detect_format(path: impl AsRef<Path>) -> Result<Format, Error> {
    if let Ok(format) = detect_format_by_extension(path.as_ref()) {
        return Ok(format);
    }

    detect_format_by_content(path.as_ref())
}

/// Attempt to detect subtitle format from its extension first, then by file contents using the given encoding if that fails
///
/// # Errors
///
/// - Returns [`Error::FileIoError`] if an error results while attempting to open the file for content detection.
/// - Returns [`Error::FormatUnknownError`] if unable to conclusively determine a single format.
pub fn detect_format_with_encoding(
    path: impl AsRef<Path>,
    encoding: Option<&'static Encoding>,
) -> Result<Format, Error> {
    if let Ok(format) = detect_format_by_extension(path.as_ref()) {
        return Ok(format);
    }

    detect_format_by_content_with_encoding(path.as_ref(), encoding)
}

/// Attempt to detect subtitle format using its file extension
///
/// # Errors
///
/// Returns [`Error::FormatUnknownError`] if file extension is not recognised
pub fn detect_format_by_extension(path: impl AsRef<Path>) -> Result<Format, Error> {
    let ext = path.as_ref().extension();
    match ext
        .map(std::ffi::OsStr::to_ascii_lowercase)
        .unwrap_or_default()
        .to_str()
    {
        Some("ass") => Ok(Format::Ass),
        Some("ssa") => Ok(Format::Ssa),
        Some("srt") => Ok(Format::SubRip),
        Some("sub") => Ok(Format::MicroDvd),
        Some("vtt") => Ok(Format::WebVtt),
        _ => Err(Error::FormatUnknownError),
    }
}

/// Attempt to detect subtitle format from file contents
///
/// # Errors
///
/// - Returns [`Error::FileIoError`] if an error results while attempting to open the file for encoding detection or format detection.
/// - Returns [`Error::FormatUnknownError`] if unable to conclusively determine a single format.
pub fn detect_format_by_content(path: impl AsRef<Path>) -> Result<Format, Error> {
    let encoding = detect_file_encoding(path.as_ref(), None).ok();

    detect_format_by_content_with_encoding(path, encoding)
}

/// Attempt to detect subtitle format from file contents, using specified encoding to read file
///
/// # Errors
///
/// - Returns [`Error::FileIoError`] if an error results while attempting to open the file for content detection.
/// - Returns [`Error::FormatUnknownError`] if unable to conclusively determine a single format.
pub fn detect_format_by_content_with_encoding(
    path: impl AsRef<Path>,
    encoding: Option<&'static Encoding>,
) -> Result<Format, Error> {
    let file = File::open(path)?;
    let transcoded = DecodeReaderBytesBuilder::new()
        .encoding(encoding)
        .build(file);
    let reader = BufReader::new(transcoded);

    let mut texts = Vec::new();
    let mut counter = 0;
    let lines = reader.lines();
    for line in lines {
        let Ok(line) = line else {
            continue;
        };
        texts.push(line);
        if counter > 30 {
            break;
        }
        counter += 1;
    }

    detect_format_from_str(texts.join("\n").as_str())
}

/// Attempt to detect subtitle format from text of the first few lines of the subtitle
///
/// # Errors
///
/// Returns [`Error::FormatUnknownError`] if unable to conclusively determine a single format.
pub fn detect_format_from_str(text: &str) -> Result<Format, Error> {
    if parse_header(text).is_ok() {
        return Ok(Format::WebVtt);
    }
    if parse_new_line(text).is_ok() {
        return Ok(Format::SubRip);
    }
    if parse_script_info_heading(text).is_ok() {
        if let Ok((_, format)) = parse_format(text) {
            return Ok(format);
        }

        return Ok(Format::Ass);
    }
    if parse_microdvd_line(text).is_ok() {
        return Ok(Format::MicroDvd);
    }

    Err(Error::FormatUnknownError)
}
