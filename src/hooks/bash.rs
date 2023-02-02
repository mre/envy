use anyhow::Result;
use std::env::current_exe;

// Shamelessly taken from direnv
// https://github.com/direnv/direnv/blob/e54386bdcccf9c7eea5976f787c4c31ddb5157d5/shell_bash.go
static BASH_HOOK: &str = r#" 
_envy_hook() {
    local previous_exit_status=$?;
    eval "$("{{.SelfPath}}" export bash)";
    return $previous_exit_status;
  };
  if ! [[ "$PROMPT_COMMAND" =~ _envy_hook ]]; then
    PROMPT_COMMAND="_envy_hook;$PROMPT_COMMAND"
  fi
"#;

pub struct Bash;

impl Bash {
    pub fn hook() -> Result<String> {
        Ok(BASH_HOOK.replace("{{.SelfPath}}", &current_exe()?.to_string_lossy()))
    }
}
