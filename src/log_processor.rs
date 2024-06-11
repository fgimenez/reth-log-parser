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

    pub fn print_summary<W: std::io::Write>(&self, writer: &mut W) {
        let pipelines = self.pipelines.lock().unwrap();
        let mut total_duration = Duration::new(0, 0);

        for (index, pipeline) in pipelines.iter().enumerate() {
            pipeline.print_summary(index, writer);
            total_duration += pipeline.durations.values().sum::<Duration>();
        }

        writeln!(writer, "Total Aggregate Duration: {:.2?}", total_duration).unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_log_processor_new() {
        let processor = LogProcessor::new().unwrap();
        assert!(processor.regexes.contains_key("start"));
        assert!(processor.regexes.contains_key("end"));
        assert!(processor.regexes.contains_key("state_root"));
    }

    #[test]
    fn test_process_line_start_stage() {
        let processor = LogProcessor::new().unwrap();
        let line = "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";

        processor.process_line(line).unwrap();

        let _pipelines = processor.pipelines.lock().unwrap();
        let current_pipeline = processor.current_pipeline.lock().unwrap();

        assert!(current_pipeline.is_some());
        assert!(current_pipeline
            .as_ref()
            .unwrap()
            .stages
            .contains_key("Headers"));
    }

    #[test]
    fn test_process_line_end_stage() {
        let processor = LogProcessor::new().unwrap();
        let start_line = "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";
        let end_line = "2024-06-07T09:06:20.873354Z  INFO Finished stage pipeline_stages=1/12 stage=Headers checkpoint=20038569 target=None stage_progress=100.00%";

        processor.process_line(start_line).unwrap();
        processor.process_line(end_line).unwrap();

        let _pipelines = processor.pipelines.lock().unwrap();
        let current_pipeline = processor.current_pipeline.lock().unwrap();

        assert!(current_pipeline.is_some());
        assert!(current_pipeline
            .as_ref()
            .unwrap()
            .durations
            .contains_key("Headers"));
    }

    #[test]
    fn test_process_line_state_root() {
        let processor = LogProcessor::new().unwrap();
        let line = "Validated state root elapsed=2.5s";

        processor.process_line(line).unwrap();

        let current_pipeline = processor.current_pipeline.lock().unwrap();

        assert!(current_pipeline.is_some());
        assert!(current_pipeline
            .as_ref()
            .unwrap()
            .stats
            .contains_key("state_root"));
    }

    #[test]
    fn test_print_summary() {
        let processor = LogProcessor::new().unwrap();
        let start_line = "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";
        let end_line = "2024-06-07T09:06:20.873354Z  INFO Finished stage pipeline_stages=1/12 stage=Headers checkpoint=20038569 target=None stage_progress=100.00%";

        processor.process_line(start_line).unwrap();
        processor.process_line(end_line).unwrap();

        // Finalize the last pipeline by pushing it to pipelines
        {
            let mut pipelines = processor.pipelines.lock().unwrap();
            let mut current_pipeline = processor.current_pipeline.lock().unwrap();
            if let Some(pipeline) = current_pipeline.take() {
                pipelines.push(pipeline);
            }
        }

        let mut output = Cursor::new(Vec::new());
        processor.print_summary(&mut output);

        let output_str = String::from_utf8(output.into_inner()).unwrap();

        assert!(output_str.contains("Pipeline 1:"));
        assert!(output_str.contains("Stage Headers:"));
        assert!(output_str.contains("Total Pipeline Duration:"));
    }
}
