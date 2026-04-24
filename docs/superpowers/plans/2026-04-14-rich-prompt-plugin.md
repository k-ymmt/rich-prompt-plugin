# Rich Prompt Plugin Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a yosh shell plugin that displays a starship-like rich two-line prompt with username/host, directory, git status, and command duration segments.

**Architecture:** Modular segment-based design. Each segment is a standalone module with a `render` function that returns a styled string. The `RichPromptPlugin` struct implements the yosh `Plugin` trait, using `hook_pre_exec`/`hook_post_exec` for timing and exit code tracking, and `hook_pre_prompt` to assemble and print the prompt.

**Tech Stack:** Rust (edition 2024), yosh-plugin-sdk (git dep from k-ymmt/yosh), git2, whoami, gethostname

---

## File Structure

```
Cargo.toml                  -- crate config, cdylib, dependencies
src/
  lib.rs                    -- RichPromptPlugin struct, Plugin trait impl, export! macro
  segments/
    mod.rs                  -- re-export all segment modules
    character.rs            -- prompt character (>) with exit-code coloring
    duration.rs             -- command execution time formatting
    directory.rs            -- cwd with ~ substitution
    username.rs             -- user@host display
    git.rs                  -- git branch + staged/unstaged/untracked counts
```

---

### Task 1: Project Setup — Cargo.toml and Module Skeleton

**Files:**
- Modify: `Cargo.toml`
- Create: `src/lib.rs` (overwrite template)
- Create: `src/segments/mod.rs`
- Create: `src/segments/character.rs`
- Create: `src/segments/duration.rs`
- Create: `src/segments/directory.rs`
- Create: `src/segments/username.rs`
- Create: `src/segments/git.rs`

- [ ] **Step 1: Update Cargo.toml with dependencies and cdylib crate type**

Replace the contents of `Cargo.toml` with:

```toml
[package]
name = "rich-prompt-plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
yosh-plugin-sdk = { git = "https://github.com/k-ymmt/yosh" }
git2 = "0.20"
whoami = "1"
gethostname = "1"

[dev-dependencies]
tempfile = "3"
```

- [ ] **Step 2: Create module skeleton files**

Create `src/segments/mod.rs`:

```rust
pub mod character;
pub mod directory;
pub mod duration;
pub mod git;
pub mod username;
```

Create `src/segments/character.rs`:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render(exit_code: i32) -> String {
    todo!()
}
```

Create `src/segments/duration.rs`:

```rust
use std::time::Duration;

use yosh_plugin_sdk::style::{Color, Style};

pub fn render(duration: Duration) -> Option<String> {
    todo!()
}
```

Create `src/segments/directory.rs`:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render(cwd: &str, home: Option<&str>) -> String {
    todo!()
}
```

Create `src/segments/username.rs`:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render() -> String {
    todo!()
}
```

Create `src/segments/git.rs`:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render(cwd: &str) -> Option<String> {
    todo!()
}
```

Create `src/lib.rs`:

```rust
use std::time::{Duration, Instant};

use yosh_plugin_sdk::{Capability, Plugin, PluginApi, export};

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

        if let Some(duration) = self.last_duration {
            if let Some(duration_segment) = segments::duration::render(duration) {
                line1_parts.push(duration_segment);
            }
        }

        let line1 = line1_parts.join(" ");
        let line2 = segments::character::render(self.last_exit_code);

        api.print(&format!("{line1}\n{line2} "));
    }
}

export!(RichPromptPlugin);
```

- [ ] **Step 3: Verify the project compiles**

Run: `cargo check`
Expected: compiles with no errors (there will be warnings about `todo!()` but no compilation errors)

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml Cargo.lock src/
git commit -m "feat: scaffold project with module structure and Plugin trait impl"
```

---

### Task 2: Character Segment (TDD)

**Files:**
- Modify: `src/segments/character.rs`

- [ ] **Step 1: Write tests for character segment**

Replace `src/segments/character.rs` with:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render(exit_code: i32) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_exit_code_renders_green() {
        let result = render(0);
        let expected = Style::new().fg(Color::Green).bold().paint(">");
        assert_eq!(result, expected);
    }

    #[test]
    fn failure_exit_code_renders_red() {
        let result = render(1);
        let expected = Style::new().fg(Color::Red).bold().paint(">");
        assert_eq!(result, expected);
    }

    #[test]
    fn negative_exit_code_renders_red() {
        let result = render(-1);
        let expected = Style::new().fg(Color::Red).bold().paint(">");
        assert_eq!(result, expected);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib segments::character`
Expected: FAIL — `not yet implemented`

- [ ] **Step 3: Implement character segment**

