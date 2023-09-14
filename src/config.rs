use std::{
    collections::HashMap,
    fs::{create_dir_all, read_to_string, write},
    sync::Arc,
};

use directories::ProjectDirs;
use reqwest::{
    blocking::{Client, ClientBuilder},
    cookie::Jar,
};
use serde::{Deserialize, Serialize};
use url::Url;

const CONFIG_FILE: &str = "config.toml";
const CONFIG_COMMENT: &str =
    "#This is the configuration file for qbtrs, a cli qbittorrent client.\n#If manually modifying this file, make sure that the default value (if not null) always has a corresponding entry in the cookies list.\n\n";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub cookies: HashMap<Url, String>,
    pub default: Option<Url>,
}

#[derive(Debug)]
pub struct RequestInfo<'a> {
    pub jar: Arc<Jar>,
    pub client: Client,
    pub url: &'a Url,
}

impl Config {
    pub fn get_request_info(&self) -> RequestInfo {
        if let Some(url) = &self.default {
            let cookie = self
                .cookies
                .get(url)
                .expect("Invalid default value")
                .to_owned();

            let jar = Arc::new(Jar::default());
            jar.add_cookie_str(&cookie, &url);
            let client = ClientBuilder::new()
                .cookie_provider(jar.clone())
                .build()
                .unwrap();

            return RequestInfo {
                jar,
                client,
                url: &url,
            };
        } else {
            panic!("No Default value!");
        }
    }

    pub fn remove_url(&mut self, url: &Url) {
        if self.default == Some(url.clone()) {
            self.default = None;
        }

        if self.cookies.remove(&url).is_some() {
            println!("Removed {}", &url)
        } else {
            println!("{} is not stored.", &url)
        }
    }

    pub fn activate_url(&mut self, url: &Url) {
        if self.cookies.get(&url).is_none() {
            println!("{} is not stored", &url);
            return;
        }
        self.default = Some(url.clone());
    }

    pub fn list_cookies(&self, show_secrets: bool) {
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
            println!()
        }
    }

    pub fn save_config(&self, dirs: &ProjectDirs) {
        let path = dirs.config_dir().join(CONFIG_FILE);

        let mut toml = toml::to_string_pretty(&self).unwrap();
        toml = CONFIG_COMMENT.to_string() + &toml;

        write(path, toml).unwrap();
    }

    pub fn from_file(dirs: &ProjectDirs) -> Self {
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

pub fn get_dirs() -> ProjectDirs {
    return ProjectDirs::from("", "", "qbtrs").unwrap();
}
