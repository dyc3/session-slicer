//! Stuff for managing recording sessions outputted by teleprompt-studio.

use std::path::Path;
use std::time::Duration;

use serde::{Deserialize, Deserializer};

use crate::timestamp::Timestamp;

pub const AUDIO_WAV: &str = "audio.wav";
pub const TAKES_CSV: &str = "takes.csv";
pub const META_JSON: &str = "metadata.json";

#[derive(Debug)]
pub struct Session {
    meta: SessionMeta,
    takes: SessionTakes,
}

impl Session {
    pub fn new(meta: SessionMeta, takes: SessionTakes) -> Self {
        Self { meta, takes }
    }

    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let meta = SessionMeta::from_path(path.join(META_JSON).as_path())?;
        let takes = SessionTakes::from_path(path.join(TAKES_CSV).as_path())?;

        Ok(Self::new(meta, takes))
    }

    pub fn takes(&self) -> &[SessionTake] {
        self.takes.takes()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct SessionMeta {
    sync_offset: Timestamp,
}

impl SessionMeta {
    fn from_path(path: &Path) -> anyhow::Result<Self> {
        let file = std::fs::File::open(path)?;
        let meta = serde_json::from_reader(file)?;
        Ok(meta)
    }
}

#[derive(Debug)]
pub struct SessionTakes {
    takes: Vec<SessionTake>,
}
impl SessionTakes {
    fn from_path(path: &Path) -> anyhow::Result<Self> {
        let mut reader = csv::Reader::from_path(path)?;
        let mut takes = Vec::new();

        for result in reader.deserialize() {
            let take: SessionTake = result?;
            takes.push(take);
        }

        Ok(Self { takes })
    }

    pub fn takes(&self) -> &[SessionTake] {
        &self.takes
    }
}

/// From the session csv.
///
/// ```csv
/// header,chunk_index,chunk_text,take_index,take_mark,take_start,take_end
/// ```
#[derive(Debug, Deserialize)]
pub struct SessionTake {
    header: String,
    chunk_index: usize,
    chunk_text: String,
    take_index: usize,
    take_mark: String,
    take_start: Timestamp,
    take_end: Timestamp,
}

impl SessionTake {
    pub fn start(&self) -> Timestamp {
        self.take_start
    }

    pub fn end(&self) -> Timestamp {
        self.take_end
    }

    pub fn duration(&self) -> Duration {
        self.take_end - self.take_start
    }

    pub fn mark(&self) -> &str {
        &self.take_mark
    }

    pub fn out_slice_file_name(&self) -> String {
        format!(
            "chunk-{}-take-{}-{}.mp4",
            self.chunk_index, self.take_index, self.take_mark
        )
    }
}
