use kish_plugin_sdk::style::{Color, Style};

pub fn render(exit_code: i32) -> String {
    let color = if exit_code == 0 {
        Color::Green
    } else {
        Color::Red
    };

    Style::new().fg(color).bold().paint(">")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn success_exit_code_renders_green() {
        let result = render(0);
        let expected = Style::new().fg(Color::Green).bold().paint(">");
        assert_eq!(result, expected);
    }

    #[test]
    fn failure_exit_code_renders_red() {
        let result = render(1);
        let expected = Style::new().fg(Color::Red).bold().paint(">");
        assert_eq!(result, expected);
    }

    #[test]
    fn negative_exit_code_renders_red() {
        let result = render(-1);
        let expected = Style::new().fg(Color::Red).bold().paint(">");
        assert_eq!(result, expected);
    }
}
