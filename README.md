
## About

**musso** is a **mus**ic **so**rter CLI tool that helps you to keep your music folder sorted. It's
designed to be simple and fast, but also powerful and fully automated. Currently, 
**musso** supports MP3, FLAC, OGG, M4A and M4P. This project was forked from [muso](https://github.com/quebin31/muso).

## Building
To build **musso** yourself you need at least Rust 1.41. If you aren't going 
to install it using a package manager you should build **musso** with feature 
`standalone` activated, for example:

```bash
cargo build --features standalone --release
```

The standalone feature include contents of [service](share/musso.service) and 
[config](share/config.toml) in binary, so **musso** can create these files by itself.

## Installing
To install from source using cargo (installed bin is in `$HOME/.cargo/bin`)
you can do the following:

```bash
cargo install --path . --features standalone
```

[//]: # (Package is also [available on the AUR]&#40;https://aur.archlinux.org/packages/musso&#41; )

[//]: # (for Arch Linux, just install it using your preferred method, for example:)

[//]: # ()
[//]: # (```bash)

[//]: # (yay -S musso)

[//]: # (```)

## Concepts

### Format string
**musso** is all about renaming and moving files around, but how it'll decide
where the new file will reside, or which is going to be its name? Fortunately
you can tell **musso** how to rename your files with a *format string*. This
string will build the new name (path) using one or more of the following
placeholders:

- `{artist}`: Artist name (**Album Artist** from tags is preferred, then **Artist**).
- `{album}`: Album name.
- `{disc}`: Disc number.
- `{track}`: Track number.
- `{title}`: Song title.
- `{ext}`: File extension (e.g. `mp3`, `flac`)

As an example, the default format that **musso** will use is the following.

```rs
"{artist}/{album}/{track} - {title}.{ext}"
```

The `{disc}` and `{track}` placeholders have the option to fill
with leading zeros, the syntax is `{disc:n}` or `{track:n}` where `n` is the
length that has to be achieved adding leading zeros. For example, using `{disc:2}` will produce the following transformations:

- `2` will become `02`
- `10` will become `10`

Finally, all of these placeholders (except `{ext}`) support an optional flag 
(activated by adding a `?` before the `}`, e.g. `{artist?}`, `{disc:2?}`). 
Renaming a file that doesn't have a specific tag doesn't fail but leaves empty 
that placeholder in the string, however note that there are some rules:

- Directory components cannot be optional (e.g. this is invalid `{artist}/{album?}/{title}.{ext}`)
- File name component must have one required placeholder, apart from `{ext}` (e.g. this is invalid `{artist}/{title?}.{ext}`)

**Note:** These rules may be different in the future if I find a better way to fill these "unknowns" (possibly using `?` for digits and `Unknown` for strings, or adding the option to provide a custom value).

A format string can be specified for *oneshot* mode using the `-f/--format`
option, or providing it in for each [library](#libraries) in the [config
file](share/config.toml).

### Libraries
We recently talked about libraries, these objects are used in the [config
file](share/config.toml) to provide **musso** settings while it's running in
*watcher* mode. For example, the default library provided in the [default config file](share/config.toml) is described as follows.

```toml
[libraries.default]
# Specified format that will be used for this library
format = '{artist}/{album}/{track} - {title}.{ext}'
# Folders that compose this library
folders = ['$HOME/Music']
# If enabled, the rename will be compatible with exFAT
exfat-compat = true
```

They are used to provide different options, to different folders. 

### Config file
**musso** will search for a config file in the following directories in order:
- `$XDG_CONFIG_DIR/musso/config.toml`
- `$HOME/.config/musso/config.toml`

It's also possible to indicate a custom path for config file with the
`-c/--config` option. Config file is primary used when running in *watcher*
mode, but it's also able to provide a default *format string* for certain
folders while running in *oneshot* mode. For example, in the [default config
file](share/config.toml) the default library specifies a format and a list of
folders, if you would run **musso** on `$HOME/Music` without specifying a
format, it'll try to grab it from the config file, if there isn't one that
correspond to the folder it'll fall back to the [default](#format-string).

## Usage
**musso** can be used in two modes: *oneshot* and *watcher*. Both of them have 
similar functionalities, but as the naming suggest they perform it differently.
Below we have the output of `musso --help`, which explains each option or flag available

```
USAGE:
    musso [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c, --config <config>    Path to custom config file

SUBCOMMANDS:
    copy-service    Copy service file to systemd user config dir
    help            Prints this message or the help of the given subcommand(s)
    sort            Sort a music directory
    watch           Watch libraries and sort added files
```

### Oneshot
By the default, **musso** will run on the current working dir, but you can
provide your own path as a free argument. Config file is optional in this mode.

### Watcher
In this mode config file is required, and as it's described in section `[watch]` 
of the [default config file](share/config.toml), the watcher can be configured.

```toml
[watch]
every = 1 # second(s)
# Specifies which libraries will be seen by musso
libraries = [ 'default' ]
```

### Systemd service
It's recommended to invoke the *watcher* mode using the provided [service
file](share/musso.service) for `systemd`, this way you can run **musso**
automatically on boot. Service file should be run on user level (`systemctl
--user`). The easiest way to copy the service file is running **musso** with
`copy-service` subcommand.

## License

GNU General Public License v3.0 

See [LICENSE](LICENSE) to see the full text.
