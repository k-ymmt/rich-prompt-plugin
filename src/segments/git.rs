use yosh_plugin_sdk::style::{Color, Style};
use yosh_plugin_sdk::{exec, read_file};

pub fn render(cwd: &str) -> Option<String> {
    let head_content = find_head(cwd)?;
    let branch = parse_branch(&head_content);
    let (staged, unstaged, untracked) = status_counts().unwrap_or((0, 0, 0));

    let mut result = Style::new()
        .fg(Color::Magenta)
        .bold()
        .paint(&format!("\u{e0a0} {branch}"));

    let parts = build_status_parts(staged, unstaged, untracked);
    if !parts.is_empty() {
        result.push_str(&format!(" [{}]", parts.join(" ")));
    }
    Some(result)
}

fn find_head(cwd: &str) -> Option<String> {
    let mut dir = cwd.trim_end_matches('/').to_string();
    loop {
        let path = if dir.is_empty() {
            "/.git/HEAD".to_string()
        } else {
            format!("{dir}/.git/HEAD")
        };
        if let Ok(bytes) = read_file(&path)
            && let Ok(s) = String::from_utf8(bytes)
        {
            return Some(s);
        }
        match dir.rfind('/') {
            None => return None,
            Some(0) if dir.is_empty() => return None,
            Some(0) => dir.clear(),
            Some(i) => dir.truncate(i),
        }
    }
}

fn parse_branch(head: &str) -> String {
    let trimmed = head.trim();
    if let Some(refpath) = trimmed.strip_prefix("ref: ") {
        return refpath.rsplit('/').next().unwrap_or(refpath).to_string();
    }
    trimmed.chars().take(7).collect::<String>()
}

fn status_counts() -> Option<(usize, usize, usize)> {
    let out = exec(
        "git",
        &["status", "--porcelain=v1", "--untracked-files=all"],
    )
    .ok()?;
    if out.exit_code != 0 {
        return None;
    }
    Some(parse_status_porcelain(&String::from_utf8_lossy(&out.stdout)))
}

fn parse_status_porcelain(stdout: &str) -> (usize, usize, usize) {
    let mut staged = 0usize;
    let mut unstaged = 0usize;
    let mut untracked = 0usize;
    for line in stdout.lines() {
        let mut chars = line.chars();
        let x = chars.next().unwrap_or(' ');
        let y = chars.next().unwrap_or(' ');
        if x == '?' && y == '?' {
            untracked += 1;
            continue;
        }
        if matches!(x, 'M' | 'A' | 'D' | 'R' | 'C') {
            staged += 1;
        }
        if matches!(y, 'M' | 'D') {
            unstaged += 1;
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
        assert_eq!(parse_status_porcelain(""), (0, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_only_untracked() {
        let input = "?? new1.txt\n?? new2.txt\n";
        assert_eq!(parse_status_porcelain(input), (0, 0, 2));
    }

    #[test]
    fn parse_status_porcelain_staged_only() {
        let input = "M  staged.txt\nA  added.txt\n";
        assert_eq!(parse_status_porcelain(input), (2, 0, 0));
    }

    #[test]
    fn parse_status_porcelain_unstaged_only() {
        let input = " M modified.txt\n D deleted.txt\n";
        assert_eq!(parse_status_porcelain(input), (0, 2, 0));
    }

    #[test]
    fn parse_status_porcelain_mixed() {
        let input = "M  staged.txt\n M unstaged.txt\nMM both.txt\n?? untracked.txt\nA  added.txt\n";
        // staged: M_, MM, A_ → 3
        // unstaged: _M, MM → 2
        // untracked: ?? → 1
        assert_eq!(parse_status_porcelain(input), (3, 2, 1));
    }

    #[test]
    fn parse_status_porcelain_no_trailing_newline() {
        assert_eq!(parse_status_porcelain("?? a.txt"), (0, 0, 1));
    }

    #[test]
    fn build_status_parts_all_zero_returns_empty() {
        assert!(build_status_parts(0, 0, 0).is_empty());
    }

    #[test]
    fn build_status_parts_all_nonzero_returns_three() {
        let parts = build_status_parts(2, 1, 3);
        assert_eq!(parts.len(), 3);
    }

    #[test]
    fn build_status_parts_only_staged() {
        let parts = build_status_parts(1, 0, 0);
        assert_eq!(parts.len(), 1);
        assert!(parts[0].contains("+1"));
    }
}
