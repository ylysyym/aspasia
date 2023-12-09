//! aspasia is a subtitle parsing library for Rust

#![warn(
    clippy::all,
    clippy::cargo,
    clippy::pedantic,
    missing_docs,
    clippy::perf,
    anonymous_parameters,
    trivial_numeric_casts,
    trivial_casts,
    single_use_lifetimes,
    nonstandard_style,
    unreachable_pub
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::doc_markdown,
    clippy::struct_excessive_bools,
    clippy::similar_names
)]

mod detection;
mod encoding;
mod errors;
/// MicroDVD (.sub) format subtitle implementations
pub mod microdvd;
mod parsing;
/// Implementations for plain subtitles
pub mod plain;
/// SubRip (.srt) format subtitle implementations
pub mod subrip;
/// SubStation (.ass / .ssa) format subtitle implementations
pub mod substation;
mod timed_subtitle;
/// Types used for subtitle timing
pub mod timing;
mod traits;
/// WebVTT (.vtt) format subtitle implementations
pub mod webvtt;

pub use detection::{
    detect_format, detect_format_by_content, detect_format_by_content_with_encoding,
    detect_format_by_extension, detect_format_from_str, detect_format_with_encoding,
};
pub use errors::Error;
#[doc(inline)]
pub use microdvd::{MicroDvdSubtitle, TimedMicroDvdSubtitle};
#[doc(inline)]
pub use plain::PlainSubtitle;
#[doc(inline)]
pub use subrip::SubRipSubtitle;
#[doc(inline)]
pub use substation::{ass::AssSubtitle, ssa::SsaSubtitle};
pub use timed_subtitle::{Format, TimedSubtitleFile};
pub use timing::{Moment, TimeDelta};
pub use traits::{
    Subtitle, TextEvent, TextEventInterface, TextSubtitle, TimedEvent, TimedEventInterface,
    TimedSubtitle,
};
#[doc(inline)]
pub use webvtt::WebVttSubtitle;
