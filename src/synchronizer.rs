use std::{
    fs::File,
    path::{Path, PathBuf},
    time::Duration,
};

use symphonia::core::{
    codecs::{DecoderOptions, CODEC_TYPE_NULL},
    formats::FormatOptions,
    io::MediaSourceStream,
    meta::MetadataOptions,
    probe::Hint,
};

use crate::{timestamp::Timestamp, tui};

pub trait TrackSync {
    fn find_sync_offset(&self, path: impl AsRef<Path>) -> anyhow::Result<Timestamp>;
}

#[derive(Debug)]
pub struct FileTrackSyncer;

impl FileTrackSyncer {
    pub fn new() -> Self {
        Self
    }
}

impl TrackSync for FileTrackSyncer {
    fn find_sync_offset(&self, path: impl AsRef<Path>) -> anyhow::Result<Timestamp> {
        let path = path.as_ref();
        let src = File::open(&path)?;
        let media_source = MediaSourceStream::new(Box::new(src), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = path.extension() {
            hint.with_extension(ext.to_str().unwrap());
        }

        let meta_opts: MetadataOptions = Default::default();
        let fmt_opts: FormatOptions = Default::default();

        let probed =
            symphonia::default::get_probe().format(&hint, media_source, &fmt_opts, &meta_opts)?;

        loop {
            use symphonia::core::errors::Error;

            let mut format = probed.format;
            let track = format
                .tracks()
                .iter()
                .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
                .ok_or(anyhow::anyhow!("no supported audio format found"))?;

            let dec_opts: DecoderOptions = Default::default();
            let mut decoder =
                symphonia::default::get_codecs().make(&track.codec_params, &dec_opts)?;
            let track_id = track.id;

            loop {
                let packet = match format.next_packet() {
                    Ok(packet) => packet,
                    Err(Error::ResetRequired) => {
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                };

                // Consume any new metadata that has been read since the last packet.
                while !format.metadata().is_latest() {
                    // Pop the old head of the metadata queue.
                    format.metadata().pop();

                    // Consume the new metadata at the head of the metadata queue.
                }

                if packet.track_id() != track_id {
                    continue;
                }

                match decoder.decode(&packet) {
                    Ok(decoded) => {}
                    Err(Error::IoError(_)) => {
                        // The packet failed to decode due to an IO error, skip the packet.
                        continue;
                    }
                    Err(Error::DecodeError(_)) => {
                        // The packet failed to decode due to invalid data, skip the packet.
                        continue;
                    }
                    Err(err) => return Err(err.into()),
                }
            }

            break;
        }

        todo!("find sync offset");
    }
}

pub struct AskUserSyncer;

impl AskUserSyncer {
    pub fn new() -> Self {
        Self
    }
}

impl TrackSync for AskUserSyncer {
    fn find_sync_offset(&self, path: impl AsRef<Path>) -> anyhow::Result<Timestamp> {
        let path = path.as_ref();
        eprintln!(
            "Enter sync timestamp for {:?}: [ format: HH:MM:SS.mmm eg. 01:02:03.123 ]",
            path
        );
        loop {
            let buf = tui::prompt();
            let Ok(sync_offset) = Timestamp::parse(buf) else {
                continue;
            };

            return Ok(sync_offset);
        }
    }
}
