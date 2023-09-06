use std::{
    collections::HashMap,
    io::{self, Write},
};

use reqwest::blocking::{Client, ClientBuilder};
use rpassword::read_password;
use url::Url;

use super::util::exit_if_expired;

pub fn auth_interactive(
    url: Url,
    username: String,
    provided_password: Option<String>,
) -> Option<(Url, String)> {
    let client = ClientBuilder::new().cookie_store(true).build().unwrap();

    let password;
    if let Some(pw) = provided_password {
        password = pw;
    } else {
        print!("Please provide the password for user {}: ", username);
        io::stdout().flush().unwrap();
        password = read_password().unwrap();
    }

    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("username", &username);
    map.insert("password", &password);

    let login_res = client
        .post(url.join("api/v2/auth/login").unwrap().to_string())
        .header("Referer", &url.to_string())
        .form(&map)
        .send()
        .unwrap();
    exit_if_expired(&login_res);

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

pub fn logout(url: &Url) {
    let client = Client::new();

    let res = client
        .post(url.join("/api/v2/auth/logout").unwrap())
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!("Logged out of {}.", &url);
}
