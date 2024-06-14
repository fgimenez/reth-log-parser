use eyre::Result;
use regex::Regex;
use std::time::{Duration, SystemTime};

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

pub(crate) fn extract_timestamp(line: &str) -> Result<SystemTime> {
    let timestamp_str = Regex::new(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z)")?
        .captures(line)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str())
        .ok_or_else(|| eyre::eyre!("Failed to extract timestamp"))?;

    let dt = timestamp_str.parse::<chrono::DateTime<chrono::Utc>>()?;
    Ok(SystemTime::from(dt))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Timelike};

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(&Duration::new(59, 0)), "59s");
        assert_eq!(format_duration(&Duration::new(61, 0)), "1m 1s");
        assert_eq!(format_duration(&Duration::new(3601, 0)), "1h 0m");
        assert_eq!(format_duration(&Duration::new(3661, 0)), "1h 1m");
    }

    #[test]
    fn test_extract_timestamp() {
        let line = "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";
        let timestamp = extract_timestamp(line).unwrap();
        let expected_time = chrono::Utc
            .with_ymd_and_hms(2024, 6, 7, 9, 5, 20)
            .unwrap()
            .with_nanosecond(873_354_000)
            .unwrap();

        assert_eq!(SystemTime::from(expected_time), timestamp);

        let line = "Invalid log line without a timestamp";
        let result = extract_timestamp(line);
        assert!(result.is_err());
    }
}
