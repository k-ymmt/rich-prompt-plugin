use std::time::Duration;

use kish_plugin_sdk::style::{Color, Style};

const THRESHOLD_SECS: u64 = 2;

pub fn render(duration: Duration) -> Option<String> {
    if duration.as_secs() < THRESHOLD_SECS {
        return None;
    }

    let text = format!("took {}", format_duration(duration));
    Some(Style::new().fg(Color::Yellow).paint(&text))
}

fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let hours = total_secs / 3600;
    let minutes = (total_secs % 3600) / 60;
    let seconds = total_secs % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn below_threshold_returns_none() {
        let result = render(Duration::from_secs(1));
        assert!(result.is_none());
    }

    #[test]
    fn exactly_two_seconds_returns_some() {
        let result = render(Duration::from_secs(2));
        let expected = Style::new().fg(Color::Yellow).paint("took 2s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn seconds_only() {
        let result = render(Duration::from_secs(45));
        let expected = Style::new().fg(Color::Yellow).paint("took 45s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn minutes_and_seconds() {
        let result = render(Duration::from_secs(83));
        let expected = Style::new().fg(Color::Yellow).paint("took 1m 23s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn hours_minutes_seconds() {
        let result = render(Duration::from_secs(3723));
        let expected = Style::new().fg(Color::Yellow).paint("took 1h 2m 3s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn exact_minute() {
        let result = render(Duration::from_secs(60));
        let expected = Style::new().fg(Color::Yellow).paint("took 1m 0s");
        assert_eq!(result, Some(expected));
    }

    #[test]
    fn exact_hour() {
        let result = render(Duration::from_secs(3600));
        let expected = Style::new().fg(Color::Yellow).paint("took 1h 0m 0s");
        assert_eq!(result, Some(expected));
    }
}
