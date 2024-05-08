use eyre::Result;
use log_parser::LogProcessor;
use regex::Regex;
use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_log_file>", args[0]);
        return Ok(());
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let regexes = vec![(
        "state_root".to_string(),
        Regex::new(r"Validated state root.*elapsed=(\d+\.\d+)(ms|s)")?,
    )];
    let mut processor = LogProcessor::new(regexes);

    for line in reader.lines() {
        let line = line?;
        processor.process_line(&line)?;
    }

    // Print results for each computation type
    for (label, stats) in processor.get_computations() {
        println!(
            "{}: Average elapsed time = {:.3} s, Standard deviation = {:.3} s",
            label,
            stats.mean,
            stats.stddev()
        );
    }

    Ok(())
}
