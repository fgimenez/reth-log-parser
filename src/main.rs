use eyre::Result;
use log_parser::log_processor::LogProcessor;
use rayon::{prelude::*, ThreadPoolBuilder};
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
    sync::Arc,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_log_file>", args[0]);
        return Ok(());
    }

    // Configure the number of threads in the global rayon thread pool
    ThreadPoolBuilder::new().num_threads(4).build_global()?; // Adjust the number of threads as needed

    let path = Path::new(&args[1]);
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

    // Print the summary of each pipeline and the total aggregate duration
    processor.print_summary();

    Ok(())
}
