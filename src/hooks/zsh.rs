use crate::errors::EnvyError;
use std::env::current_exe;

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
    pub fn hook() -> Result<String, EnvyError> {
        Ok(ZSH_HOOK.replace("{{.SelfPath}}", &current_exe()?.to_string_lossy()))
    }
}
