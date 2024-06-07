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

    pub fn print_summary(&self, index: usize) {
        println!("Pipeline {}: ", index + 1);
        for (stage, duration) in &self.durations {
            println!("  Stage {}: {:.2?}", stage, duration);
        }
        println!(
            "  Total Pipeline Duration: {:.2?}",
            self.durations.values().sum::<Duration>()
        );
    }
}
