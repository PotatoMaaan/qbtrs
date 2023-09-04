use clap::{Args, Parser, Subcommand, ValueEnum};
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

/// Control authentication for different urls
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
        /// Print the actual cookies
        #[arg(long)]
        show_secrets: bool,
    },
    /// Authenticate a url with a username and password. (password input is interactive) The Resulting cookie is stored on disk, beware!
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
    List {
        #[arg(short, long)]
        sort_by: Option<TorrentSortingOptions>,
    },
    Add {
        url_or_file: String,
    },
    Remove {
        id: String,
    },
    Pause {
        id: String,
    },
    Resume {
        id: String,
    },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TorrentSortingOptions {
    Name,
    Hash,
    Progress,
    Size,
    Ratio,
    State,
}
