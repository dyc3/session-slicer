use std::{collections::HashMap, path::PathBuf};

use log::*;

use crate::timestamp::Timestamp;

#[derive(Debug, Default)]
pub struct Slicer {
    pub sessions: Vec<Session>,
    /// Map of session id to takes
    pub takes: HashMap<String, Vec<Take>>,
}

impl Slicer {
    pub fn register_session(&mut self, session: impl IntoSession) {
        let takes = session.takes();
        let session = session.into_session();
        debug!("registering session: {}", session.session_id);
        self.takes.insert(session.session_id.clone(), takes);
        self.sessions.push(session);
    }

    pub fn add_track(&mut self, session_id: &str, track: Track) {
        let session = self
            .sessions
            .iter_mut()
            .find(|s| s.session_id == session_id)
            .unwrap();
        session.tracks.push(track);
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
}

pub trait IntoSession {
    fn into_session(self) -> Session;

    fn takes(&self) -> Vec<Take>;
}
