use std::{
    collections::HashMap,
    fs::{create_dir, write, File},
    io::{self, Write},
    process::exit,
};

use clap::Parser;
use cli::BaseCommand;
use directories::ProjectDirs;
use reqwest::blocking::ClientBuilder;
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use url::Url;

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
    let mut config = load_config(&dirs);

    match args.commands {
        cli::Commands::Auth(args) => match args.commands {
            cli::AuthCommands::Activate { url } => {
                config.activate_url(&url);
            }
            cli::AuthCommands::List { show_secrets } => {
                config.list_cookies(show_secrets);
            }
            cli::AuthCommands::Add { url, username } => {
                if let Some((url, cookie)) = auth_with_ask(url, username) {
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
        cli::Commands::Torrent(args) => {}
    }
    save_config(&dirs, &config);
}

impl Config {
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
}

fn save_config(dirs: &ProjectDirs, config: &Config) {
    let json = serde_json::to_string_pretty(&config).unwrap();
    let _ = write(dirs.config_dir().join(CONFIG_FILE), json).unwrap();
}

fn load_config(dirs: &ProjectDirs) -> Config {
    let dir = dirs.config_dir();
    if !dir.exists() {
        let _ = create_dir(&dir).unwrap();
    }

    let file = match File::open(&dir.join(CONFIG_FILE)) {
        Ok(f) => f,
        Err(_) => return Config::default(),
    };
    let cookies: Config = serde_json::from_reader(file).unwrap();

    return cookies;
}

fn get_dirs() -> ProjectDirs {
    return ProjectDirs::from("", "", "qbtrs").unwrap();
}

fn auth_with_ask(url: Url, username: String) -> Option<(Url, String)> {
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

    if let Some(cookie) = cookies.get(0) {
        return Some((url, cookie.value().to_string()));
    } else {
        return None;
    }
}
