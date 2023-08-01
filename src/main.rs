use clap::Parser;
use log::*;

mod cli;
mod session;
pub mod timestamp;

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();

    stderrlog::new().verbosity(args.verbosity as usize).init()?;

    debug!("args: {:?}", args);

    let sessions_dir = args
        .sessions
        .unwrap_or_else(|| args.project.unwrap().join("sessions"));

    debug!("sessions_dir: {:?}", sessions_dir);

    let sessions = std::fs::read_dir(sessions_dir)?;
    for session in sessions {
        let session = session?;
        let session_path = session.path();
        let session = session::Session::from_path(session_path.as_path())?;

        trace!("parsed session: {:?}", session);
    }
    Ok(())
}
