use clap::{Args, Parser, Subcommand, ValueEnum};
use url::Url;

#[derive(Debug, Clone, Parser)]
#[command(version, about)]
pub struct BaseCommand {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Print the location if the config dir and exit
    ConfigDir,
    Auth(Auth),
    Torrent(Torrent),
    Global(Global),
}

/// Control authentication for different urls
#[derive(Debug, Clone, Args)]
pub struct Auth {
    #[command(subcommand)]
    pub commands: AuthCommands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum AuthCommands {
    /// set a url as the default
    SetDefault { url: Url },

    /// List all active urls
    List {
        /// Print the actual cookies
        #[arg(long)]
        show_secrets: bool,
    },
    /// Authenticate a url with a username and password. (password input is interactive by default) The Resulting cookie is stored on disk, beware!
    Add {
        /// URL of the qbittorrent api
        url: Url,

        /// The username to use
        username: String,

        /// To avoid passwords in the shell history, the password is asked for interactively by default. You can bypass this by specifying a password here
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Removes the given url
    Remove { url: Url },
    /// Log out of the provided url
    Logout { url: Url },
}

/// Controls global settings etc. for the qbittorrent app
#[derive(Debug, Clone, Args)]
pub struct Global {
    #[command(subcommand)]
    pub commands: GlobalCommands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum GlobalCommands {
    /// Shuts down the app
    Shutdown,

    /// Displays the version the app is running
    Version,

    /// Displays the logs
    Log,

    /// Displays or toggles alternative speed limits
    AltSpeed {
        #[arg(short, long)]
        toggle: bool,
    },
}

/// Control torrents with actions such as add, pause, etc.
#[derive(Debug, Clone, Args)]
pub struct Torrent {
    #[command(subcommand)]
    pub commands: TorrentCommands,
}

#[derive(Subcommand, Clone, Debug)]
pub enum TorrentCommands {
    /// List all torrents
    List {
        /// Sort the torrents by this value
        #[arg(short, long)]
        sort: Option<TorrentSortingOptions>,

        /// Reverse the display order
        #[arg(short, long)]
        reverse: bool,

        /// Limit the number of displayed torrents
        #[arg(short, long)]
        limit: Option<u32>,

        /// Refresh the screen every X milliseconds
        #[arg(short, long)]
        interval: Option<u64>,
    },
    /// Show the contents of a specific torrent
    Content {
        /// The hash of the torrent
        hash: String,
    },
    /// Add a new torrent from a file or URL
    Add {
        /// A url (magnet) or a path to a torrent file
        url_or_path: String,

        /// pause the torrent upon creation (don't download immediately)
        #[arg(short, long)]
        pause: bool,
    },
    /// Delete one or multiple torrents (and optionally their files on disk)
    Delete {
        /// The hashs of the torrents to be deleted
        hashes: Vec<String>,

        /// DANGER! This will also delete the downloaded files from the filesystem
        #[arg(short, long)]
        delete_files: bool,
    },
    /// Pasue a torrent
    Pause {
        /// The hash of the torrent
        hash: String,
    },
    /// Resume a torrent
    Resume {
        /// The hash of the torrent
        hash: String,
    },
    /// Forces the recheck of a torrent
    Recheck { hash: String },
    /// Forces the reannounce of a torrent
    Reannounce { hash: String },
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
#[allow(non_camel_case_types)]
pub enum TorrentSortingOptions {
    Name,
    Hash,
    Progress,
    Size,
    Ratio,
    State,
    Added_On,
}
