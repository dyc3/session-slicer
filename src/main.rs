use std::{io::stdin, time::Duration};

use clap::Parser;
use log::*;
use signalo::filters::dimensioned::dimensions::Time;

use crate::{data::Slicer, synchronizer::TrackSync, timestamp::Timestamp};

mod cli;
mod data;
mod session;
mod synchronizer;
pub mod timestamp;
mod tui;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    stderrlog::new().verbosity(args.verbosity as usize).init()?;

    debug!("args: {:?}", args);

    let sessions_dir = args
        .sessions
        .unwrap_or_else(|| args.project.clone().unwrap().join("sessions"));

    debug!("sessions_dir: {:?}", sessions_dir);

    let mut slicer = Slicer::default();
    let sessions = std::fs::read_dir(sessions_dir)?;
    for session in sessions {
        let session = session?;
        let session_path = session.path();
        let session = session::TelepromptStudioSession::from_path(session_path.as_path())?;

        trace!("parsed session: {:?}", session);

        slicer.register_session(session);
    }
    info!(
        "found {} sessions, {} takes",
        slicer.sessions.len(),
        slicer.takes.values().map(|v| v.len()).sum::<usize>()
    );

    info!("searching for corresponding video tracks");
    let video_dir = args
        .video
        .unwrap_or_else(|| args.project.clone().unwrap().join("video"));

    for session in slicer.sessions.iter_mut() {
        let session_id = session.session_id.clone();
        let video_path = video_dir.join(format!("video-session-{}.mp4", &session_id));
        if video_path.exists() {
            info!("found video for session {}", session_id);
            // let syncer = synchronizer::FileTrackSyncer::new();
            let syncer = synchronizer::AskUserSyncer::new();
            let sync_offset = syncer.find_sync_offset(&video_path)?;

            session.tracks.push(data::Track {
                file: video_path,
                sync_offset,
            });
        } else {
            warn!("no video found for session {}", session_id);
        }
    }

    Ok(())
}
