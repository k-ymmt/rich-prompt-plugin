# rich-prompt-plugin

A [yosh](https://github.com/k-ymmt/yosh) shell plugin that displays a rich, [starship](https://starship.rs)-like prompt.

```
kazuki@mac ~/Projects/rust  main [+2 !1 ?3] took 3s
❯
```

## Features

| Segment | Description | Color |
|---------|-------------|-------|
| Username & Hostname | `user@host` (hostname truncated at first `.`) | Cyan, Bold |
| Directory | Current directory with `~` substitution | Blue, Bold |
| Git Branch & Status | Branch name + staged/unstaged/untracked counts | Magenta, Bold |
| Command Duration | Execution time (shown when >= 2s) | Yellow |
| Prompt Character | `>` — green on success, red on failure | Green / Red |

### Git Status Indicators

- `+N` — staged changes (green)
- `!N` — unstaged changes (red)
- `?N` — untracked files (yellow)

## Installation

### From GitHub

```sh
yosh plugin install https://github.com/k-ymmt/rich-prompt-plugin
yosh plugin sync
```

### From GitHub (pinned version)

```sh
yosh plugin install https://github.com/k-ymmt/rich-prompt-plugin@0.2.2
yosh plugin sync
```

### Manual configuration

Add to `~/.config/yosh/plugins.toml`:

```toml
[[plugin]]
name = "rich-prompt-plugin"
source = "github:k-ymmt/rich-prompt-plugin"
version = "0.2.2"
enabled = true
allowed_commands = [
    "whoami",
    "hostname",
    "git status:*",
]
```

Then run:

```sh
yosh plugin sync
```

The `allowed_commands` list is required for the `commands:exec` capability — without it, user/host caching falls back to the literals `"user"` / `"host"` and the git status counts (`+N !N ?N`) are omitted (the branch name still renders).

### Build from source

This plugin is a WebAssembly Component (`.wasm`), built with `cargo-component`:

```sh
git clone https://github.com/k-ymmt/rich-prompt-plugin.git
cd rich-prompt-plugin
rustup target add wasm32-wasip2
cargo install cargo-component --locked --version 0.18.0
cargo component build --target wasm32-wasip2 --release
```

Install the built component:

```sh
yosh plugin install target/wasm32-wasip2/release/rich_prompt_plugin.wasm
yosh plugin sync
```

## Required Capabilities

| Capability | Purpose |
|------------|---------|
| `io` | Print the first prompt line to stdout |
| `filesystem` | Read the current working directory |
| `variables:read` | Read `HOME` |
| `variables:write` | Set `PS1` |
| `files:read` | Read `<repo>/.git/HEAD` to detect repo and branch |
| `commands:exec` | Run `whoami` / `hostname` (once at load) and `git status` |
| `hooks:pre_exec` | Track command start time |
| `hooks:post_exec` | Track exit code and duration |
| `hooks:pre_prompt` | Render the prompt |

## Limitations

- **Linked worktrees not supported.** Repositories where `.git` is a file containing `gitdir: <path>` (created via `git worktree add`) are not detected in this version. Plain `.git/` directories work as expected. Support may be added in a future release.
- **Graceful degradation when commands are unavailable.** If `git` is not on `PATH` or `commands:exec` is denied / restricted by `allowed_commands`, status counts are omitted and the branch name alone is rendered. If `whoami` / `hostname` are unavailable at load time, the prompt shows the literal strings `"user"` / `"host"`.

## Requirements

- [yosh](https://github.com/k-ymmt/yosh) 0.2.x or later
- A terminal with ANSI color support
- [Nerd Font](https://www.nerdfonts.com/) (for the git branch icon ``)
- `git` on `PATH` (for status counts)

## License

MIT
