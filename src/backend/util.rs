use std::{
    io::{self, BufRead, Write},
    process::exit,
};

use chrono::NaiveDateTime;
use reqwest::blocking::Response;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TorrentState {
    Error,
    MissingFiles,
    Uploading,
    PausedUP,
    QueuedUP,
    StalledUP,
    CheckingUP,
    ForcedUP,
    Allocating,
    Downloading,
    MetaDL,
    PausedDL,
    QueuedDL,
    StalledDL,
    CheckingDL,
    ForcedDL,
    CheckingResumeData,
    Moving,
    ForcedMetaDL,
    Unknown,
}

impl std::fmt::Display for TorrentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match f.alternate() {
            false => write!(f, "{:?}", self),
            true => {
                let description = match *self {
                    TorrentState::Error => "Some error occurred, applies to paused torrents",
                    TorrentState::MissingFiles => "Torrent data files are missing",
                    TorrentState::Uploading => {
                        "Torrent is being seeded and data is being transferred"
                    }
                    TorrentState::PausedUP => "Torrent is paused and has finished downloading",
                    TorrentState::QueuedUP => "Queuing is enabled and torrent is queued for upload",
                    TorrentState::StalledUP => {
                        "Torrent is being seeded, but no connections were made"
                    }
                    TorrentState::CheckingUP => {
                        "Torrent has finished downloading and is being checked"
                    }
                    TorrentState::ForcedUP => {
                        "Torrent is forced to uploading and ignores queue limit"
                    }
                    TorrentState::Allocating => "Torrent is allocating disk space for download",
                    TorrentState::Downloading => {
                        "Torrent is being downloaded and data is being transferred"
                    }
                    TorrentState::MetaDL => {
                        "Torrent has just started downloading and is fetching metadata"
                    }
                    TorrentState::PausedDL => "Torrent is paused and has NOT finished downloading",
                    TorrentState::QueuedDL => {
                        "Queuing is enabled and torrent is queued for download"
                    }
                    TorrentState::StalledDL => {
                        "Torrent is being downloaded, but no connections were made"
                    }
                    TorrentState::CheckingDL => {
                        "Same as checkingUP, but torrent has NOT finished downloading"
                    }
                    TorrentState::ForcedDL => {
                        "Torrent is forced to downloading to ignore queue limit"
                    }
                    TorrentState::CheckingResumeData => "Checking resume data on qBt startup",
                    TorrentState::Moving => "Torrent is moving to another location",
                    TorrentState::ForcedMetaDL => "[UNDOCUMENTED]",
                    TorrentState::Unknown => "Unknown status",
                };
                write!(f, "{}", description)
            }
        }
    }
}

pub fn progress_render(progress: f64) -> String {
    let progress = (progress * 10.0) as u32;
    let mut s = "<".to_string();

    for _ in 0..progress {
        s.push('#');
    }

    for _ in progress..10 {
        s.push('_');
    }
    s.push('>');

    return s;
}

#[allow(unused)]
pub enum DefaultChoice {
    Yes,
    No,
}

pub fn confirm(text: &str, default: DefaultChoice) -> bool {
    print!(
        "{text} [{}] ",
        match default {
            DefaultChoice::No => "y/N",
            DefaultChoice::Yes => "Y/n",
        }
    );
    io::stdout().flush().unwrap();

    let line = readline().to_ascii_lowercase();

    match line.as_str() {
        "" => match default {
            DefaultChoice::Yes => true,
            DefaultChoice::No => false,
        },
        "y" => true,
        "n" => false,
        _ => false,
    }
}

pub fn readline() -> String {
    io::stdin().lock().lines().next().unwrap().unwrap()
}

pub fn exit_if_expired(res: &Response) {
    if res.status() == 403 {
        eprintln!("Your cookie is expired. Run the auth add command again to renew it.");
        exit(1);
    }
}

pub fn epoch_to_datetime(epoch: i64) -> NaiveDateTime {
    NaiveDateTime::from_timestamp_opt(epoch, 0).unwrap()
}
