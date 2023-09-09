use std::process::exit;

use crate::cli::BaseCommand;
use crate::config::RequestInfo;
use crate::{backend::*, cli, Config};
use directories::ProjectDirs;

pub fn handle_cli(args: BaseCommand, dirs: &ProjectDirs, config: &mut Config) {
    match args.commands {
        /*
        AUTH SUBCOMMAND
         */
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

        /*
        TORRENT SUBCOMMAND
         */
        cli::Commands::Torrent(args) => {
            let info = get_info_if_default(&config);

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
                    if torrent_content(&info, hash).is_none() {
                        eprintln!("Request failed, make sure the hash is valid");
                    }
                }
                cli::TorrentCommands::Recheck { hash } => recheck(&info, hash),
                cli::TorrentCommands::Reannounce { hash } => reannounce(&info, hash),
            }
        }

        /*
        CONFIG_DIR SUBCOMMAND
         */
        cli::Commands::ConfigDir => {
            println!("Config dir at: {}", dirs.config_dir().display());
            exit(0);
        }

        /*
        GLOBAL SUBCOMMAND
         */
        cli::Commands::Global(args) => {
            let info = get_info_if_default(&config);

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
}

fn get_info_if_default(config: &Config) -> RequestInfo {
    if config.default.is_none() || config.cookies.is_empty() {
        eprintln!("No (default) url configured. Please configure a url using the auth subcommand!");
        exit(1);
    }

    config.get_request_info()
}
