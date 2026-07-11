use yosh_plugin_sdk::style::{Color, Style};
use yosh_plugin_sdk::{exec, read_file};

pub fn render(cwd: &str) -> Option<String> {
    let head_content = find_head(cwd)?;
    let branch = parse_branch(&head_content);
    let (staged, unstaged, untracked, conflicted) = status_counts().unwrap_or((0, 0, 0, 0));

    let mut result = Style::new()
        .fg(Color::Magenta)
        .bold()
        .paint(&format!("\u{e0a0} {branch}"));

    let parts = build_status_parts(staged, unstaged, untracked, conflicted);
    if !parts.is_empty() {
        result.push_str(&format!(" [{}]", parts.join(" ")));
    }
    Some(result)
}

fn find_head(cwd: &str) -> Option<String> {
    find_head_with(cwd, |path| {
        read_file(path)
            .ok()
            .and_then(|bytes| String::from_utf8(bytes).ok())
    })
}

fn find_head_with(cwd: &str, read: impl Fn(&str) -> Option<String>) -> Option<String> {
    let mut dir = cwd.trim_end_matches('/').to_string();
    loop {
        let git_path = if dir.is_empty() {
            "/.git".to_string()
        } else {
            format!("{dir}/.git")
        };
        if let Some(s) = read(&format!("{git_path}/HEAD")) {
            return Some(s);
        }
        // Linked worktrees and submodules have `.git` as a file containing
        // a `gitdir:` pointer to the real git directory.
        if let Some(content) = read(&git_path)
            && let Some(gitdir) = parse_gitdir(&content)
        {
            let base = if dir.is_empty() { "/" } else { dir.as_str() };
            let resolved = resolve_gitdir(base, gitdir);
            return read(&format!("{resolved}/HEAD"));
        }
        match dir.rfind('/') {
            None => return None,
            Some(0) => dir.clear(),
            Some(i) => dir.truncate(i),
        }
    }
}

fn parse_gitdir(content: &str) -> Option<&str> {
    content.trim().strip_prefix("gitdir:").map(str::trim)
}

fn resolve_gitdir(base_dir: &str, gitdir: &str) -> String {
    if gitdir.starts_with('/') {
        return gitdir.to_string();
    }
    let mut components: Vec<&str> = base_dir.split('/').filter(|c| !c.is_empty()).collect();
    for part in gitdir.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                components.pop();
            }
            other => components.push(other),
        }
    }
    format!("/{}", components.join("/"))
}

fn parse_branch(head: &str) -> String {
    let trimmed = head.trim();
    if let Some(refpath) = trimmed.strip_prefix("ref: ") {
        return refpath.rsplit('/').next().unwrap_or(refpath).to_string();
    }
    trimmed.chars().take(7).collect::<String>()
}

fn status_counts() -> Option<(usize, usize, usize, usize)> {
    let out = exec(
        "git",
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    )
    .ok()?;
    if out.exit_code != 0 {
        return None;
    }
    Some(parse_status_porcelain(&String::from_utf8_lossy(&out.stdout)))
}

fn parse_status_porcelain(stdout: &str) -> (usize, usize, usize, usize) {
    let mut staged = 0usize;
    let mut unstaged = 0usize;
    let mut untracked = 0usize;
    let mut conflicted = 0usize;
    for line in stdout.lines() {
        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        if x == '?' && y == '?' {
            untracked += 1;
            continue;
        }
        // Unmerged entries: any 'U', or both sides added/deleted.
        if x == 'U' || y == 'U' || (x == 'A' && y == 'A') || (x == 'D' && y == 'D') {
            conflicted += 1;
            continue;
        }
        if matches!(x, 'M' | 'A' | 'D' | 'R' | 'C' | 'T') {
            staged += 1;
        }
        if matches!(y, 'M' | 'D' | 'T' | 'A') {
            unstaged += 1;
        }
    }
    (staged, unstaged, untracked, conflicted)
}

