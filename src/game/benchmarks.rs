use std::{fmt::Display, time::{Duration, Instant}};

pub struct BenchmarkStatistic {
    timer: Instant,
    max_samples: usize,
    name: String,
    n_name: Option<String>,
    benchmarks: Vec<Duration>,
    n: usize,
}

impl BenchmarkStatistic {
    pub fn new(name: String, n_name: Option<String>, max_samples: usize) -> Self {
        Self { 
            timer: Instant::now(),
            max_samples,
            name,
            n_name,
            benchmarks: vec![],
            n: 0,
        }
    }

    pub fn start(&mut self) {
        self.timer = Instant::now();
    }

    pub fn stop(&mut self, n: Option<usize>) {
        let benchmark = self.timer.elapsed();
        if let Some(n) = n {
            self.n += n;
        }
        self.benchmarks.push(benchmark);

        if self.benchmarks.len() >= self.max_samples {
            println!("{}", self);
            self.benchmarks.clear();
            self.n = 0;
        }
    }

    // average benchmarks in micros
    pub fn get_averages(&self) -> (f64, f64) {
        let total: f64 = self.benchmarks.iter().map(|d| d.as_secs_f64()).sum();
        let avg = total / self.benchmarks.len() as f64;
        let avg_to_n = total / self.n as f64;

        (avg * 1_000_000.0, avg_to_n * 1_000_000.0)
    }
}

impl Display for BenchmarkStatistic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (avg_time, avg_time_per_n) = self.get_averages();
        let avg_n = self.n / self.max_samples;
        if let Some(n_name) = &self.n_name {
            write!(f, "\n\nBenchmark analysis for {}. 
            \nAvg time per frame: {}µs. \nAvg {}s per frame: {}. \nAvg time per {}: {}µs", self.name,
            avg_time, n_name, avg_n, n_name, avg_time_per_n) 
        } else {
            write!(f, "\n\nBenchmarks analysis for {}.
            \nTime per frame: {}µs", self.name, avg_time)        
        }
    }
}

#[allow(dead_code)]
pub struct BenchmarkTests {
    pub rigid_collision_detection: BenchmarkStatistic,
    pub rigid_collision_solving: BenchmarkStatistic,
    pub string_constraint_detection: BenchmarkStatistic,
    pub string_constraint_solving: BenchmarkStatistic,
    pub updating: BenchmarkStatistic,
    pub rendering: BenchmarkStatistic,
}

impl Default for BenchmarkTests {
    fn default() -> Self {
        let rigid_collision_detection = BenchmarkStatistic::new(
            "rigid body collision detection".to_string(), 
            Some("rigid body".to_string()), 
            1000,
        );

        let rigid_collision_solving = BenchmarkStatistic::new(
            "rigid body collision solving".to_string(), 
            Some("collision".to_string()), 
            1000,
        );

        let string_constraint_detection = BenchmarkStatistic::new(
            "string collision constraint detection".to_string(), 
            Some("string joint".to_string()), 
            1000,
        );

        let string_constraint_solving = BenchmarkStatistic::new(
            "string constraint solving".to_string(), 
            Some("constraint".to_string()), 
            1000,
        );

        let updating = BenchmarkStatistic::new(
            "simulation update".to_string(), 
            None, 
            1000,
        );

        let rendering = BenchmarkStatistic::new(
            "simulation rendering".to_string(), 
            Some("rigid body".to_string()), 
            1000,
        );

        Self { 
            rigid_collision_detection, 
            rigid_collision_solving, 
            string_constraint_detection, 
            string_constraint_solving, 
            updating, 
            rendering, 
        }           
    }
}