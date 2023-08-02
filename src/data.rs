use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, RwLock},
};

use log::*;

use crate::timestamp::Timestamp;

#[derive(Debug, Default)]
pub struct Slicer {
    pub sessions: Arc<RwLock<HashMap<String, Session>>>,
    /// Map of session id to takes
    pub takes: HashMap<String, Vec<Take>>,
}

impl Slicer {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn with_ffmpeg(oath: impl AsRef<Path>) -> Self {
        todo!("impl")
    }

    pub fn register_session(&mut self, session: impl IntoSession) {
        let takes = session.takes();
        let session = session.into_session();
        debug!("registering session: {}", session.session_id);
        self.takes.insert(session.session_id.clone(), takes);
        self.sessions
            .write()
            .unwrap()
            .insert(session.session_id.clone(), session);
    }

    pub fn add_track(&mut self, session_id: &str, track: Track) {
        let mut sessions = self.sessions.write().unwrap();
        let session = sessions.get_mut(session_id).expect("session not found");
        session.tracks.push(track);
    }

    fn takes_iter(&self) -> impl Iterator<Item = &Take> {
        self.takes.values().flatten()
    }

    pub fn perform_slicing(&self, output_dir: impl AsRef<Path>) -> anyhow::Result<()> {
        let output_dir = output_dir.as_ref();
        info!("slicer outputting to {:?}", output_dir);
        std::fs::create_dir_all(output_dir)?;

        let results: Vec<_> = self
            .takes_iter()
            .enumerate()
            .map(|(idx, take)| self.slice_take(idx, take, output_dir))
            .collect();

        todo!("finish impl");

        Ok(())
    }

    fn slice_take(&self, index: usize, take: &Take, output_dir: &Path) -> anyhow::Result<()> {
        let sessions = self.sessions.read().unwrap();
        for (track_idx, track) in sessions
            .get(&take.session_id)
            .unwrap()
            .tracks
            .iter()
            .enumerate()
        {
            let ext = track.file.extension().unwrap();
            let start = take.start + track.sync_offset;
            let end = take.end + track.sync_offset;

            let file_name = format!(
                "chunk-{}-take-{}-track-{}-{}.{}",
                take.chunk_id,
                index,
                track_idx,
                take.mark,
                ext.to_str().unwrap(),
            );
            debug!("slicing {} to {}", track.file.display(), file_name);

            Command::new("ffmpeg")
                .arg("-i")
                .arg(track.file.as_os_str())
                .arg("-ss")
                .arg(start.to_string())
                .arg("-to")
                .arg(end.to_string())
                .arg("-c:a")
                .arg("copy")
                .arg(output_dir.join(file_name))
                .output()?;
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct Session {
    pub session_id: String,
    pub tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
pub struct Track {
    pub file: PathBuf,
    pub sync_offset: Timestamp,
}

#[derive(Debug, Clone)]
pub struct Take {
    pub session_id: String,
    pub chunk_id: String,
    pub start: Timestamp,
    pub end: Timestamp,
    pub mark: String,
}

pub trait IntoSession {
    fn into_session(self) -> Session;

    fn takes(&self) -> Vec<Take>;
}
