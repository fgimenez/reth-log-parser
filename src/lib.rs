use eyre::Result;
use regex::Regex;
use std::collections::HashMap;

#[derive(Default)]
pub struct Stats {
    pub count: usize,
    pub mean: f64,
    pub m2: f64,
}

impl Stats {
    pub fn new() -> Self {
        Stats {
            count: 0,
            mean: 0.0,
            m2: 0.0,
        }
    }

    pub fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }

    pub fn stddev(&self) -> f64 {
        if self.count > 1 {
            (self.m2 / (self.count - 1) as f64).sqrt()
        } else {
            0.0
        }
    }
}

pub struct LogProcessor {
    regexes: Vec<(String, Regex)>,
    computations: HashMap<String, Stats>,
}

impl LogProcessor {
    pub fn new(regexes: Vec<(String, Regex)>) -> Self {
        LogProcessor {
            regexes,
            computations: HashMap::new(),
        }
    }

    pub fn process_line(&mut self, line: &str) -> Result<()> {
        for (label, regex) in &self.regexes {
            if let Some(caps) = regex.captures(line) {
                if let (Some(matched_value), Some(unit)) = (caps.get(1), caps.get(2)) {
                    let mut elapsed: f64 = matched_value.as_str().parse()?;
                    if unit.as_str() == "ms" {
                        elapsed /= 1000.0;
                    }
                    self.computations
                        .entry(label.clone())
                        .or_default()
                        .update(elapsed);
                }
            }
        }
        Ok(())
    }

    pub fn get_computations(&self) -> &HashMap<String, Stats> {
        &self.computations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_multiple_computation_types() {
        let regexes = vec![
            (
                "state_root".to_string(),
                Regex::new(r"Validated state root.*elapsed=(\d+\.\d+)(ms|s)").unwrap(),
            ),
            (
                "other_computation".to_string(),
                Regex::new(r"Some other computation.*elapsed=(\d+\.\d+)(ms|s)").unwrap(),
            ),
        ];
        let mut processor = LogProcessor::new(regexes);

        // Test inputs for different computation types
        let lines = vec![
            "2024-05-05T00:43:56.963520Z DEBUG blockchain_tree::chain: Validated state root number=19800396 hash=0x90a13c04d0fad10ef62ce482a6b878e8f6c9502e882e43cac2c415b3dfc1990a elapsed=937.929415ms",
            "2024-05-05T00:46:09.023209Z DEBUG blockchain_tree::chain: Some other computation number=19800407 hash=0x6412fbd4ca061ef5413315eb4554aaf3aafa2384cec0032b59adaf001686f335 elapsed=2.115731592s",
        ];

        for line in lines {
            processor.process_line(line).unwrap();
        }
        let computations = processor.get_computations();

        assert_eq!(computations["state_root"].count, 1);
        let expected_elapsed_time = 0.937929415;
        assert!((computations["state_root"].mean - expected_elapsed_time).abs() < 1e-6);

        assert_eq!(computations["other_computation"].count, 1);
        let expected_other_elapsed_time = 2.115731592;
        assert!(
            (computations["other_computation"].mean - expected_other_elapsed_time).abs() < 1e-6
        );

        assert_eq!(computations["state_root"].stddev(), 0.0);
        assert_eq!(computations["other_computation"].stddev(), 0.0);
    }
}
