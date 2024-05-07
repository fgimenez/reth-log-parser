use regex::Regex;
use std::collections::HashMap;
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

    let regexes = vec![(
        "state_root",
        Regex::new(r"Validated state root.*elapsed=(\d+\.\d+)(ms|s)").unwrap(),
    )];

    let mut computations: HashMap<&str, Stats> = HashMap::new();

    for line in reader.lines() {
        let line = line?;
        for (label, regex) in &regexes {
            if let Some(caps) = regex.captures(&line) {
                if let (Some(matched_value), Some(unit)) = (caps.get(1), caps.get(2)) {
                    let mut elapsed: f64 = matched_value.as_str().parse().unwrap();
                    if unit.as_str() == "ms" {
                        elapsed /= 1000.0;
                    }
                    computations
                        .entry(label)
                        .or_insert_with(Stats::new)
                        .update(elapsed);
                }
            }
        }
    }

    for (label, stats) in computations {
        println!("{}:", label);
        stats.print();
    }

    Ok(())
}

struct Stats {
    count: usize,
    mean: f64,
    m2: f64,
}

impl Stats {
    fn new() -> Self {
        Stats {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    fn stddev(&self) -> f64 {
        if self.count > 1 {
            (self.m2 / (self.count - 1) as f64).sqrt()
        } else {
            0.0
        }
    }

    fn print(&self) {
        println!(
            "  Average elapsed time = {:.3} s, Standard deviation = {:.3} s",
            self.mean,
            self.stddev()
        );
    }
}
