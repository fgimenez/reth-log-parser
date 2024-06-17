#[derive(Default, Clone)]
pub struct Stats {
    pub count: usize,
    pub mean: f64,
    pub m2: f64,
}

impl Stats {
    pub fn update(&mut self, value: f64) {
        self.count += 1;
        let delta = value - self.mean;
        self.mean += delta / self.count as f64;
        let delta2 = value - self.mean;
        self.m2 += delta * delta2;
    }
}
