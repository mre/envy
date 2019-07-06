# envy

A magical little tool that takes care of your environment settings.  

## What?

It sets environment variables when you enter a directory that matches a regular
expression.

## Example

At work, I have to set some environment variables whenever I'm working on
certain projects. For example, these can be Google Cloud settings, my Consul or
Docker config.

It's tedious to do that myself every time. 

[direnv] automatically loads `.env` files, but I didn't want to clutter my system
with `.env` files and I need the same environment variables in a few unrelated
projects and I don't want to have to keep the the `.env` files in sync. 

Thus, envy uses a config file that defines what environment variables to set for each folder.

Run `envy edit` to open the config file.
(On macOS, this file is located at `/Users/<user>/Library/Application Support/Envy/Config.toml`)

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

The first regular expression that matches a path will be used.

## Installation

```
cargo install envy-cli
```

Add the following line at the end of the ~/.zshrc file:

```
eval "$(envy hook zsh)"
```

Once you open a new shell, envy will watch directories and set the specified
environment variables from the config file.

## Limitations

* Only supports zsh for now.
* Only tested on macOS. Should also work on Linux and Windows.
* Does not unset variables when you leave a directory.
* Developing this for myself. Thus, this project won't be very actively
  developed.

[direnv]: https://direnv.net/