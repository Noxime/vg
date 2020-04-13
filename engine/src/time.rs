

/// A timestamp
#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct Time(f32);
impl Time {
    pub const fn epoch() -> Time {
        Time(0.0)
    }

    pub const fn from_secs(secs: f32) -> Time {
        Time(secs)
    }
}