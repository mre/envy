# envy

...sets environment variables when you enter a directory.

[![Documentation](https://docs.rs/envy-cli/badge.svg)](https://docs.rs/envy-cli/)
![Rust](https://github.com/mre/envy/workflows/Rust/badge.svg)

## Example

At work, I have to set some environment variables every time I'm working on certain projects.  
For example, these can be Google Cloud settings, the Consul host, or Docker configs.

It's tedious to set the correct environment variables myself every time.

[direnv] automatically loads `.env` files, but I don't want to clutter my system  
with `.env` files. Also, I need the same environment variables in a few unrelated  
projects, and I don't want to keep the `.env` files in sync.

Thus, the idea for `envy` was born.

It uses a config file that defines what environment variables to set for each folder.

## Installation

```
cargo install envy-cli
```

Add the following line to the end of your `~/.zshrc` file:

```
eval "$(envy hook zsh)"
```

Once you open a new shell, `envy` will start matching directories and set the specified
environment variables from the config file.

## Usage

Run `envy edit` to open the config file.
(On macOS, this file is located at `/Users/<user>/Library/Application Support/Envy/Config.toml`.)

Define the list of regular expressions and the settings.
The first regular expression that matches a path wins.

```toml
[[paths]]
pattern = ".*project1.*"
env = [
  "CONSUL_HTTP_ADDR=http://consul:8500",
  "GITHUB_TOKEN=123"
]

[[paths]]
pattern = ".*project2.*"
env = [
  "DOCKER_HOST=tcp://127.0.0.1:2376",
  "foo=bar"
]
```

The moment you save the file, the current terminal will automatically pick up the new settings;
no need to reload or open a new terminal. :v:

## direnv compatibility

`envy` supports loading environment files à la `direnv` as well. Run `envy allow .env` to auto-load the `.env` file in the current path on enter. You can add
multiple `.env` files (e.g. `envy allow .envrc`). Duplicate keys will be
overwritten in the order of appearance in the envy config file (run `envy edit`
to modify order).
Use `envy deny .env` to remove an environment file from the list.

## Command-line options

```
envy 0.4.0
context-based environment variables

USAGE:
    envy <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    allow     Grants envy to load the given `.env` file
    deny      Revokes the authorization of a given `.env` file
    edit      Edit the envy config file
    export    Export environment variables based on the current directory
    find      Find a single environment variable and print its value
    help      Prints this message or the help of the given subcommand(s)
    hook      Print the hook to activate envy for your shell
    load      Load environment variables from a given `.env` file (for the current session only)
    path      Print path to envy config file
    show      Show envy config for current directory
```

Note: To load the environment variables in the current shell, you need to run `eval "$(envy load)"`.

## Limitations

- Only zsh supported for now.
- Only tested on macOS. It should also work on Linux and Windows, though.
- Does not unset variables when you leave a directory.
- Developing this for myself. Thus, this project won't be worked on very actively.

[direnv]: https://direnv.net/
