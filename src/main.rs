use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <path_to_log_file>", args[0]);
        return Ok(());
    }

    let path = Path::new(&args[1]);
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let elapsed_regex = Regex::new(r"elapsed=(\d+\.\d+)ms").unwrap();
    let mut n = 0;
    let mut mean = 0.0;
    let mut m2 = 0.0;

    for line in reader.lines() {
        let line = line?;
        if let Some(caps) = elapsed_regex.captures(&line) {
            if let Some(matched) = caps.get(1) {
                let elapsed: f64 = matched.as_str().parse().unwrap();
                n += 1;
                let delta = elapsed - mean;
                mean += delta / n as f64;
                let delta2 = elapsed - mean;
                m2 += delta * delta2;
            }
        }
    }

    if n == 0 {
        println!("No elapsed times found.");
        return Ok(());
    }

    let variance = if n > 1 { m2 / (n - 1) as f64 } else { 0.0 };
    let stddev = variance.sqrt();

    println!("Average elapsed time: {:.3} ms", mean);
    println!("Standard deviation: {:.3} ms", stddev);

    Ok(())
}
