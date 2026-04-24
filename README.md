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
yosh plugin install https://github.com/k-ymmt/rich-prompt-plugin@0.1.0
yosh plugin sync
```

### Manual configuration

Add to `~/.config/yosh/plugins.toml`:

```toml
[[plugin]]
name = "rich-prompt-plugin"
source = "github:k-ymmt/rich-prompt-plugin"
version = "0.1.0"
enabled = true
```

Then run:

```sh
yosh plugin sync
```

### Build from source

```sh
git clone https://github.com/k-ymmt/rich-prompt-plugin.git
cd rich-prompt-plugin
cargo build --release
```

Install the built library:

```sh
yosh plugin install target/release/librich_prompt_plugin.dylib
```

## Required Capabilities

This plugin requires the following capabilities:

| Capability | Purpose |
|------------|---------|
| `io` | Print prompt to stdout |
| `filesystem` | Read current working directory |
| `variables:read` | Read `HOME` environment variable |
| `variables:write` | Set `PS1` for prompt display |
| `hooks:pre_exec` | Track command start time |
| `hooks:post_exec` | Track exit code and duration |
| `hooks:pre_prompt` | Render the prompt |

## Requirements

- [yosh](https://github.com/k-ymmt/yosh) shell
- A terminal with ANSI color support
- [Nerd Font](https://www.nerdfonts.com/) (for the git branch icon ``)

## License

MIT
