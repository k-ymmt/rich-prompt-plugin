use yosh_plugin_sdk::style::{Color, Style};

pub fn render(user: &str, host: &str) -> String {
    Style::new()
        .fg(Color::Cyan)
        .bold()
        .paint(&format!("{user}@{host}"))
}

pub(crate) fn truncate_hostname(hostname: &str) -> &str {
    hostname.split('.').next().unwrap_or(hostname)
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
    fn truncate_hostname_empty() {
        assert_eq!(truncate_hostname(""), "");
    }

    #[test]
    fn render_basic() {
        let result = render("alice", "mac");
        let expected = Style::new().fg(Color::Cyan).bold().paint("alice@mac");
        assert_eq!(result, expected);
    }

    #[test]
    fn render_includes_at_sign() {
        let result = render("alice", "mac");
        assert!(result.contains("alice@mac"));
    }
}
