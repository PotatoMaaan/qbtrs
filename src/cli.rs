use clap::{Args, Parser, Subcommand};
use url::Url;

#[derive(Debug, Clone, Parser)]
pub struct BaseCommand {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Auth(Auth),
    Torrent(Torrent),
}

#[derive(Debug, Clone, Args)]
pub struct Auth {
    #[command(subcommand)]
    pub commands: AuthCommands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum AuthCommands {
    /// Use this url as the default
    Activate { url: Url },
    /// List all active urls
    List {
        #[arg(long)]
        show_secrets: bool,
    },
    /// Authenticate a url with a username and password. The Resulting cookie is stored on disk, beware!
    Add { url: Url, username: String },
    /// Removes the given url
    Remove { url: Url },
}

#[derive(Debug, Clone, Args)]
pub struct Torrent {
    #[command(subcommand)]
    pub commands: TorrentCommands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum TorrentCommands {
    List,
    Add { url_or_file: String },
}
