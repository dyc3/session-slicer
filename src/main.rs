use clap::Parser;
use log::*;

use crate::{
    data::Slicer,
    synchronizer::{SyncerCache, TrackSync},
};

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
        slicer.sessions.read().unwrap().len(),
        slicer.takes.values().map(|v| v.len()).sum::<usize>()
    );

    info!("searching for corresponding video tracks");
    let video_dir = args
        .video
        .unwrap_or_else(|| args.project.clone().unwrap().join("video"));

    let syncer_cache_path = video_dir.join("syncer_cache.json");
    let (mut syncer_cache, mut should_save_syncer) = match SyncerCache::load(&syncer_cache_path) {
        Ok(cache) => (cache, false),
        Err(e) => {
            warn!("failed to load syncer cache: {}", e);
            (SyncerCache::default(), true)
        }
    };

    for session in slicer.sessions.write().unwrap().values_mut() {
        let session_id = session.session_id.clone();
        let video_path = video_dir.join(format!("video-session-{}.mp4", &session_id));
        if video_path.exists() {
            info!("found video for session {}", session_id);
            // let syncer = synchronizer::FileTrackSyncer::new();

            let file_name = video_path.file_name().unwrap().to_str().unwrap();

            let sync_offset = match syncer_cache.get(file_name) {
                Some(timestamp) => {
                    info!(
                        "using cached sync offset for {}: {:?}",
                        file_name, timestamp
                    );
                    session.tracks.push(data::Track {
                        file: video_path,
                        sync_offset: timestamp,
                    });
                    continue;
                }
                None => {
                    let syncer = synchronizer::AskUserSyncer::new();
                    let timestamp = syncer.find_sync_offset(&video_path)?;

                    syncer_cache.set(file_name, timestamp);
                    should_save_syncer = true;
                    timestamp
                }
            };

            session.tracks.push(data::Track {
                file: video_path,
                sync_offset,
            });
        } else {
            warn!("no video found for session {}", session_id);
        }
    }

    if should_save_syncer {
        syncer_cache.save(&syncer_cache_path)?;
    }

    let output_dir = args
        .output
        .unwrap_or_else(|| args.project.clone().unwrap().join("video/slicer_output/"));

    slicer.perform_slicing(output_dir)?;

    Ok(())
}
