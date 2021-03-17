use anyhow::Result;
use std::env::current_exe;

// Shamelessly taken from direnv
// https://github.com/direnv/direnv/blob/e54386bdcccf9c7eea5976f787c4c31ddb5157d5/shell_zsh.go
static ZSH_HOOK: &str = r#" 
_envy_hook() {
    eval "$("{{.SelfPath}}" export zsh)";
}
typeset -ag precmd_functions;
if [[ -z ${precmd_functions[(r)_envy_hook]} ]]; then
precmd_functions+=_envy_hook;
fi
"#;

pub struct Zsh;

impl Zsh {
    pub fn hook() -> Result<String> {
        Ok(ZSH_HOOK.replace("{{.SelfPath}}", &current_exe()?.to_string_lossy()))
    }
}
