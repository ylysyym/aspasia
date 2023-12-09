use std::{
    fmt::Display,
    ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign},
};

/// Moment in time, in milliseconds relative to the start of the media file
#[derive(PartialEq, PartialOrd, Eq, Ord, Hash, Debug, Clone, Copy)]
pub struct Moment(i64);

/// Difference between two moments in milliseconds
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct TimeDelta(i64);

/// Frame index of a video
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Hash)]
pub struct Frame(i64);

impl Moment {
    /// When expressed as a timestamp, the number of hours
    #[must_use]
    pub fn hours(&self) -> i64 {
        self.0 / 1000 / 60 / 60
    }

    /// When expressed as a timestamp, the number of minutes
    #[must_use]
    pub fn minutes(&self) -> i64 {
        (self.0 / 1000 / 60) % 60
    }

    /// When expressed as a timestamp, the number of seconds
    #[must_use]
    pub fn seconds(&self) -> i64 {
        (self.0 / 1000) % 60
    }

    /// Number of milliseconds
    #[must_use]
    pub fn ms(&self) -> i64 {
        self.0 % 1000
    }

    /// Number of centiseconds
    #[must_use]
    pub fn cs(&self) -> i64 {
        self.0 / 10 % 100
    }

    /// Convert to .vtt timestamp format (`HH:MM:SS.0ms`)
    #[must_use]
    pub fn as_vtt_timestamp(&self) -> String {
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.ms()
        )
    }

    /// Convert to .srt timestamp format (`HH:MM:SS,0ms`)
    #[must_use]
    pub fn as_srt_timestamp(&self) -> String {
        format!(
            "{:02}:{:02}:{:02},{:03}",
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.ms()
        )
    }

    /// Convert to .ass timestamp format (`H:MM:SS,cs`)
    #[must_use]
    pub fn as_substation_timestamp(&self) -> String {
        format!(
            "{:01}:{:02}:{:02}.{:02}",
            self.hours(),
            self.minutes(),
            self.seconds(),
            self.cs()
        )
    }
}

impl Sub for Moment {
    type Output = TimeDelta;

    fn sub(self, rhs: Moment) -> Self::Output {
        TimeDelta(self.0 - rhs.0)
    }
}

impl Add<TimeDelta> for Moment {
    type Output = Moment;

    fn add(self, rhs: TimeDelta) -> Self::Output {
        Moment(self.0 + rhs.0)
    }
}

impl Sub<TimeDelta> for Moment {
    type Output = Moment;

    fn sub(self, rhs: TimeDelta) -> Self::Output {
        Moment(self.0 - rhs.0)
    }
}

impl Mul<i64> for Moment {
    type Output = Moment;

    fn mul(self, rhs: i64) -> Self::Output {
        Moment(self.0 * rhs)
    }
}

impl Div<i64> for Moment {
    type Output = Moment;

    fn div(self, rhs: i64) -> Self::Output {
        Moment(self.0 / rhs)
    }
}

impl AddAssign<TimeDelta> for Moment {
    fn add_assign(&mut self, rhs: TimeDelta) {
        self.0 += rhs.0;
    }
}

impl SubAssign<TimeDelta> for Moment {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        self.0 -= rhs.0;
    }
}

impl From<i64> for Moment {
    fn from(value: i64) -> Self {
        Moment(value)
    }
}

impl From<Moment> for i64 {
    fn from(value: Moment) -> Self {
        value.0
    }
}

impl Add<Moment> for TimeDelta {
    type Output = Moment;

    fn add(self, rhs: Moment) -> Self::Output {
        Moment(self.0 + rhs.0)
    }
}

impl Add for TimeDelta {
    type Output = TimeDelta;

    fn add(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 + rhs.0)
    }
}

impl Sub for TimeDelta {
    type Output = TimeDelta;

    fn sub(self, rhs: Self) -> Self::Output {
        TimeDelta(self.0 - rhs.0)
    }
}

impl Mul<i64> for TimeDelta {
    type Output = TimeDelta;

    fn mul(self, rhs: i64) -> Self::Output {
        TimeDelta(self.0 * rhs)
    }
}

impl Mul<TimeDelta> for i64 {
    type Output = TimeDelta;

    fn mul(self, rhs: TimeDelta) -> Self::Output {
        TimeDelta(self * rhs.0)
    }
}

impl Div<i64> for TimeDelta {
    type Output = TimeDelta;

    fn div(self, rhs: i64) -> Self::Output {
        TimeDelta(self.0 / rhs)
    }
}

impl AddAssign for TimeDelta {
    fn add_assign(&mut self, rhs: TimeDelta) {
        self.0 += rhs.0;
    }
}

impl SubAssign for TimeDelta {
    fn sub_assign(&mut self, rhs: TimeDelta) {
        self.0 -= rhs.0;
    }
}

impl MulAssign<i64> for TimeDelta {
    fn mul_assign(&mut self, rhs: i64) {
        self.0 *= rhs;
    }
}

impl DivAssign<i64> for TimeDelta {
    fn div_assign(&mut self, rhs: i64) {
        self.0 /= rhs;
    }
}

impl From<i64> for TimeDelta {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<TimeDelta> for i64 {
    fn from(value: TimeDelta) -> Self {
        value.0
    }
}

impl From<i64> for Frame {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<Frame> for i64 {
    fn from(value: Frame) -> Self {
        value.0
    }
}

impl Display for Frame {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) fn moment_to_frame(moment: Moment, framerate: f32) -> Frame {
    Frame(((i64::from(moment) as f32) * framerate / 1000.0).round() as i64)
}

pub(crate) fn frame_to_moment(frame: Frame, framerate: f32) -> Moment {
    Moment(((frame.0 * 1000) as f32 / framerate).round() as i64)
}
