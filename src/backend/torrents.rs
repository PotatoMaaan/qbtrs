use super::util::TorrentState;
use crate::{
    backend::util::{self, confirm, epoch_to_datetime, exit_if_expired, progress_render},
    cli::TorrentSortingOptions,
    config::RequestInfo,
};
use humansize::{format_size, DECIMAL};
use reqwest::blocking::multipart::Form;
use serde::Deserialize;
use std::{collections::HashMap, path::PathBuf, thread, time::Duration};
use url::Url;

#[derive(Debug, Deserialize)]
struct TorrentInfoResponse {
    hash: String,
    name: String,
    progress: f64,
    ratio: f64,
    size: u64,
    state: TorrentState,
    added_on: i64,
}

pub fn list_torrents(
    info: &RequestInfo,
    sort_by: TorrentSortingOptions,
    reverse: bool,
    limit: Option<u32>,
    interval: Option<u64>,
) {
    let sort_string = format!("{:?}", sort_by).to_ascii_lowercase();

    let mut query: HashMap<&str, String> = HashMap::new();
    query.insert("sort", sort_string.clone());

    if reverse {
        query.insert("reverse", "true".to_string());
    }

    if let Some(limit) = limit {
        query.insert("limit", limit.to_string());
    }

    let mut refresh_counter = 0;
    loop {
        let res = info
            .client
            .get(info.url.join("api/v2/torrents/info").unwrap())
            .query(&query)
            .send()
            .unwrap();

        exit_if_expired(&res);

        let torrents: Vec<TorrentInfoResponse> = res.json().unwrap();

        println!("\n");
        for t in &torrents {
            let added_on = epoch_to_datetime(t.added_on);

            println!("   | {}\n   |", t.name);
            println!("   |  > Hash: {}", t.hash);
            println!(
                "   |  > Progress: {:.2}% {}",
                t.progress * 100.0,
                progress_render(t.progress)
            );
            println!("   |  > Size: {}", format_size(t.size, DECIMAL));
            println!("   |  > Aded on: {}", added_on);
            println!("   |  > Ratio: {:.2}", t.ratio);
            println!("   |  > State: {} ({:#})", t.state, t.state);

            println!("\n")
        }

        if let Some(interval) = interval {
            println!(
                "Found {} torrents, sorted by: {} {}\nRefreshed {} times, every {}ms",
                torrents.len(),
                sort_string,
                match reverse {
                    true => "(reversed)",
                    false => "",
                },
                refresh_counter,
                interval
            );
            thread::sleep(Duration::from_millis(interval));
            refresh_counter += 1;

            // Clear screen control char
            print!("{}[2J", 27 as char);
        } else {
            println!(
                "Found {} torrents, sorted by: {} {}",
                torrents.len(),
                sort_string,
                match reverse {
                    true => "(reversed)",
                    false => "",
                }
            );
            break;
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct TorrentFileResponse {
    index: u64,
    name: String,
    piece_range: [u32; 2],
    progress: f64,
    size: u64,
}

pub fn torrent_content(info: &RequestInfo, hash: String) -> Option<()> {
    let mut query: HashMap<&str, String> = HashMap::new();
    query.insert("hash", hash);

    let content_res = info
        .client
        .get(info.url.join("api/v2/torrents/files").unwrap())
        .query(&query)
        .send()
        .unwrap();
    exit_if_expired(&content_res);

    let json: Vec<TorrentFileResponse> = match content_res.json() {
        Ok(v) => v,
        Err(_) => return None,
    };

    for file in &json {
        println!("\n\n   | {}\n   |", file.name);
        println!(
            "   |  > Progress: {:.2}% {}",
            file.progress * 100.0,
            progress_render(file.progress)
        );
        println!("   |  > Size: {}", format_size(file.size, DECIMAL));
    }

    println!("\n\nTorrent contains {} files.", json.len());

    Some(())
}

pub fn add_torrent(info: &RequestInfo, url_or_path: String, pause: bool) {
    let form = Form::new();

    if let Ok(url) = Url::parse(&url_or_path) {
        let form = form.text("urls", url.to_string());
        let form = form.text("paused", pause.to_string());

        let file_res = info
            .client
            .post(info.url.join("api/v2/torrents/add").unwrap())
            .multipart(form)
            .send()
            .unwrap();
        exit_if_expired(&file_res);

        if file_res.text().unwrap() == "Ok." {
            println!("Added url.")
        } else {
            eprintln!("Adding url failed.")
        }

        return;
    }

    let path = PathBuf::from(&url_or_path);
    let form = match form.file("torrents", &path) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed reading file '{}': {}", &path.display(), e);
            return;
        }
    };
    let form = form.text("paused", pause.to_string());

    let file_res = info
        .client
        .post(info.url.join("api/v2/torrents/add").unwrap())
        .multipart(form)
        .send()
        .unwrap();
    exit_if_expired(&file_res);

    if file_res.text().unwrap() == "Ok." {
        println!("Added torrent file.");
    } else {
        eprintln!("Adding torrent file failed.");
    }

    return;
}

pub fn delete_torrents(info: &RequestInfo, hashes: Vec<String>, delete_files: bool) {
    let mut multipart = Form::new();

    multipart = multipart.text("deleteFiles", delete_files.to_string());
    multipart = multipart.text("hashes", hashes.join("|"));

    if !confirm(
        &format!(
            "You are about to delete {} torrent(s){}. Are you sure?",
            hashes.len(),
            match delete_files {
                true => " AND THEIR FILES ON DISK",
                false => "",
            }
        ),
        util::DefaultChoice::No,
    ) {
        println!("Cancelled");
        return;
    }

    let delete_res = info
        .client
        .post(info.url.join("api/v2/torrents/delete").unwrap())
        .multipart(multipart)
        .send()
        .unwrap();
    exit_if_expired(&delete_res);

    println!("Sent request to delete {} torrent(s).", hashes.len());
}

pub fn pause_torrent(info: &RequestInfo, hash: String) {
    let mut multipart = Form::new();

    multipart = multipart.text("hashes", hash);

    let res = info
        .client
        .post(info.url.join("api/v2/torrents/pause").unwrap())
        .multipart(multipart)
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!("Sent request to pause torrent.");
}

pub fn resume_torrent(info: &RequestInfo, hash: String) {
    let mut multipart = Form::new();

    multipart = multipart.text("hashes", hash);

    let res = info
        .client
        .post(info.url.join("api/v2/torrents/resume").unwrap())
        .multipart(multipart)
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!("Sent request to resume torrent.");
}

pub fn recheck(info: &RequestInfo, hash: String) {
    let mut multipart = Form::new();

    multipart = multipart.text("hashes", hash);

    let res = info
        .client
        .post(info.url.join("api/v2/torrents/recheck").unwrap())
        .multipart(multipart)
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!("Sent request to recheck torrent.");
}

pub fn reannounce(info: &RequestInfo, hash: String) {
    let mut multipart = Form::new();

    multipart = multipart.text("hashes", hash);

    let res = info
        .client
        .post(info.url.join("api/v2/torrents/reannounce").unwrap())
        .multipart(multipart)
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!("Sent request to reannounce torrent.");
}
