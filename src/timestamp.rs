use std::{
    ops::{Add, Sub},
    time::Duration,
};

use anyhow::Context;
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Timestamp(Duration);

impl Timestamp {
    pub fn parse(timestamp: impl AsRef<str>) -> anyhow::Result<Self> {
        let mut parts = timestamp.as_ref().split(':');
        let hours = parts.next().unwrap().parse::<u64>().context("hours")?;
        let minutes = parts.next().unwrap().parse::<u64>().context("minutes")?;
        let rest = parts.next().unwrap();
        let mut parts = rest.split('.');
        let seconds = parts.next().unwrap().parse::<u64>().context("seconds")?;
        let millis = parts.next().unwrap().parse::<u64>().context("millis")?;

        Ok((Duration::from_secs(hours * 3600 + minutes * 60 + seconds)
            + Duration::from_millis(millis))
        .into())
    }
}

impl Serialize for Timestamp {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        format!(
            "{:02}:{:02}:{:02}.{:03}",
            self.0.as_secs() / 3600,
            self.0.as_secs() / 60,
            self.0.as_secs() % 60,
            self.0.subsec_millis()
        )
        .serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Timestamp {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let buf = String::deserialize(deserializer)?;
        let result = Self::parse(&buf).map_err(serde::de::Error::custom::<anyhow::Error>);
        result
    }
}

impl From<Duration> for Timestamp {
    fn from(duration: Duration) -> Self {
        Self(duration)
    }
}

impl From<Timestamp> for Duration {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.0
    }
}

impl Add for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Self) -> Self::Output {
        (self.0 + rhs.0).into()
    }
}

impl Add<Duration> for Timestamp {
    type Output = Timestamp;

    fn add(self, rhs: Duration) -> Self::Output {
        (self.0 + rhs).into()
    }
}

impl Sub for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.0 - rhs.0
    }
}

impl Sub<Duration> for Timestamp {
    type Output = Duration;

    fn sub(self, rhs: Duration) -> Self::Output {
        (self.0 - rhs).into()
    }
}

#[cfg(test)]
mod tests {
    use core::time;

    use super::*;

    #[test]
    fn timestamp_zero() {
        let timestamp = "00:00:00.000";

        let duration = Timestamp::parse(timestamp).unwrap();

        assert_eq!(duration, time::Duration::from_millis(0).into());
    }

    #[test]
    fn timestamp_example() {
        let timestamp = "01:02:03.004";

        let duration = Timestamp::parse(timestamp).unwrap();

        assert_eq!(duration, time::Duration::from_millis(3723004).into());
    }
}
