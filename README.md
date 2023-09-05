# qbtrs

A qbittorrent cli client focused on ease of use and simplicity

## Getting started

First, either compile the program yourself or aquire a binary release.

### Adding a url

After that, start by running the `auth add` command to authenticate your qbittorrent instance. You will be prompted to enter your password. Alternatively, pass `--password <YOUR PASSWORD>` to specify it directly.

```
qbtrs auth add http://example.com:8080 username
```

> NOTE: If this fails, you won't be getting any helpful error messages because the qbittorrent API does not return anything useful

### Listing all torrents

Now that you have authenticated to your instance, you can run `torrent list` to get a list of all torrents in your instance.

You can use flags like `--sort` to sort the output by things like `added-on` or `size`

```
qbtrs torrent list
```

### Adding a torrent

Adding a torrent is also very easy. Just run `torrent add` and provide either a path to a torrent file or a magnet link.

```
qbtrs torrent add path/to/torrent.torrent
```

> NOTE: When passing a magnet link, pass it in quotes to avoid the shell messing with it.
