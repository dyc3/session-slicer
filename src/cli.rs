use std::{path::PathBuf, str::FromStr};

use clap::{Parser, ValueEnum};

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short, long)]
    pub ffmpeg_path: Option<PathBuf>,

    /// The `sessions` directory.
    #[arg(long)]
    pub sessions: Option<PathBuf>,

    /// The `video` directory.
    #[arg(long)]
    pub video: Option<PathBuf>,

    /// Where the sliced videos will be saved.
    #[arg(long)]
    pub output: Option<PathBuf>,

    pub project: Option<PathBuf>,

    #[arg(short, long, short, long, value_enum, default_value_t=Verbosity::Info)]
    pub(crate) verbosity: Verbosity,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Verbosity {
    Error = 0,
    Warn = 1,
    Info = 2,
    Debug = 3,
    Trace = 4,
}

impl std::fmt::Display for Verbosity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self {
                Verbosity::Error => "error",
                Verbosity::Warn => "warn",
                Verbosity::Info => "info",
                Verbosity::Debug => "debug",
                Verbosity::Trace => "trace",
            }
        ))
    }
}

impl FromStr for Verbosity {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "error" => Ok(Verbosity::Error),
            "warn" => Ok(Verbosity::Warn),
            "info" => Ok(Verbosity::Info),
            "debug" => Ok(Verbosity::Debug),
            "trace" => Ok(Verbosity::Trace),
            _ => Err(anyhow::anyhow!("Invalid verbosity level: {}", s)),
        }
    }
}
