use crate::stats;
use eyre::Result;
use std::{
    collections::HashMap,
    time::{Duration, SystemTime},
};

#[derive(Default)]
pub struct Pipeline {
    pub stages: HashMap<String, (SystemTime, Option<SystemTime>)>,
    pub durations: HashMap<String, Duration>,
    pub stats: HashMap<String, stats::Stats>,
}

impl Pipeline {
    pub fn new() -> Self {
        Pipeline {
            stages: HashMap::new(),
            durations: HashMap::new(),
            stats: HashMap::new(),
        }
    }

    pub fn record_stage_start(&mut self, stage_name: &str, timestamp: SystemTime) {
        self.stages
            .insert(stage_name.to_string(), (timestamp, None));
    }

    pub fn record_stage_end(&mut self, stage_name: &str, timestamp: SystemTime) -> Result<()> {
        if let Some((start_time, _)) = self.stages.get_mut(stage_name) {
            let duration = timestamp.duration_since(*start_time)?;
            self.durations.insert(stage_name.to_string(), duration);
            self.stats
                .entry(stage_name.to_string())
                .or_default()
                .update(duration.as_secs_f64());
        }
        Ok(())
    }

    pub fn update_stats(&mut self, label: &str, elapsed: f64) {
        self.stats
            .entry(label.to_string())
            .or_default()
            .update(elapsed);
    }

    pub fn print_summary<W: std::io::Write>(&self, index: usize, writer: &mut W) {
        writeln!(writer, "Pipeline {}: ", index + 1).unwrap();
        for (stage, duration) in &self.durations {
            writeln!(writer, "  Stage {}: {:.2?}", stage, duration).unwrap();
        }
        writeln!(
            writer,
            "  Total Pipeline Duration: {:.2?}",
            self.durations.values().sum::<Duration>()
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, SystemTime};

    #[test]
    fn test_record_stage_start() {
        let mut pipeline = Pipeline::new();
        let stage_name = "Headers";
        let timestamp = SystemTime::now();

        pipeline.record_stage_start(stage_name, timestamp);

        assert_eq!(pipeline.stages.len(), 1);
        assert!(pipeline.stages.contains_key(stage_name));
        assert_eq!(pipeline.stages[stage_name], (timestamp, None));
    }

    #[test]
    fn test_record_stage_end() -> Result<()> {
        let mut pipeline = Pipeline::new();
        let stage_name = "Headers";
        let start_time = SystemTime::now();
        let end_time = start_time + Duration::from_secs(60); // 1 minute later

        pipeline.record_stage_start(stage_name, start_time);
        pipeline.record_stage_end(stage_name, end_time)?;

        assert_eq!(pipeline.durations.len(), 1);
        assert!(pipeline.durations.contains_key(stage_name));
        assert_eq!(pipeline.durations[stage_name], Duration::from_secs(60));
        Ok(())
    }

    #[test]
    fn test_update_stats() {
        let mut pipeline = Pipeline::new();
        let label = "state_root";
        let elapsed = 2.5;

        pipeline.update_stats(label, elapsed);

        assert_eq!(pipeline.stats.len(), 1);
        assert!(pipeline.stats.contains_key(label));
        assert_eq!(pipeline.stats[label].mean, elapsed);
    }

    #[test]
    fn test_print_summary() {
        let mut pipeline = Pipeline::new();
        let stage_name = "Headers";
        let start_time = SystemTime::now();
        let end_time = start_time + Duration::from_secs(60); // 1 minute later

        pipeline.record_stage_start(stage_name, start_time);
        pipeline.record_stage_end(stage_name, end_time).unwrap();

        let mut output = Vec::new();
        pipeline.print_summary(0, &mut output);

        let output_str = String::from_utf8(output).unwrap();
        let expected_output =
            "Pipeline 1: \n  Stage Headers: 60.00s\n  Total Pipeline Duration: 60.00s\n"
                .to_string();
        assert_eq!(expected_output, output_str);
    }
}
