# envy

...sets environment variables when you enter a directory.
[![Documentation](https://docs.rs/envy-cli/badge.svg)](https://docs.rs/envy-cli/)
![Rust](https://github.com/mre/envy/workflows/Rust/badge.svg)

## Example

At work, I have to set some environment variables every time I'm working on certain projects.  
For example, these can be Google Cloud settings, the Consul host or Docker configs.

It's tedious to set the correct environment variables myself every time.

[direnv] automatically loads `.env` files, but I don't want to clutter my system  
with `.env` files. Also, I need the same environment variables in a few unrelated  
projects and I don't want to keep the `.env` files in sync.

Thus, the idea for envy was born.

It uses a config file that defines what environment variables to set for each folder.
The first regular expression that matches a path wins.

Run `envy edit` to open the config file.
(On macOS, this file is located at `/Users/<user>/Library/Application Support/Envy/Config.toml`.)

```toml
[[paths]]
pattern = ".*project1.*"
env = [
  "CONSUL_HTTP_ADDR=http://consul:8500"
]

[[paths]]
pattern = ".*project2.*"
env = [
  "DOCKER_HOST=tcp://127.0.0.1:2376",
  "foo=bar"
]
```

## direnv compatibility

`envy` supports loading environment files a la `direnv` as well. Run `envy allow .env` to auto-load the `.env` file in the current path on enter. You can add
multiple `.env` files (e.g. `envy allow .envrc`). Duplicate keys will be
overwritten in the order of appearance in the envy config file (run `envy edit`
to modify order).

## Installation

```
cargo install envy-cli
```

Add the following line at the end of the `~/.zshrc` file:

```
eval "$(envy hook zsh)"
```

Once you open a new shell, envy will start matching directories and set the specified
environment variables from the config file.

## Usage

```
envy 0.3.0
Matthias Endler <matthias-endler@gmx.net>
context-based environment variables

USAGE:
    envy <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    edit      Edit the envy config file
    export    Export environment variables based on the current directory
    help      Prints this message or the help of the given subcommand(s)
    hook      Print the hook to activate envy for your shell
    show      Show envy config for current directory
```

## Limitations

- Only zsh supported for now.
- Only tested on macOS. It should also work on Linux and Windows, though.
- Does not unset variables when you leave a directory.
- Developing this for myself. Thus, this project won't be worked on very actively.

[direnv]: https://direnv.net/
