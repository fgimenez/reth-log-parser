use crate::pipeline::Pipeline;
use eyre::Result;
use regex::Regex;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

pub struct LogProcessor {
    regexes: HashMap<String, Regex>,
    pub pipelines: Arc<Mutex<Vec<Pipeline>>>,
    pub current_pipeline: Arc<Mutex<Option<Pipeline>>>,
}

impl LogProcessor {
    pub fn new() -> Result<Self> {
        let regexes = vec![
            ("start".to_string(), Regex::new(r"Preparing stage pipeline_stages=\d+/\d+ stage=(\w+) checkpoint=\d+ target=None")?),
            ("end".to_string(), Regex::new(r"Finished stage pipeline_stages=\d+/\d+ stage=(\w+) checkpoint=\d+ target=None stage_progress=100.00%")?),
            ("state_root".to_string(), Regex::new(r"Validated state root.*elapsed=(\d+\.\d+)(ms|s)")?),
        ].into_iter().collect();

        Ok(LogProcessor {
            regexes,
            pipelines: Arc::new(Mutex::new(Vec::new())),
            current_pipeline: Arc::new(Mutex::new(Some(Pipeline::new()))),
        })
    }

    pub fn process_line(&self, line: &str) -> Result<()> {
        if let Some(start_caps) = self.regexes["start"].captures(line) {
            let stage_name = start_caps.get(1).unwrap().as_str();
            let timestamp = self.extract_timestamp(line)?;

            let mut pipelines = self.pipelines.lock().unwrap();
            let mut current_pipeline = self.current_pipeline.lock().unwrap();

            if current_pipeline.is_none() || self.is_first_stage(stage_name) {
                if let Some(pipeline) = current_pipeline.take() {
                    pipelines.push(pipeline);
                }
                *current_pipeline = Some(Pipeline::new());
            }

            if let Some(ref mut pipeline) = *current_pipeline {
                pipeline.record_stage_start(stage_name, timestamp);
            }
        }

        if let Some(end_caps) = self.regexes["end"].captures(line) {
            let stage_name = end_caps.get(1).unwrap().as_str();
            let timestamp = self.extract_timestamp(line)?;
            let mut current_pipeline = self.current_pipeline.lock().unwrap();
            if let Some(ref mut pipeline) = *current_pipeline {
                pipeline.record_stage_end(stage_name, timestamp)?;
            }
        }

        if let Some(caps) = self.regexes["state_root"].captures(line) {
            if let (Some(matched_value), Some(unit)) = (caps.get(1), caps.get(2)) {
                let mut elapsed: f64 = matched_value.as_str().parse()?;
                if unit.as_str() == "ms" {
                    elapsed /= 1000.0;
                }
                let mut current_pipeline = self.current_pipeline.lock().unwrap();
                if let Some(ref mut pipeline) = *current_pipeline {
                    pipeline.update_stats("state_root", elapsed);
                }
            }
        }

        Ok(())
    }

    fn is_first_stage(&self, stage_name: &str) -> bool {
        stage_name == "Headers"
    }

    fn extract_timestamp(&self, line: &str) -> Result<SystemTime> {
        let timestamp_str = Regex::new(r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{6}Z)")?
            .captures(line)
            .and_then(|caps| caps.get(1))
            .map(|m| m.as_str())
            .ok_or_else(|| eyre::eyre!("Failed to extract timestamp"))?;

        let dt = timestamp_str.parse::<chrono::DateTime<chrono::Utc>>()?;
        Ok(SystemTime::from(dt))
    }

    pub fn print_summary(&self) {
        let pipelines = self.pipelines.lock().unwrap();
        let mut total_duration = Duration::new(0, 0);

        for (index, pipeline) in pipelines.iter().enumerate() {
            pipeline.print_summary(index);
            total_duration += pipeline.durations.values().sum::<Duration>();
        }

        println!("Total Aggregate Duration: {:.2?}", total_duration);
    }
}
