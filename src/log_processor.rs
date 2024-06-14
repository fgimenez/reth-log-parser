use crate::{
    pipeline::Pipeline,
    time::{extract_timestamp, format_duration},
};
use eyre::Result;
use regex::Regex;
use std::{collections::HashMap, time::Duration};

pub struct LogProcessor {
    regexes: HashMap<String, Regex>,
    pub pipelines: Vec<Pipeline>,
    pub current_pipeline: Option<Pipeline>,
}

impl LogProcessor {
    pub fn new() -> Result<Self> {
        let regexes = vec![
            (
                "start".to_string(),
                Regex::new(
                    r"Preparing stage pipeline_stages=\d+/\d+ stage=(\w+) checkpoint=\d+ target=",
                )?,
            ),
            (
                "end".to_string(),
                Regex::new(
                    r"Finished stage pipeline_stages=\d+/\d+ stage=(\w+) checkpoint=\d+ target=",
                )?,
            ),
            (
                "state_root".to_string(),
                Regex::new(r"Validated state root.*elapsed=(\d+\.\d+)(ms|s)")?,
            ),
        ]
        .into_iter()
        .collect();

        Ok(LogProcessor {
            regexes,
            pipelines: Vec::new(),
            current_pipeline: Some(Pipeline::new()),
        })
    }

    pub fn process_line(&mut self, line: &str) -> Result<()> {
        if let Some(start_caps) = self.regexes["start"].captures(line) {
            let stage_name = start_caps.get(1).unwrap().as_str();
            let timestamp = extract_timestamp(line)?;

            if self.current_pipeline.is_none() || self.is_first_stage(stage_name) {
                if let Some(pipeline) = self.current_pipeline.take() {
                    self.pipelines.push(pipeline);
                }
                self.current_pipeline = Some(Pipeline::new());
            }

            if let Some(ref mut pipeline) = self.current_pipeline {
                pipeline.record_stage_start(stage_name, timestamp);
            }
        }

        if let Some(end_caps) = self.regexes["end"].captures(line) {
            let stage_name = end_caps.get(1).unwrap().as_str();
            let timestamp = extract_timestamp(line)?;
            let current_pipeline = &mut self.current_pipeline;
            if let Some(ref mut pipeline) = current_pipeline {
                pipeline.record_stage_end(stage_name, timestamp)?;
            }
        }

        if let Some(caps) = self.regexes["state_root"].captures(line) {
            if let (Some(matched_value), Some(unit)) = (caps.get(1), caps.get(2)) {
                let mut elapsed: f64 = matched_value.as_str().parse()?;
                if unit.as_str() == "ms" {
                    elapsed /= 1000.0;
                }
                let current_pipeline = &mut self.current_pipeline;
                if let Some(ref mut pipeline) = current_pipeline {
                    pipeline.update_stats("state_root", elapsed);
                }
            }
        }

        Ok(())
    }

    fn is_first_stage(&self, stage_name: &str) -> bool {
        stage_name == "Headers"
    }

    pub fn print_summary<W: std::io::Write>(&self, writer: &mut W) {
        let pipelines = &self.pipelines;
        let mut total_duration = Duration::new(0, 0);

        for (index, pipeline) in pipelines.iter().enumerate() {
            pipeline.print_summary(index, writer);
            total_duration += pipeline.durations.values().sum::<Duration>();
        }

        writeln!(
            writer,
            "Total Aggregate Duration: {}",
            format_duration(&total_duration)
        )
        .unwrap();
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
        let mut processor = LogProcessor::new().unwrap();
        let line = "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";

        processor.process_line(line).unwrap();

        let current_pipeline = processor.current_pipeline;

        assert!(current_pipeline.is_some());
        assert!(current_pipeline
            .as_ref()
            .unwrap()
            .stages
            .contains_key("Headers"));
    }

    #[test]
    fn test_process_line_end_stage() {
        let mut processor = LogProcessor::new().unwrap();
        let start_line = "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";
        let end_line = "2024-06-07T09:06:20.873354Z  INFO Finished stage pipeline_stages=1/12 stage=Headers checkpoint=20038569 target=344353";

        processor.process_line(start_line).unwrap();
        processor.process_line(end_line).unwrap();

        let current_pipeline = processor.current_pipeline;

        assert!(current_pipeline.is_some());
        assert!(current_pipeline
            .as_ref()
            .unwrap()
            .durations
            .contains_key("001 - Headers"));
    }

    #[test]
    fn test_process_line_state_root() {
        let mut processor = LogProcessor::new().unwrap();
        let line = "Validated state root elapsed=2.5s";

        processor.process_line(line).unwrap();

        let current_pipeline = processor.current_pipeline;

        assert!(current_pipeline.is_some());
        assert!(current_pipeline
            .as_ref()
            .unwrap()
            .stats
            .contains_key("state_root"));
    }

    #[test]
    fn test_print_summary() {
        let mut processor = LogProcessor::new().unwrap();
        let stages = [
            "Headers",
            "Bodies",
            "Receipts",
            "Senders",
            "Execution",
            "HashState",
            "IntermediateHashes",
            "AccountHashing",
            "StorageHashing",
            "MerkleTrie",
            "Finalization",
            "Refinement",
        ];

        for (i, stage) in stages.iter().enumerate() {
            let start_line = format!("2024-06-07T09:{:02}:00.000000Z  INFO Preparing stage pipeline_stages={}/12 stage={} checkpoint=20037711 target=1000230230", i, i+1, stage);
            let end_line = format!("2024-06-07T09:{:02}:30.000000Z  INFO Finished stage pipeline_stages={}/12 stage={} checkpoint=20038569 target=None stage_progress=100.00%", i, i+1, stage);

            processor.process_line(&start_line).unwrap();
            processor.process_line(&end_line).unwrap();
        }

        // Adding multiple "Preparing stage" entries for the same stage to test overwriting
        let additional_start_line = "2024-06-07T09:06:00.000000Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None";
        processor.process_line(additional_start_line).unwrap();

        let additional_end_line = "2024-06-07T09:06:30.000000Z  INFO Finished stage pipeline_stages=1/12 stage=Headers checkpoint=20038569 target=None";
        processor.process_line(additional_end_line).unwrap();

        // Finalize the last pipeline by pushing it to pipelines
        if let Some(pipeline) = processor.current_pipeline.take() {
            processor.pipelines.push(pipeline);
        }

        let mut output = Cursor::new(Vec::new());
        processor.print_summary(&mut output);

        let output_str = String::from_utf8(output.into_inner()).unwrap();

        assert!(output_str.contains("Pipeline 1:"));
        for (index, stage) in stages.iter().enumerate() {
            assert!(output_str.contains(&format!("Stage {:03} - {}:", index + 1, stage)));
        }
        assert!(output_str.contains("Total Pipeline Duration:"));
    }
}
