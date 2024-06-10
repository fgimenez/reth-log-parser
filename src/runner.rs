use crate::log_processor::LogProcessor;
use eyre::Result;
use rayon::{prelude::*, ThreadPoolBuilder};
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    sync::Arc,
};

pub struct Runner<W: Write> {
    log_file: String,
    num_threads: usize,
    stdout_writer: W,
}

impl<W: Write> Runner<W> {
    pub fn builder() -> RunnerBuilder<W> {
        RunnerBuilder::default()
    }

    pub fn run(&mut self) -> Result<()> {
        ThreadPoolBuilder::new()
            .num_threads(self.num_threads)
            .build_global()?;

        let path = Path::new(&self.log_file);
        let file = File::open(path)?;
        let reader = BufReader::new(file);

        let processor = Arc::new(LogProcessor::new()?);

        reader.lines().par_bridge().for_each(|line| {
            if let Ok(line) = line {
                let processor = Arc::clone(&processor);
                processor.process_line(&line).unwrap_or_else(|err| {
                    eprintln!("Error processing line: {}", err);
                });
            }
        });

        // Capture the last pipeline if it was still in progress
        {
            let mut pipelines = processor.pipelines.lock().unwrap();
            let mut current_pipeline = processor.current_pipeline.lock().unwrap();
            if let Some(pipeline) = current_pipeline.take() {
                pipelines.push(pipeline);
            }
        }

        processor.print_summary(&mut self.stdout_writer);

        Ok(())
    }
}

pub struct RunnerBuilder<W: Write> {
    log_file: Option<String>,
    num_threads: Option<usize>,
    stdout_writer: Option<W>,
}

impl<W: Write> Default for RunnerBuilder<W> {
    fn default() -> Self {
        RunnerBuilder {
            log_file: None,
            num_threads: None,
            stdout_writer: None,
        }
    }
}

impl<W: Write> RunnerBuilder<W> {
    pub fn with_log_file(mut self, log_file: &str) -> Self {
        self.log_file = Some(log_file.to_string());
        self
    }

    pub fn with_threads(mut self, num_threads: usize) -> Self {
        self.num_threads = Some(num_threads);
        self
    }

    pub fn with_stdout_writer(mut self, stdout_writer: W) -> Self {
        self.stdout_writer = Some(stdout_writer);
        self
    }

    pub fn build(self) -> Result<Runner<W>> {
        Ok(Runner {
            log_file: self
                .log_file
                .ok_or_else(|| eyre::eyre!("log_file is required"))?,
            num_threads: self.num_threads.unwrap_or(4),
            stdout_writer: self
                .stdout_writer
                .ok_or_else(|| eyre::eyre!("stdout_writer is required"))?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use tempfile::NamedTempFile;

    #[test]
    fn test_runner_builder() {
        let log_file = NamedTempFile::new().unwrap();
        let num_threads = 8;
        let stdout_writer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let runner = Runner::builder()
            .with_log_file(log_file.path().to_str().unwrap())
            .with_threads(num_threads)
            .with_stdout_writer(stdout_writer)
            .build()
            .unwrap();

        assert_eq!(runner.log_file, log_file.path().to_str().unwrap());
        assert_eq!(runner.num_threads, num_threads);
    }

    #[test]
    fn test_runner_run() {
        let mut log_file = NamedTempFile::new().unwrap();
        writeln!(log_file, "2024-06-07T09:05:20.873354Z  INFO Preparing stage pipeline_stages=1/12 stage=Headers checkpoint=20037711 target=None").unwrap();
        writeln!(log_file, "2024-06-07T09:06:20.873354Z  INFO Finished stage pipeline_stages=1/12 stage=Headers checkpoint=20038569 target=None stage_progress=100.00%").unwrap();
        let stdout_writer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let mut runner = Runner::builder()
            .with_log_file(log_file.path().to_str().unwrap())
            .with_threads(4)
            .with_stdout_writer(stdout_writer)
            .build()
            .unwrap();

        runner.run().unwrap();

        let output = runner.stdout_writer.into_inner();
        let output_str = String::from_utf8(output).unwrap();

        // Check if the output contains expected summary text
        assert!(output_str.contains("Pipeline"));
        assert!(output_str.contains("Total Pipeline Duration"));
    }

    #[test]
    fn test_runner_builder_missing_log_file() {
        let stdout_writer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

        let result = Runner::builder()
            .with_threads(4)
            .with_stdout_writer(stdout_writer)
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_runner_builder_missing_stdout_writer() {
        let log_file = NamedTempFile::new().unwrap();

        let result: Result<Runner<Cursor<Vec<u8>>>> = Runner::builder()
            .with_log_file(log_file.path().to_str().unwrap())
            .with_threads(4)
            .build();

        assert!(result.is_err());
    }
}
