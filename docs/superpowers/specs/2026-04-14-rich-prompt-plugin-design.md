# Rich Prompt Plugin Design Spec

## Overview

A yosh shell plugin that displays a rich, starship-like prompt using the `hook_pre_prompt` hook. The plugin shows contextual information about the current environment in a styled two-line prompt.

## Prompt Layout

Two-line format:

```
user@host ~/Projects/rust  main [+2 !1 ?3] took 3s
>
```

- **Line 1**: Information segments separated by spaces
- **Line 2**: Prompt character `>` (color indicates last command's success/failure)

## Segments

### 1. Username & Hostname (`segments/username.rs`)

- Uses `whoami` crate for username, `gethostname` crate for hostname
- Hostname truncated at first `.` (e.g., `mac.local` -> `mac`)
- Format: `user@host` (bold, cyan)
- Signature: `pub fn render() -> String`

### 2. Directory (`segments/directory.rs`)

- Gets current directory from `api.cwd()`
- Replaces home directory prefix with `~` (home obtained from `api.get_var("HOME")`)
- Format: full path in bold blue
- Example: `/Users/kazuki/Projects/rust` -> `~/Projects/rust`
- Signature: `pub fn render(cwd: &str, home: Option<&str>) -> String`

### 3. Git Branch & Status (`segments/git.rs`)

- Uses `git2` crate with `Repository::discover` to find repo from current directory upward
- Hidden entirely when not inside a git repository
- Format: ` branch [+N !N ?N]` (purple for branch icon/name)
  - ` ` — Git branch icon (Nerd Font)
  - `+N` — staged changes count (green)
  - `!N` — unstaged changes count (red)
  - `?N` — untracked files count (yellow)
  - Detached HEAD: shows first 7 characters of commit hash
  - No changes: branch name only (no brackets)
- Signature: `pub fn render(cwd: &str) -> Option<String>` (returns `None` outside a repo)

### 4. Command Duration (`segments/duration.rs`)

- Computed from `Instant` recorded in `hook_pre_exec` and elapsed in `hook_post_exec`
- Only displayed when duration >= 2 seconds
- Format: `took Ns`, `took Nm Ns`, or `took Nh Nm Ns` (yellow)
- Signature: `pub fn render(duration: Duration) -> Option<String>` (returns `None` if < 2s)

### 5. Prompt Character (`segments/character.rs`)

- Displays `>` on line 2
- Exit code 0 -> green, non-zero -> red
- First prompt (no command executed yet) -> green
- Signature: `pub fn render(exit_code: i32) -> String`

## Architecture

### Plugin Struct

```rust
#[derive(Default)]
struct RichPromptPlugin {
    last_exit_code: i32,
    last_cmd_start: Option<Instant>,
    last_duration: Option<Duration>,
}
```

### Hook Usage

| Hook | Purpose |
|------|---------|
| `hook_pre_exec` | Record command start time (`Instant::now()`) |
| `hook_post_exec` | Record exit code and compute duration |
| `hook_pre_prompt` | Assemble and print the two-line prompt |

### Required Capabilities

- `io` — stdout output via `api.print()`
- `filesystem` — `api.cwd()` for current directory
- `variables:read` — `api.get_var("HOME")` for home directory
- `hooks:pre_exec` — command start time tracking
- `hooks:post_exec` — exit code and duration tracking
- `hooks:pre_prompt` — prompt rendering

### Commands

This plugin exposes no custom commands. `commands()` returns `&[]` and `exec()` is a no-op (returns 0). All functionality is delivered through hooks.

### Flow

1. `hook_pre_exec`: `self.last_cmd_start = Some(Instant::now())`
2. `hook_post_exec`: `self.last_exit_code = exit_code; self.last_duration = self.last_cmd_start.map(|s| s.elapsed())`
3. `hook_pre_prompt`:
   - Call each segment's `render` function
   - Concatenate line 1 segments with space separators
   - Append newline + character segment
   - Output via `api.print()`

## File Structure

```
src/
  lib.rs                 -- RichPromptPlugin, Plugin trait impl, export! macro
  segments/
    mod.rs               -- re-exports
    username.rs           -- username@hostname
    directory.rs          -- current directory with ~ substitution
    git.rs               -- git branch and status
    duration.rs           -- command execution time
    character.rs          -- prompt character (>)
```

## Dependencies

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
yosh-plugin-sdk = { git = "https://github.com/k-ymmt/yosh" }
git2 = "0.20"
whoami = "1"
gethostname = "1"
```

## Testing Strategy

- **directory.rs**: Test path substitution (under home, outside home, root, home itself)
- **git.rs**: Create temporary repos with `tempfile` + `git2`, test branch name, staged/unstaged/untracked counts, detached HEAD
- **duration.rs**: Test formatting at boundary values (2s, 59s, 60s, 3600s+)
- **character.rs**: Test exit code 0 (green) and non-zero (red)
- **Dev dependency**: `tempfile` crate for git segment tests

## No Configuration

All display settings are hardcoded. Configuration file support is out of scope for the initial version.
