#![allow(clippy::needless_return)]
#![allow(clippy::needless_borrow)]
use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string, write},
    process::exit,
    sync::Arc,
};

use backend::{
    add_torrent, auth_interactive, content_torrent, delete_torrents, list_torrents, logout, logs,
    pause_torrent, reannounce, recheck, resume_torrent, shutdown, toggle_alt_speed, version,
};
use clap::Parser;
use cli::BaseCommand;
use directories::ProjectDirs;
use reqwest::{
    blocking::{Client, ClientBuilder},
    cookie::Jar,
};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::backend::get_alt_speed;

mod backend;
mod cli;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub cookies: HashMap<Url, String>,
    pub default: Option<Url>,
}

const CONFIG_FILE: &str = "config.toml";
const CONFIG_COMMENT: &str =
    "#This is the configuration file for qbtrs, a cli qbittorrent client.\n#If manually modifying this file, make sure that the default value (if not null) always has a corresponding entry in the cookies list.\n\n";

fn main() {
    let args = BaseCommand::parse();
    let dirs = get_dirs();
    let mut config = Config::from_file(&dirs);

    match args.commands {
        cli::Commands::Auth(args) => match args.commands {
            cli::AuthCommands::SetDefault { url } => {
                if config.cookies.contains_key(&url) {
                    config.activate_url(&url);
                    println!("Set {} as the default", url)
                } else {
                    eprintln!(
                        "Url {} is not registered. Use add subcommand to add it.",
                        &url
                    );
                }
            }
            cli::AuthCommands::List { show_secrets } => {
                config.list_cookies(show_secrets);
            }
            cli::AuthCommands::Add {
                url,
                username,
                password,
            } => {
                if let Some((url, cookie)) = auth_interactive(url, username, password) {
                    println!("Authentication successful!");
                    config.activate_url(&url);
                    config.cookies.insert(url, cookie);
                } else {
                    eprintln!("Authentication failed!");
                    exit(1);
                }
            }
            cli::AuthCommands::Remove { url } => {
                config.remove_url(&url);
            }
            cli::AuthCommands::Logout { url } => logout(&url),
        },
        cli::Commands::Torrent(args) => {
            if config.default.is_none() || config.cookies.is_empty() {
                eprintln!("No (default) url configured. Please configure a url using the auth subcommand!");
                exit(1);
            }

            let info = config.get_request_info();

            match args.commands {
                cli::TorrentCommands::List {
                    sort,
                    reverse,
                    limit,
                    interval,
                } => {
                    list_torrents(
                        &info,
                        sort.unwrap_or(cli::TorrentSortingOptions::Name),
                        reverse,
                        limit,
                        interval,
                    );
                }
                cli::TorrentCommands::Add { url_or_path, pause } => {
                    add_torrent(&info, url_or_path, pause)
                }
                cli::TorrentCommands::Delete {
                    hashes,
                    delete_files,
                } => {
                    if hashes.is_empty() {
                        eprintln!("At least one hash must be provided!");
                        exit(1);
                    }

                    delete_torrents(&info, hashes, delete_files)
                }
                cli::TorrentCommands::Pause { hash } => pause_torrent(&info, hash),
                cli::TorrentCommands::Resume { hash } => resume_torrent(&info, hash),
                cli::TorrentCommands::Content { hash } => {
                    if content_torrent(&info, hash).is_none() {
                        eprintln!("Request failed, make sure the hash is valid");
                    }
                }
                cli::TorrentCommands::Recheck { hash } => recheck(&info, hash),
                cli::TorrentCommands::Reannounce { hash } => reannounce(&info, hash),
            }
        }
        cli::Commands::ConfigDir => {
            println!("Config dir at: {}", dirs.config_dir().display());
            exit(0);
        }
        cli::Commands::Global(args) => {
            if config.default.is_none() || config.cookies.is_empty() {
                eprintln!("No (default) url configured. Please configure a url using the auth subcommand!");
                exit(1);
            }

            let info = config.get_request_info();

            match args.commands {
                cli::GlobalCommands::Shutdown => shutdown(&info),
                cli::GlobalCommands::Version => version(&info),
                cli::GlobalCommands::Log => logs(&info),
                cli::GlobalCommands::AltSpeed { toggle } => {
                    if toggle {
                        toggle_alt_speed(&info);
                    } else {
                        println!(
                            "Alternative speed limits are currently: {}",
                            get_alt_speed(&info)
                        )
                    }
                }
            }
        }
    }

    config.save_config(&dirs);
}

pub struct RequestInfo {
    pub jar: Arc<Jar>,
    pub client: Client,
    pub url: Url,
}

impl Config {
    fn get_request_info(&self) -> RequestInfo {
        let url = &self.default.clone().unwrap();

        let cookie = self
            .cookies
            .get(url)
            .expect("Invalid default value")
            .to_owned();

        let jar = Arc::new(Jar::default());
        jar.add_cookie_str(&cookie, url);
        let client = ClientBuilder::new()
            .cookie_provider(jar.clone())
            .build()
            .unwrap();

        return RequestInfo {
            jar,
            client,
            url: url.clone(),
        };
    }

    fn remove_url(&mut self, url: &Url) {
        if self.default == Some(url.clone()) {
            self.default = None;
        }

        if self.cookies.remove(&url).is_some() {
            println!("Removed {}", &url)
        } else {
            println!("{} is not stored.", &url)
        }
    }

    fn activate_url(&mut self, url: &Url) {
        self.default = Some(url.clone());
    }

    fn list_cookies(&self, show_secrets: bool) {
        if self.cookies.is_empty() {
            println!("No stored cookies!");
            return;
        }

        if !show_secrets {
            println!("NOTE: secrets are redacted. To reveal, pass --show-secrets\n")
        }

        println!("DEFAULT\tURL\tCOOKIE");
        for (url, cookie) in &self.cookies {
            if self.default == Some(url.clone()) {
                print!("[*]\t")
            } else {
                print!("[ ]\t")
            }
            print!("{}: ", url);
            if show_secrets {
                print!("{}", cookie);
            } else {
                print!("[REDACTED]")
            }
            println!("")
        }
    }

    fn save_config(&self, dirs: &ProjectDirs) {
        let path = dirs.config_dir().join(CONFIG_FILE);

        let mut toml = toml::to_string_pretty(&self).unwrap();
        toml = CONFIG_COMMENT.to_string() + &toml;

        write(path, toml).unwrap();
    }

    fn from_file(dirs: &ProjectDirs) -> Self {
        let dir = dirs.config_dir();
        if !dir.exists() {
            create_dir_all(&dir).expect("Failed creating config dir");
        }

        let file = match read_to_string(&dir.join(CONFIG_FILE)) {
            Ok(f) => f,
            Err(_) => return Config::default(),
        };
        let config: Config = toml::from_str(&file).unwrap();

        return config;
    }
}

fn get_dirs() -> ProjectDirs {
    return ProjectDirs::from("", "", "qbtrs").unwrap();
}
