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
    fn render_returns_styled_string() {
        let result = render();
        // The result should contain an @ sign and ANSI escape codes
        assert!(result.contains("@"));
    }
}
