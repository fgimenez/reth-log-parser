use std::time::Duration;

pub(crate) struct Formatter {}

impl Formatter {
    pub(crate) fn format_duration(duration: &Duration) -> String {
        let secs = duration.as_secs();
        if secs >= 3600 {
            let hours = secs / 3600;
            let minutes = (secs % 3600) / 60;
            format!("{}h {}m", hours, minutes)
        } else if secs >= 60 {
            let minutes = secs / 60;
            let seconds = secs % 60;
            format!("{}m {}s", minutes, seconds)
        } else {
            format!("{}s", secs)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(Formatter::format_duration(&Duration::new(59, 0)), "59s");
        assert_eq!(Formatter::format_duration(&Duration::new(61, 0)), "1m 1s");
        assert_eq!(Formatter::format_duration(&Duration::new(3601, 0)), "1h 0m");
        assert_eq!(Formatter::format_duration(&Duration::new(3661, 0)), "1h 1m");
    }
}
