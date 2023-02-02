use anyhow::Result;
use std::env::current_exe;

static FISH_HOOK: &str = r#"
function __direnv_export_eval --on-event fish_prompt;
	eval ("{{.SelfPath}}" export fish);
end
"#;

pub struct Fish;

impl Fish {
    pub fn hook() -> Result<String> {
        Ok(FISH_HOOK.replace("{{.SelfPath}}", &current_exe()?.to_string_lossy()))
    }
}
