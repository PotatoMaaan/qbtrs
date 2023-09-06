use chrono::NaiveDateTime;
use serde::Deserialize;

use crate::RequestInfo;

use super::util::exit_if_expired;

pub fn shutdown(info: &RequestInfo) {
    let res = info
        .client
        .post(info.url.join("api/v2/app/shutdown").unwrap())
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!("Sent request to shutdown the app.");
}

pub fn version(info: &RequestInfo) {
    let res = info
        .client
        .post(info.url.join("api/v2/app/version").unwrap())
        .send()
        .unwrap();
    exit_if_expired(&res);

    let text = res.text().unwrap();

    println!("The qBittorrent app is running: {}", text);
}

#[derive(Debug, Deserialize)]
struct LogResponse {
    id: u32,
    message: String,
    timestamp: i64,
    // r# to use type as an identifier
    r#type: u32,
}

pub fn logs(info: &RequestInfo) {
    let res = info
        .client
        .post(info.url.join("api/v2/log/main").unwrap())
        .send()
        .unwrap();
    exit_if_expired(&res);

    let logs: Vec<LogResponse> = res.json().unwrap();

    println!("ID\tTYPE\tTIME\t\t\tMESSAGE\n");

    for log in &logs {
        let level = match log.r#type {
            1 => "NORM",
            2 => "INFO",
            4 => "WARN",
            8 => "CRIT",
            _ => "UNKNOWN",
        };

        let time = NaiveDateTime::from_timestamp_opt(log.timestamp, 0).unwrap();

        println!("{}\t{}\t{}\t{}", log.id, level, time, log.message);
    }
}

pub fn get_alt_speed(info: &RequestInfo) -> String {
    let res = info
        .client
        .post(info.url.join("api/v2/transfer/speedLimitsMode").unwrap())
        .send()
        .unwrap();
    exit_if_expired(&res);

    match res.text().unwrap().as_str() {
        "1" => "Enabled",
        "0" => "Disabled",
        _ => "Unknown",
    }
    .to_string()
}

pub fn toggle_alt_speed(info: &RequestInfo) {
    let res = info
        .client
        .post(
            info.url
                .join("api/v2/transfer/toggleSpeedLimitsMode")
                .unwrap(),
        )
        .send()
        .unwrap();
    exit_if_expired(&res);

    println!(
        "Alternative speed limits toggled. They are now: {}",
        get_alt_speed(info)
    )
}
