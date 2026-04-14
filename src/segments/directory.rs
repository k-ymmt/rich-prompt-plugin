use kish_plugin_sdk::style::{Color, Style};

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
