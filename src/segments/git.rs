use git2::{Repository, StatusOptions};
use kish_plugin_sdk::style::{Color, Style};

pub fn render(cwd: &str) -> Option<String> {
    let repo = Repository::discover(cwd).ok()?;

    let branch_name = get_branch_name(&repo);
    let (staged, unstaged, untracked) = get_status_counts(&repo);

    let status_parts = build_status_parts(staged, unstaged, untracked);

    let mut result = Style::new()
        .fg(Color::Magenta)
        .bold()
        .paint(&format!("\u{e0a0} {branch_name}"));

    if !status_parts.is_empty() {
        result.push_str(&format!(" [{}]", status_parts.join(" ")));
    }

    Some(result)
}

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
    fn clean_repo_shows_no_status_indicators() {
        let dir = tempfile::tempdir().unwrap();
        let repo = init_repo(dir.path());
        create_initial_commit(&repo, dir.path());

        let result = render(dir.path().to_str().unwrap()).unwrap();
        // Status indicators should not appear in a clean repo
        assert!(!result.contains("+"));
        assert!(!result.contains("!"));
        assert!(!result.contains("?"));
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
