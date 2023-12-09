use std::{
    borrow::Cow,
    fmt::Display,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
    str::FromStr,
};

use encoding_rs::Encoding;

use crate::{errors::Error, Moment, TimeDelta};

/// Base trait for all subtitle implementations.
pub trait Subtitle: Sized + Display + FromStr {
    /// Event type for the given subtitle format
    type Event;

    /// Load subtitle from given path.
    /// Automatically attempts to detect the encoding to use from the file contents.
    ///
    /// # Errors
    ///
    /// If an error is encountered while opening the file, returns [`Error::FileIoError`]
    fn from_path(path: impl AsRef<Path>) -> Result<Self, Error> {
        Self::from_path_with_encoding(path, None)
    }

    /// Load subtitle format from path using the given encoding
    ///
    /// # Errors
    ///
    /// If an error is encountered while opening the file, returns [`Error::FileIoError`]
    fn from_path_with_encoding(
        path: impl AsRef<Path>,
        encoding: Option<&'static Encoding>,
    ) -> Result<Self, Error>;

    /// Get list of events as a slice
    fn events(&self) -> &[Self::Event];

    /// Get list of events as a mutable slice
    fn events_mut(&mut self) -> &mut [Self::Event];

    /// Try to get event at given index
    fn event(&self, index: usize) -> Option<&Self::Event> {
        self.events().get(index)
    }
    /// Try to get mutable event at given index
    fn event_mut(&mut self, index: usize) -> Option<&mut Self::Event> {
        self.events_mut().get_mut(index)
    }

    /// Write subtitles to file at the given path
    ///
    /// # Errors
    ///
    /// Returns [`Error::FileIoError`] if method fails to create file at the specified path
    fn export(&self, path: impl AsRef<Path>) -> Result<(), Error> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        Ok(write!(writer, "{self}")?)
    }
}

/// Trait representing textual subtitle formats
pub trait TextSubtitle: Subtitle
where
    Self::Event: TextEvent,
{
    /// Remove all styling/formatting information from the text and subtitle metadata
    fn strip_formatting(&mut self) {
        for event in self.events_mut() {
            event.strip_formatting();
        }
    }
}

/// Time-based subtitle
pub trait TimedSubtitle: Subtitle
where
    Self::Event: TimedEvent,
{
    /// Shift all events in subtitle by given amount of time, in milliseconds.
    fn shift(&mut self, delta: TimeDelta)
    where
        <Self as Subtitle>::Event: TimedEvent,
    {
        for event in self.events_mut() {
            event.shift(delta);
        }
    }
}

/// Trait offering helper functions for textual subtitle events
pub trait TextEvent: TextEventInterface {
    /// Remove all formatting tags from event text
    fn strip_formatting(&mut self) {
        self.set_text(self.unformatted_text().into_owned());
    }

    /// Get text content with all formatting tags removed
    fn unformatted_text(&self) -> Cow<'_, String>;
}

/// Interface for getting/modifying textual subtitle event fields.
/// Required for implementation of [`TextEvent`].
///
/// Not recommended to use this trait outside of the implementation of [`TextEvent`],
/// as it will pollute the namespace of subtitle event type properties.
pub trait TextEventInterface {
    /// Text associated with event
    fn text(&self) -> String;

    /// Modify text associated with event
    fn set_text(&mut self, text: String);
}

/// Helper methods for time-based subtitle events.
pub trait TimedEvent: TimedEventInterface {
    /// Shift subtitle event by given amount of time in milliseconds.
    ///
    /// Positive numbers will result in the subtitle event appearing later,
    /// while negative numbers will make the event appear earlier.
    fn shift(&mut self, delta: TimeDelta) {
        self.set_start(self.start() + delta);
        self.set_end(self.end() + delta);
    }

    /// Get duration of event in milliseconds, as a `TimeDelta`
    fn duration(&self) -> TimeDelta {
        self.end() - self.start()
    }
}

/// Interface for interacting with timed events.
/// Required for implementation of [`TimedEvent`].
pub trait TimedEventInterface {
    /// Start time of event
    fn start(&self) -> Moment;

    /// End time of event
    fn end(&self) -> Moment;

    /// Modify start time of event
    fn set_start(&mut self, moment: Moment);

    /// Modify end time of event
    fn set_end(&mut self, moment: Moment);
}
