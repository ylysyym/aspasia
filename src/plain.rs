use crate::{
    traits::TimedSubtitle, Moment, TextEvent, TextSubtitle, TimedEvent, TimedEventInterface,
    TimedSubtitleFile,
};

/// Basic subtitle data containing only the textual content and start/end timing with no style or formatting information
#[derive(Clone, Debug)]
pub struct PlainSubtitle {
    events: Vec<PlainEvent>,
}

/// A basic event, containing only the most essential information: text to display without any formatting, and the start and end time.
#[derive(Clone, Debug)]
pub struct PlainEvent {
    /// Textual content for the event. Should not contain any formatting tags.
    pub text: String,
    /// Start time of the event
    pub start: Moment,
    /// End time of the event
    pub end: Moment,
}

impl Default for PlainSubtitle {
    fn default() -> Self {
        Self::new()
    }
}

impl PlainSubtitle {
    /// Create empty instance of `PlainSubtitle`
    #[must_use]
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    /// Create new instance using a list of already defined [`PlainEvent`]s
    #[must_use]
    pub fn from_events(events: Vec<PlainEvent>) -> Self {
        Self { events }
    }

    /// Get list of events as a slice
    #[must_use]
    pub fn events(&self) -> &[PlainEvent] {
        self.events.as_slice()
    }

    /// Get list of events as a mutable slice
    pub fn events_mut(&mut self) -> &mut [PlainEvent] {
        self.events.as_mut_slice()
    }

    /// Retrieve event at index
    #[must_use]
    pub fn event(&self, index: usize) -> Option<&PlainEvent> {
        self.events.get(index)
    }

    /// Get mutable event at index
    pub fn event_mut(&mut self, index: usize) -> Option<&mut PlainEvent> {
        self.events.get_mut(index)
    }
}

impl<T: TimedSubtitle + TextSubtitle> From<&T> for PlainSubtitle
where
    T::Event: TimedEvent + TextEvent,
{
    /// Convert from any implementor of `TimedSubtitle` and `TextSubtitle`.
    /// This strips all formatting tags from the text.
    fn from(value: &T) -> Self {
        Self {
            events: value
                .events()
                .iter()
                .map(|e| PlainEvent {
                    text: e.as_plaintext().into_owned(),
                    start: e.start(),
                    end: e.end(),
                })
                .collect(),
        }
    }
}

impl From<TimedSubtitleFile> for PlainSubtitle {
    fn from(value: TimedSubtitleFile) -> Self {
        match value {
            TimedSubtitleFile::Ass(data) => (&data).into(),
            TimedSubtitleFile::MicroDvd(data) => (&data).into(),
            TimedSubtitleFile::Ssa(data) => (&data).into(),
            TimedSubtitleFile::SubRip(data) => (&data).into(),
            TimedSubtitleFile::WebVtt(data) => (&data).into(),
        }
    }
}