fn build_status_parts(
    staged: usize,
    unstaged: usize,
    untracked: usize,
    conflicted: usize,
) -> Vec<String> {
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
    if conflicted > 0 {
        parts.push(
            Style::new()
                .fg(Color::Red)
                .bold()
                .paint(&format!("={conflicted}")),
        );
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_branch_normal_ref() {
        assert_eq!(parse_branch("ref: refs/heads/main\n"), "main");
    }

    #[test]
    fn parse_branch_nested_ref() {
        assert_eq!(parse_branch("ref: refs/heads/feature/foo\n"), "foo");
    }

    #[test]
    fn parse_branch_no_trailing_newline() {
        assert_eq!(parse_branch("ref: refs/heads/dev"), "dev");
    }

    #[test]
    fn parse_branch_detached_head() {
        let hash = "0123456789abcdef0123456789abcdef01234567";
        assert_eq!(parse_branch(hash), "0123456");
    }

    #[test]
    fn parse_branch_detached_with_trailing_newline() {
        assert_eq!(
            parse_branch("0123456789abcdef0123456789abcdef01234567\n"),
            "0123456"
        );
    }

    #[test]
    fn parse_status_porcelain_empty() {
        assert_eq!(parse_status_porcelain(""), (0, 0, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_only_untracked() {
        let input = "?? new1.txt\n?? new2.txt\n";
        assert_eq!(parse_status_porcelain(input), (0, 0, 2, 0));
    }

    #[test]
    fn parse_status_porcelain_staged_only() {
        let input = "M  staged.txt\nA  added.txt\n";
        assert_eq!(parse_status_porcelain(input), (2, 0, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_unstaged_only() {
        let input = " M modified.txt\n D deleted.txt\n";
        assert_eq!(parse_status_porcelain(input), (0, 2, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_mixed() {
        let input = "M  staged.txt\n M unstaged.txt\nMM both.txt\n?? untracked.txt\nA  added.txt\n";
        // staged: M_, MM, A_ → 3
        // unstaged: _M, MM → 2
        // untracked: ?? → 1
        assert_eq!(parse_status_porcelain(input), (3, 2, 1, 0));
    }

    #[test]
    fn parse_status_porcelain_no_trailing_newline() {
        assert_eq!(parse_status_porcelain("?? a.txt"), (0, 0, 1, 0));
    }

    #[test]
    fn parse_status_porcelain_typechange_staged() {
        assert_eq!(parse_status_porcelain("T  typechange.txt\n"), (1, 0, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_typechange_unstaged() {
        assert_eq!(parse_status_porcelain(" T typechange.txt\n"), (0, 1, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_intent_to_add_is_unstaged() {
        assert_eq!(parse_status_porcelain(" A intent.txt\n"), (0, 1, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_unmerged_conflicts() {
        let input = "UU both-modified.txt\nAA both-added.txt\nDD both-deleted.txt\n";
        assert_eq!(parse_status_porcelain(input), (0, 0, 0, 3));
    }

    #[test]
    fn parse_status_porcelain_one_sided_unmerged() {
        let input = "AU added-by-us.txt\nUD deleted-by-them.txt\n";
        assert_eq!(parse_status_porcelain(input), (0, 0, 0, 2));
    }

    #[test]
    fn parse_status_porcelain_conflict_not_double_counted() {
        // A conflicted entry must not also count as staged/unstaged.
        let input = "UU conflict.txt\nM  staged.txt\n";
        assert_eq!(parse_status_porcelain(input), (1, 0, 0, 1));
    }

    #[test]
    fn build_status_parts_all_zero_returns_empty() {
        assert!(build_status_parts(0, 0, 0, 0).is_empty());
    }

    #[test]
    fn build_status_parts_all_nonzero_returns_four() {
        let parts = build_status_parts(2, 1, 3, 1);
        assert_eq!(parts.len(), 4);
    }

    #[test]
    fn build_status_parts_only_staged() {
        let parts = build_status_parts(1, 0, 0, 0);
        assert_eq!(parts.len(), 1);
        assert!(parts[0].contains("+1"));
    }

    #[test]
    fn build_status_parts_only_conflicted() {
        let parts = build_status_parts(0, 0, 0, 2);
        assert_eq!(parts.len(), 1);
        assert!(parts[0].contains("=2"));
    }

    fn fake_fs<'a>(files: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
        move |path: &str| {
            files
                .iter()
                .find(|(p, _)| *p == path)
                .map(|(_, c)| (*c).to_string())
        }
    }

    #[test]
    fn find_head_normal_repo() {
        let fs = fake_fs(&[("/repo/.git/HEAD", "ref: refs/heads/main\n")]);
        assert_eq!(
            find_head_with("/repo", fs),
            Some("ref: refs/heads/main\n".to_string())
        );
    }

    #[test]
    fn find_head_walks_up_from_subdirectory() {
        let fs = fake_fs(&[("/repo/.git/HEAD", "ref: refs/heads/main\n")]);
        assert_eq!(
            find_head_with("/repo/src/deep", fs),
            Some("ref: refs/heads/main\n".to_string())
        );
    }

    #[test]
    fn find_head_not_a_repo() {
        let fs = fake_fs(&[]);
        assert_eq!(find_head_with("/tmp/foo", fs), None);
    }

    #[test]
    fn find_head_worktree_gitdir_file() {
        // In a linked worktree, `.git` is a file pointing at the real git dir.
        let fs = fake_fs(&[
            ("/wt/.git", "gitdir: /repo/.git/worktrees/wt\n"),
            ("/repo/.git/worktrees/wt/HEAD", "ref: refs/heads/feature\n"),
        ]);
        assert_eq!(
            find_head_with("/wt", fs),
            Some("ref: refs/heads/feature\n".to_string())
        );
    }

    #[test]
    fn find_head_submodule_relative_gitdir() {
        // Submodules use a relative gitdir pointer.
        let fs = fake_fs(&[
            ("/repo/sub/.git", "gitdir: ../.git/modules/sub\n"),
            ("/repo/.git/modules/sub/HEAD", "ref: refs/heads/main\n"),
        ]);
        assert_eq!(
            find_head_with("/repo/sub", fs),
            Some("ref: refs/heads/main\n".to_string())
        );
    }

    #[test]
    fn parse_gitdir_valid() {
        assert_eq!(
            parse_gitdir("gitdir: /repo/.git/worktrees/wt\n"),
            Some("/repo/.git/worktrees/wt")
        );
    }

    #[test]
    fn parse_gitdir_invalid() {
        assert_eq!(parse_gitdir("ref: refs/heads/main\n"), None);
    }

    #[test]
    fn resolve_gitdir_absolute_path_unchanged() {
        assert_eq!(
            resolve_gitdir("/wt", "/repo/.git/worktrees/wt"),
            "/repo/.git/worktrees/wt"
        );
    }

    #[test]
    fn resolve_gitdir_relative_path_resolved() {
        assert_eq!(
            resolve_gitdir("/repo/sub", "../.git/modules/sub"),
            "/repo/.git/modules/sub"
        );
    }

    #[test]
    fn resolve_gitdir_relative_with_current_dir_component() {
        assert_eq!(
            resolve_gitdir("/repo/sub", "./gitdir"),
            "/repo/sub/gitdir"
        );
    }
}