Replace the `render` function body in `src/segments/character.rs`:

```rust
pub fn render(exit_code: i32) -> String {
    let color = if exit_code == 0 {
        Color::Green
    } else {
        Color::Red
    };

    Style::new().fg(color).bold().paint(">")
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib segments::character`
Expected: 3 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/segments/character.rs
git commit -m "feat: implement character segment with exit-code coloring"
```

---

### Task 3: Duration Segment (TDD)

**Files:**
- Modify: `src/segments/duration.rs`

- [ ] **Step 1: Write tests for duration segment**

Replace `src/segments/duration.rs` with:

```rust
use std::time::Duration;

use yosh_plugin_sdk::style::{Color, Style};

pub fn render(duration: Duration) -> Option<String> {
    todo!()
}

fn format_duration(duration: Duration) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_returns_none() {
        let result = render(Duration::from_secs(1));
        assert!(result.is_none());
    }

    #[test]
    fn exactly_two_seconds_returns_some() {
        let result = render(Duration::from_secs(2));
        let expected = Style::new().fg(Color::Yellow).paint("took 2s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn seconds_only() {
        let result = render(Duration::from_secs(45));
        let expected = Style::new().fg(Color::Yellow).paint("took 45s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn minutes_and_seconds() {
        let result = render(Duration::from_secs(83));
        let expected = Style::new().fg(Color::Yellow).paint("took 1m 23s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn hours_minutes_seconds() {
        let result = render(Duration::from_secs(3723));
        let expected = Style::new().fg(Color::Yellow).paint("took 1h 2m 3s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn exact_minute() {
        let result = render(Duration::from_secs(60));
        let expected = Style::new().fg(Color::Yellow).paint("took 1m 0s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn exact_hour() {
        let result = render(Duration::from_secs(3600));
        let expected = Style::new().fg(Color::Yellow).paint("took 1h 0m 0s");
        assert_eq!(result, Some(expected));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib segments::duration`
Expected: FAIL — `not yet implemented`

- [ ] **Step 3: Implement duration segment**

Replace the `render` and `format_duration` function bodies in `src/segments/duration.rs`:

```rust
const THRESHOLD_SECS: u64 = 2;

pub fn render(duration: Duration) -> Option<String> {
    if duration.as_secs() < THRESHOLD_SECS {
        return None;
    }

    let text = format!("took {}", format_duration(duration));
    Some(Style::new().fg(Color::Yellow).paint(&text))
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib segments::duration`
Expected: 7 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/segments/duration.rs
git commit -m "feat: implement duration segment with human-readable time formatting"
```

---

### Task 4: Directory Segment (TDD)

**Files:**
- Modify: `src/segments/directory.rs`

- [ ] **Step 1: Write tests for directory segment**

Replace `src/segments/directory.rs` with:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render(cwd: &str, home: Option<&str>) -> String {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replaces_home_with_tilde() {
        let result = render("/Users/kazuki/Projects/rust", Some("/Users/kazuki"));
        let expected = Style::new().fg(Color::Blue).bold().paint("~/Projects/rust");
        assert_eq!(result, expected);
    }

    #[test]
    fn home_directory_itself_shows_tilde() {
        let result = render("/Users/kazuki", Some("/Users/kazuki"));
        let expected = Style::new().fg(Color::Blue).bold().paint("~");
        assert_eq!(result, expected);
    }

    #[test]
    fn outside_home_shows_full_path() {
        let result = render("/tmp/foo", Some("/Users/kazuki"));
        let expected = Style::new().fg(Color::Blue).bold().paint("/tmp/foo");
        assert_eq!(result, expected);
    }

    #[test]
    fn root_directory() {
        let result = render("/", Some("/Users/kazuki"));
        let expected = Style::new().fg(Color::Blue).bold().paint("/");
        assert_eq!(result, expected);
    }

    #[test]
    fn no_home_variable_shows_full_path() {
        let result = render("/Users/kazuki/Projects", None);
        let expected = Style::new().fg(Color::Blue).bold().paint("/Users/kazuki/Projects");
        assert_eq!(result, expected);
    }

    #[test]
    fn home_prefix_not_at_boundary_is_not_replaced() {
        let result = render("/Users/kazukiyamamoto", Some("/Users/kazuki"));
        let expected = Style::new().fg(Color::Blue).bold().paint("/Users/kazukiyamamoto");
        assert_eq!(result, expected);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib segments::directory`
Expected: FAIL — `not yet implemented`

- [ ] **Step 3: Implement directory segment**

Replace the `render` function body in `src/segments/directory.rs`:

```rust
pub fn render(cwd: &str, home: Option<&str>) -> String {
    let display_path = match home {
        Some(home) if cwd == home => "~".to_string(),
        Some(home) if cwd.starts_with(home) && cwd.as_bytes().get(home.len()) == Some(&b'/') => {
            format!("~{}", &cwd[home.len()..])
        }
        _ => cwd.to_string(),
    };

    Style::new().fg(Color::Blue).bold().paint(&display_path)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib segments::directory`
Expected: 6 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/segments/directory.rs
git commit -m "feat: implement directory segment with home path substitution"
```

---

### Task 5: Username Segment (TDD)

**Files:**
- Modify: `src/segments/username.rs`

- [ ] **Step 1: Write tests for username segment**

Replace `src/segments/username.rs` with:

```rust
use yosh_plugin_sdk::style::{Color, Style};

pub fn render() -> String {
    let username = whoami::username();
    let hostname = gethostname::gethostname();
    let hostname = hostname.to_string_lossy();
    let hostname = truncate_hostname(&hostname);

    Style::new()
        .fg(Color::Cyan)
        .bold()
        .paint(&format!("{username}@{hostname}"))
}

fn truncate_hostname(hostname: &str) -> &str {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_hostname_at_first_dot() {
        assert_eq!(truncate_hostname("mac.local"), "mac");
    }

    #[test]
    fn truncate_hostname_no_dot() {
        assert_eq!(truncate_hostname("myhost"), "myhost");
    }

    #[test]
    fn truncate_hostname_multiple_dots() {
        assert_eq!(truncate_hostname("a.b.c.d"), "a");
    }

    #[test]
    fn render_returns_styled_string() {
        let result = render();
        // The result should contain an @ sign and ANSI escape codes
        assert!(result.contains("@"));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib segments::username`
Expected: FAIL — `not yet implemented`

- [ ] **Step 3: Implement truncate_hostname**

Replace the `truncate_hostname` function body in `src/segments/username.rs`:

```rust
fn truncate_hostname(hostname: &str) -> &str {
    hostname.split('.').next().unwrap_or(hostname)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib segments::username`
Expected: 4 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/segments/username.rs
git commit -m "feat: implement username segment with hostname truncation"
```

---

### Task 6: Git Segment (TDD)

**Files:**
- Modify: `src/segments/git.rs`

- [ ] **Step 1: Write tests for git segment**

Replace `src/segments/git.rs` with:

```rust
use git2::{Repository, StatusOptions};
use yosh_plugin_sdk::style::{Color, Style};

pub fn render(cwd: &str) -> Option<String> {
    let repo = Repository::discover(cwd).ok()?;

    let branch_name = get_branch_name(&repo);
    let (staged, unstaged, untracked) = get_status_counts(&repo);

    let mut result = Style::new()
        .fg(Color::Magenta)
        .bold()
        .paint(&format!("\u{e0a0} {branch_name}"));

    let status_parts = build_status_parts(staged, unstaged, untracked);
    if !status_parts.is_empty() {
        result.push_str(&format!(" [{}]", status_parts.join(" ")));
    }

    Some(result)
}

fn get_branch_name(repo: &Repository) -> String {
    todo!()
}

fn get_status_counts(repo: &Repository) -> (usize, usize, usize) {
    todo!()
}

fn build_status_parts(staged: usize, unstaged: usize, untracked: usize) -> Vec<String> {
    todo!()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use super::*;

    fn init_repo(path: &Path) -> Repository {
        let repo = Repository::init(path).unwrap();
        // Configure user for commits
        let mut config = repo.config().unwrap();
        config.set_str("user.name", "Test User").unwrap();
        config.set_str("user.email", "test@example.com").unwrap();
        repo
    }

    fn create_initial_commit(repo: &Repository, path: &Path) {
        let file_path = path.join("README.md");
        fs::write(&file_path, "# Test").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("README.md")).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = repo.signature().unwrap();
        repo.commit(Some("HEAD"), &sig, &sig, "Initial commit", &tree, &[])
            .unwrap();
    }

    #[test]
    fn not_a_repo_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let result = render(dir.path().to_str().unwrap());
        assert!(result.is_none());
    }

    #[test]
    fn branch_name_on_main() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        let result = render(dir.path().to_str().unwrap()).unwrap();
        // Should contain the branch name (default branch after init + commit)
        assert!(result.contains("main") || result.contains("master"));
    }

    #[test]
    fn detached_head_shows_short_hash() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        let head = repo.head().unwrap().target().unwrap();
        repo.set_head_detached(head).unwrap();

        let result = render(dir.path().to_str().unwrap()).unwrap();
        let short_hash = &head.to_string()[..7];
        assert!(result.contains(short_hash));
    }

    #[test]
    fn untracked_files_shown() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        fs::write(dir.path().join("new_file.txt"), "hello").unwrap();

        let result = render(dir.path().to_str().unwrap()).unwrap();
        assert!(result.contains("?1"));
    }

    #[test]
    fn staged_files_shown() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        fs::write(dir.path().join("staged.txt"), "staged content").unwrap();
        let mut index = repo.index().unwrap();
        index.add_path(Path::new("staged.txt")).unwrap();
        index.write().unwrap();

        let result = render(dir.path().to_str().unwrap()).unwrap();
        assert!(result.contains("+1"));
    }

    #[test]
    fn unstaged_modifications_shown() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        // Modify tracked file without staging
        fs::write(dir.path().join("README.md"), "modified content").unwrap();

        let result = render(dir.path().to_str().unwrap()).unwrap();
        assert!(result.contains("!1"));
    }

    #[test]
    fn clean_repo_shows_no_status_brackets() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        let result = render(dir.path().to_str().unwrap()).unwrap();
        assert!(!result.contains('['));
        assert!(!result.contains(']'));
    }

    #[test]
    fn build_status_parts_all_types() {
        let parts = build_status_parts(2, 1, 3);
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn build_status_parts_none() {
        let parts = build_status_parts(0, 0, 0);
        assert!(parts.is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib segments::git`
Expected: FAIL — `not yet implemented`

- [ ] **Step 3: Implement git segment helper functions**

Replace the `get_branch_name`, `get_status_counts`, and `build_status_parts` function bodies in `src/segments/git.rs`:

```rust
fn get_branch_name(repo: &Repository) -> String {
    if repo.head_detached().unwrap_or(false) {
        if let Ok(head) = repo.head() {
            if let Some(oid) = head.target() {
                return oid.to_string()[..7].to_string();
            }
        }
        return "HEAD".to_string();
    }

    repo.head()
        .ok()
        .and_then(|head| head.shorthand().map(String::from))
        .unwrap_or_else(|| "HEAD".to_string())
}

fn get_status_counts(repo: &Repository) -> (usize, usize, usize) {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(true);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return (0, 0, 0),
    };

    let mut staged = 0;
    let mut unstaged = 0;
    let mut untracked = 0;

    for entry in statuses.iter() {
        let status = entry.status();

        if status.intersects(
            git2::Status::INDEX_NEW
                | git2::Status::INDEX_MODIFIED
                | git2::Status::INDEX_DELETED
                | git2::Status::INDEX_RENAMED
                | git2::Status::INDEX_TYPECHANGE,
        ) {
            staged += 1;
        }

        if status.intersects(
            git2::Status::WT_MODIFIED
                | git2::Status::WT_DELETED
                | git2::Status::WT_RENAMED
                | git2::Status::WT_TYPECHANGE,
        ) {
            unstaged += 1;
        }

        if status.intersects(git2::Status::WT_NEW) {
            untracked += 1;
        }
    }

    (staged, unstaged, untracked)
}

fn build_status_parts(staged: usize, unstaged: usize, untracked: usize) -> Vec<String> {
    let mut parts = Vec::new();

    if staged > 0 {
        parts.push(Style::new().fg(Color::Green).paint(&format!("+{staged}")));
    }
    if unstaged > 0 {
        parts.push(Style::new().fg(Color::Red).paint(&format!("!{unstaged}")));
    }
    if untracked > 0 {
        parts.push(
            Style::new()
                .fg(Color::Yellow)
                .paint(&format!("?{untracked}")),
        );
    }

    parts
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib segments::git`
Expected: 9 tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/segments/git.rs
git commit -m "feat: implement git segment with branch, staged, unstaged, untracked counts"
```

---

### Task 7: Integration — Full Prompt Assembly and Final Check

**Files:**
- Verify: `src/lib.rs` (already written in Task 1 — no changes needed)

- [ ] **Step 1: Run all tests**

Run: `cargo test`
Expected: all tests PASS across all segment modules

- [ ] **Step 2: Verify the full project compiles as cdylib**

Run: `cargo build`
Expected: successful build, producing `target/debug/librich_prompt_plugin.dylib`

- [ ] **Step 3: Verify no clippy warnings**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 4: Commit any fixes if needed**

If clippy required changes:

```bash
git add -A
git commit -m "fix: address clippy warnings"
```

- [ ] **Step 5: Remove template test and leftover code**

The original `src/lib.rs` had a template `add` function and test. Verify it was fully replaced in Task 1. If any dead code remains, remove it.

Run: `cargo test`
Expected: all tests still PASS

- [ ] **Step 6: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup and verification"
```
