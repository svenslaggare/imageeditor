pub struct TimeMeasurement {
    pattern: String,
    start_time: std::time::Instant
}

impl TimeMeasurement {
    pub fn new(pattern: &str) -> TimeMeasurement {
        TimeMeasurement {
            pattern: pattern.to_owned(),
            start_time: std::time::Instant::now()
        }
    }

    pub fn elapsed_ms(&self) -> f64 {
        return (std::time::Instant::now() - self.start_time).as_nanos() as f64 / 1.0E6
    }
}

impl Drop for TimeMeasurement {
    fn drop(&mut self) {
        println!("{}: {:.2} ms", self.pattern, self.elapsed_ms())
    }
}
