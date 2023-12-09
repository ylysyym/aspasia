use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use chardetng::EncodingDetector;
use encoding_rs::Encoding;

use crate::errors::Error;

pub(crate) fn detect_file_encoding(
    path: &Path,
    line_limit: Option<usize>,
) -> Result<&'static Encoding, Error> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut detector = EncodingDetector::new();
    let mut buffer: Vec<u8> = Vec::new();

    let mut counter = 0;
    while reader.read_until(b'\n', &mut buffer).is_ok() {
        if buffer.is_empty() {
            break;
        }
        detector.feed(buffer.as_slice(), false);
        buffer.clear();

        if let Some(lines) = line_limit {
            counter += 1;
            if counter >= lines && detector.guess_assess(None, true).1 {
                break;
            }
        }
    }

    Ok(detector.guess(None, true))
}
