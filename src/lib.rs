#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::time::{Duration, Instant};

use kish_plugin_sdk::{Capability, Plugin, PluginApi, export};

mod segments;

#[derive(Default)]
struct RichPromptPlugin {
    last_exit_code: i32,
    last_cmd_start: Option<Instant>,
    last_duration: Option<Duration>,
}

impl Plugin for RichPromptPlugin {
    fn commands(&self) -> &[&str] {
        &[]
    }

    fn required_capabilities(&self) -> &[Capability] {
        &[
            Capability::Io,
            Capability::Filesystem,
            Capability::VariablesRead,
            Capability::VariablesWrite,
            Capability::HookPreExec,
            Capability::HookPostExec,
            Capability::HookPrePrompt,
        ]
    }

    fn exec(&mut self, _api: &PluginApi, _command: &str, _args: &[&str]) -> i32 {
        0
    }

    fn hook_pre_exec(&mut self, _api: &PluginApi, _cmd: &str) {
        self.last_cmd_start = Some(Instant::now());
    }

    fn hook_post_exec(&mut self, _api: &PluginApi, _cmd: &str, exit_code: i32) {
        self.last_exit_code = exit_code;
        self.last_duration = self.last_cmd_start.take().map(|start| start.elapsed());
    }

    fn hook_pre_prompt(&mut self, api: &PluginApi) {
        let cwd = api.cwd();
        let home = api.get_var("HOME");

        let mut line1_parts: Vec<String> = Vec::new();

        line1_parts.push(segments::username::render());
        line1_parts.push(segments::directory::render(&cwd, home.as_deref()));

        if let Some(git_segment) = segments::git::render(&cwd) {
            line1_parts.push(git_segment);
        }

        if let Some(duration) = self.last_duration
            && let Some(duration_segment) = segments::duration::render(duration)
        {
            line1_parts.push(duration_segment);
        }

        let line1 = line1_parts.join(" ");
        let line2 = segments::character::render(self.last_exit_code);

        api.print(&format!("{line1}\n"));
        let _ = api.set_var("PS1", &format!("{line2} "));
    }
}

export!(RichPromptPlugin);
