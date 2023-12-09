use std::fmt::Display;

/// Types of events in SubStation files
#[derive(Clone, Copy, Debug)]
pub enum SubStationEventKind {
    /// Dialogue event. Used to show text on screen to represent dialogue or other textual content.
    Dialogue,
    /// Picture event. Used to display external graphics on screen during playback.
    Picture,
    /// Sound event. Used to play audio during playback.
    Sound,
    /// Movie event. Used to play the video during playback.
    Movie,
    /// Command event. Used to execute arbitrary commands during playback
    Command,
}

/// Embedded font data for SubStation files
#[derive(Debug)]
pub struct SubStationFont {
    /// Name of font
    pub fontname: String,
    /// Encoded data
    pub data: String,
}

/// Embedded graphics data for SubStation files
#[derive(Debug)]
pub struct SubStationGraphic {
    /// Name of file
    pub filename: String,
    /// Encoded data
    pub data: String,
}

impl Display for SubStationEventKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let representation = match self {
            SubStationEventKind::Dialogue => "Dialogue",
            SubStationEventKind::Picture => "Picture",
            SubStationEventKind::Sound => "Sound",
            SubStationEventKind::Movie => "Movie",
            SubStationEventKind::Command => "Command",
        };

        write!(f, "{representation}")
    }
}

impl Display for SubStationFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "fontname: {}\n{}", self.fontname, self.data)
    }
}

impl Display for SubStationGraphic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "filename: {}\n{}", self.filename, self.data)
    }
}
