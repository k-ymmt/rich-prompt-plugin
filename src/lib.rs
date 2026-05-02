use std::time::{Duration, Instant};

use yosh_plugin_sdk::{
    Capability, HookName, Plugin, cwd, exec, export, get_var, print, set_var,
};

mod segments;

#[derive(Default)]
struct RichPromptPlugin {
    last_exit_code: i32,
    last_cmd_start: Option<Instant>,
    last_duration: Option<Duration>,
    user: String,
    host: String,
}

impl Plugin for RichPromptPlugin {
    fn commands(&self) -> &[&'static str] {
        &[]
    }

    fn required_capabilities(&self) -> &[Capability] {
        &[
            Capability::Io,
            Capability::Filesystem,
            Capability::VariablesRead,
            Capability::VariablesWrite,
            Capability::FilesRead,
            Capability::CommandsExec,
            Capability::HookPreExec,
            Capability::HookPostExec,
            Capability::HookPrePrompt,
        ]
    }

    fn implemented_hooks(&self) -> &[HookName] {
        &[HookName::PreExec, HookName::PostExec, HookName::PrePrompt]
    }

    fn on_load(&mut self) -> Result<(), String> {
        self.user = exec("whoami", &[])
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "user".to_string());
        self.host = exec("hostname", &[])
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| segments::username::truncate_hostname(s.trim()).to_string())
            .unwrap_or_else(|| "host".to_string());
        Ok(())
    }

    fn exec(&mut self, _command: &str, _args: &[String]) -> i32 {
        0
    }

    fn hook_pre_exec(&mut self, _cmd: &str) {
        self.last_cmd_start = Some(Instant::now());
    }

    fn hook_post_exec(&mut self, _cmd: &str, exit_code: i32) {
        self.last_exit_code = exit_code;
        self.last_duration = self.last_cmd_start.take().map(|s| s.elapsed());
    }

    fn hook_pre_prompt(&mut self) {
        let cwd_str = cwd().unwrap_or_default();
        let home = get_var("HOME").ok().flatten();

        let mut parts: Vec<String> = Vec::new();
        parts.push(segments::username::render(&self.user, &self.host));
        parts.push(segments::directory::render(&cwd_str, home.as_deref()));

        if let Some(g) = segments::git::render(&cwd_str) {
            parts.push(g);
        }

        if let Some(d) = self.last_duration
            && let Some(seg) = segments::duration::render(d)
        {
            parts.push(seg);
        }

        let line1 = parts.join(" ");
        let line2 = segments::character::render(self.last_exit_code);

        let _ = print(&format!("{line1}\n"));
        let _ = set_var("PS1", &format!("{line2} "));
    }
}

export!(RichPromptPlugin);
