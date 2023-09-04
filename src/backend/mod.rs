use std::{
    collections::HashMap,
    io::{self, Write},
    path::PathBuf,
};

use humansize::{format_size, DECIMAL};
use reqwest::blocking::{multipart::Form, ClientBuilder};
use rpassword::read_password;
use serde::Deserialize;
use url::Url;

mod util;

use crate::{backend::util::progress_render, cli::TorrentSortingOptions, RequestInfo};

use self::util::{confirm, TorrentState};

pub fn auth_interactive(
    url: Url,
    username: String,
    password: Option<String>,
) -> Option<(Url, String)> {
    let client = ClientBuilder::new().cookie_store(true).build().unwrap();

    print!("Please provide a password for user {}: ", username);
    io::stdout().flush().unwrap();
    let password = password.unwrap_or(read_password().unwrap());

    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("username", &username);
    map.insert("password", &password);

    let login_res = client
        .post(url.join("api/v2/auth/login").unwrap().to_string())
        .header("Referer", &url.to_string())
        .form(&map)
        .send()
        .unwrap();

    let cookies: Vec<_> = login_res.cookies().collect();
    if cookies.len() < 1 {
        return None;
    }

    let mut cookie_string = "".to_string();
    for c in cookies {
        cookie_string.push_str(format!("{}={};", c.name(), c.value()).as_str());
    }

    return Some((url, cookie_string));
}

#[derive(Debug, Deserialize)]
struct TorrentInfoResponse {
    hash: String,
    name: String,
    progress: f64,
    ratio: f64,
    size: u64,
    state: TorrentState,
}

pub fn list_torrents(
    info: &RequestInfo,
    sort_by: TorrentSortingOptions,
    reverse: bool,
    limit: Option<u32>,
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

    let info_res = info
        .client
        .get(info.url.join("api/v2/torrents/info").unwrap())
        .query(&query)
        .send()
        .unwrap();

    let torrents: Vec<TorrentInfoResponse> = info_res.json().unwrap();

    for t in &torrents {
        println!("   | {}\n   |", t.name);
        println!("   |  > Hash: {}", t.hash);
        println!(
            "   |  > Progress: {:.2}% {}",
            t.progress * 100.0,
            progress_render(t.progress)
        );
        println!("   |  > Size: {}", format_size(t.size, DECIMAL));
        println!("   |  > Ratio: {:.2}", t.ratio);
        println!("   |  > State: {} ({:#})", t.state, t.state);

        println!("\n")
    }

    println!(
        "Found {} torrents, sorted by: {} {}",
        torrents.len(),
        sort_string,
        match reverse {
            true => "(reversed)",
            false => "",
        }
    )
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

pub fn content_torrent(info: &RequestInfo, hash: String) {
    let mut query: HashMap<&str, String> = HashMap::new();
    query.insert("hash", hash);

    let content_res = info
        .client
        .get(info.url.join("api/v2/torrents/files").unwrap())
        .query(&query)
        .send()
        .unwrap();

    let json: Vec<TorrentFileResponse> = content_res.json().expect("Content invalid JSON");

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

        if file_res.text().unwrap() == "Ok." {
            println!("Added url.")
        } else {
            eprintln!("Adding url failed.")
        }

        return;
    }

    if let Ok(path) = PathBuf::try_from(&url_or_path) {
        let form = form.file("torrents", path).unwrap();
        let form = form.text("paused", pause.to_string());

        let file_res = info
            .client
            .post(info.url.join("api/v2/torrents/add").unwrap())
            .multipart(form)
            .send()
            .unwrap();

        if file_res.text().unwrap() == "Ok." {
            println!("Added torrent file.")
        } else {
            eprintln!("Adding torrent file failed.")
        }

        return;
    }

    eprintln!("Provided data was not a url or a path");
}

pub fn delete_torrents(info: &RequestInfo, hashes: Vec<String>, delete_files: bool) {
    let mut formdata: HashMap<&str, String> = HashMap::new();

    formdata.insert("deleteFiles", delete_files.to_string());
    formdata.insert("hashes", hashes.join("|"));

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

    // The response is empty no matter the result, so we might as well ignore it
    let _delete_res = info
        .client
        .post(info.url.join("api/v2/torrents/delete").unwrap())
        .form(&formdata)
        .send()
        .unwrap();

    println!("Sent request to delete {} torrent(s).", hashes.len());
}

pub fn pause_torrent(info: &RequestInfo, hash: String) {
    let mut formdata: HashMap<&str, String> = HashMap::new();

    formdata.insert("hashes", hash);

    // Again, the response is completely empty.....
    let _pause_res = info
        .client
        .post(info.url.join("api/v2/torrents/pause").unwrap())
        .form(&formdata)
        .send()
        .unwrap();

    println!("Sent request to pause torrent.");
}

pub fn resume_torrent(info: &RequestInfo, hash: String) {
    let mut formdata: HashMap<&str, String> = HashMap::new();

    formdata.insert("hashes", hash);

    // Again, the response is completely empty.....
    let _pause_res = info
        .client
        .post(info.url.join("api/v2/torrents/resume").unwrap())
        .form(&formdata)
        .send()
        .unwrap();

    println!("Sent request to resume torrent.");
}
