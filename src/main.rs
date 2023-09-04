use std::{
    collections::HashMap,
    fs::{create_dir, write, File},
    process::exit,
    sync::Arc,
};

use backend::{
    add_torrent, auth_interactive, content_torrent, delete_torrents, list_torrents, pause_torrent,
    resume_torrent,
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

mod backend;
mod cli;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub cookies: HashMap<Url, String>,
    pub default: Option<Url>,
}

const CONFIG_FILE: &'static str = "config.json";

fn main() {
    let args = BaseCommand::parse();
    let dirs = get_dirs();
    let mut config = Config::from_file(&dirs);

    match args.commands {
        cli::Commands::Auth(args) => match args.commands {
            cli::AuthCommands::SetDefault { url } => {
                config.activate_url(&url);
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
        },
        cli::Commands::Torrent(args) => {
            if config.default.is_none() || config.cookies.len() <= 0 {
                eprintln!("No (default) url configured. Please configure a url using the auth subcommand!");
                exit(1);
            }

            let info = config.get_request_info();

            match args.commands {
                cli::TorrentCommands::List {
                    sort,
                    reverse,
                    limit,
                } => {
                    list_torrents(
                        &info,
                        sort.unwrap_or(cli::TorrentSortingOptions::Name),
                        reverse,
                        limit,
                    );
                }
                cli::TorrentCommands::Add { url_or_path, pause } => {
                    add_torrent(&info, url_or_path, pause)
                }
                cli::TorrentCommands::Delete {
                    hashes,
                    delete_files,
                } => {
                    if hashes.len() < 1 {
                        eprintln!("At least one hash must be provided!");
                        exit(1);
                    }

                    delete_torrents(&info, hashes, delete_files)
                }
                cli::TorrentCommands::Pause { hash } => pause_torrent(&info, hash),
                cli::TorrentCommands::Resume { hash } => resume_torrent(&info, hash),
                cli::TorrentCommands::Content { hash } => content_torrent(&info, hash),
            }
        }
        cli::Commands::ConfigDir => {
            println!("Config dir at: {}", dirs.config_dir().display());
            exit(0);
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

        if let Some(_) = self.cookies.remove(&url) {
            println!("Removed {}", &url)
        } else {
            println!("{} is not stored.", &url)
        }
    }

    fn activate_url(&mut self, url: &Url) {
        self.default = Some(url.clone());
    }

    fn list_cookies(&self, show_secrets: bool) {
        if self.cookies.len() <= 0 {
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
            print!("\n")
        }
    }

    fn save_config(&self, dirs: &ProjectDirs) {
        let json = serde_json::to_string_pretty(&self).unwrap();
        let _ = write(dirs.config_dir().join(CONFIG_FILE), json).unwrap();
    }

    fn from_file(dirs: &ProjectDirs) -> Self {
        let dir = dirs.config_dir();
        if !dir.exists() {
            let _ = create_dir(&dir).unwrap();
        }

        let file = match File::open(&dir.join(CONFIG_FILE)) {
            Ok(f) => f,
            Err(_) => return Config::default(),
        };
        let config: Config = serde_json::from_reader(file).unwrap();

        return config;
    }
}

fn get_dirs() -> ProjectDirs {
    return ProjectDirs::from("", "", "qbtrs").unwrap();
}
