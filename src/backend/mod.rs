use std::{
    collections::HashMap,
    convert::Infallible,
    fmt::Display,
    io::{self, Write},
};

use humansize::{format_size, DECIMAL};
use reqwest::blocking::{Client, ClientBuilder};
use rpassword::read_password;
use serde::Deserialize;
use url::Url;

mod util;

use crate::{
    backend::util::{bytes_to_gib, progress_render},
    cli::TorrentSortingOptions,
    RequestInfo,
};

use self::util::TorrentState;

pub fn auth_with_ask(url: Url, username: String) -> Option<(Url, String)> {
    let client = ClientBuilder::new().cookie_store(true).build().unwrap();

    print!("Please provide a password for user {}: ", username);
    io::stdout().flush().unwrap();
    let password = read_password().unwrap();

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
