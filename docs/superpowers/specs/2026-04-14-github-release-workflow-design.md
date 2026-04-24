# GitHub Release Workflow Design

## Overview

Create a GitHub Actions workflow that automatically builds and publishes GitHub Releases when a version tag is pushed.

## Trigger

- Tag push matching `v*` pattern (e.g., `v0.1.0`, `v1.0.0-beta.1`)

## Target Platforms

| Platform | Runner | Target Triple | Output File |
|----------|--------|---------------|-------------|
| macOS aarch64 | `macos-latest` | `aarch64-apple-darwin` | `librich_prompt_plugin.dylib` |
| macOS x86_64 | `macos-13` | `x86_64-apple-darwin` | `librich_prompt_plugin.dylib` |
| Linux x86_64 | `ubuntu-latest` | `x86_64-unknown-linux-gnu` | `librich_prompt_plugin.so` |

## Workflow Structure

### Job 1: `test`
- Runs on `ubuntu-latest`
- Steps: checkout, install Rust toolchain, `cargo test`

### Job 2: `build` (depends on `test`)
- Matrix strategy across 3 platform configurations
- Steps: checkout, install Rust toolchain, `cargo build --release`
- Upload built library as artifact with platform-specific name:
  - `librich_prompt_plugin-<target-triple>.dylib` (macOS)
  - `librich_prompt_plugin-<target-triple>.so` (Linux)

### Job 3: `release` (depends on `build`)
- Runs on `ubuntu-latest`
- Downloads all build artifacts
- Creates GitHub Release using tag name as release title
- Attaches all platform binaries to the release
- Auto-generates release notes from commits

## Artifact Naming

Files uploaded to the release:
- `librich_prompt_plugin-aarch64-apple-darwin.dylib`
- `librich_prompt_plugin-x86_64-apple-darwin.dylib`
- `librich_prompt_plugin-x86_64-unknown-linux-gnu.so`

## Permissions

- `contents: write` — required to create releases and upload assets

## Dependencies

- `git2` crate requires `libgit2` system library — available by default on GitHub Actions runners
- `yosh-plugin-sdk` fetched from GitHub via `git` dependency — no special auth needed (public repo)
