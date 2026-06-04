use crate::SeededRng;

/// Defines the problem landscape for ternary agents.
#[derive(Debug, Clone)]
pub struct Environment {
    /// Number of fitness peaks in the landscape
    pub peaks: Vec<(f64, f64, f64)>, // (x, y, height)
    /// Global noise level applied to fitness evaluations
    pub noise_level: f64,
    /// X bounds of the landscape
    pub x_range: (f64, f64),
    /// Y bounds of the landscape
    pub y_range: (f64, f64),
}

/// A point in the landscape with its computed fitness.
#[derive(Debug, Clone, PartialEq)]
pub struct LandscapePoint {
    pub x: f64,
    pub y: f64,
    pub fitness: f64,
}

impl Environment {
    /// Create a new flat environment with default bounds.
    pub fn new() -> Self {
        Self {
            peaks: vec![(0.5, 0.5, 1.0)],
            noise_level: 0.0,
            x_range: (0.0, 1.0),
            y_range: (0.0, 1.0),
        }
    }

    /// Evaluate the fitness of a position, optionally with noise.
    pub fn evaluate(&self, x: f64, y: f64, rng: &mut SeededRng) -> f64 {
        let mut fitness: f64 = 0.0;
        for (px, py, height) in &self.peaks {
            let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            fitness = fitness.max(height / (1.0 + dist * 10.0));
        }
        if self.noise_level > 0.0 {
            fitness += rng.next_range(-self.noise_level, self.noise_level);
        }
        fitness.max(0.0)
    }

    /// Evaluate without noise (deterministic).
    pub fn evaluate_clean(&self, x: f64, y: f64) -> f64 {
        let mut fitness: f64 = 0.0;
        for (px, py, height) in &self.peaks {
            let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            fitness = fitness.max(height / (1.0 + dist * 10.0));
        }
        fitness.max(0.0)
    }

    /// Sample the landscape on a grid.
    pub fn sample_grid(&self, resolution: usize) -> Vec<LandscapePoint> {
        let mut points = Vec::new();
        let xr = self.x_range;
        let yr = self.y_range;
        for i in 0..resolution {
            for j in 0..resolution {
                let x = xr.0 + (xr.1 - xr.0) * (i as f64 / (resolution - 1).max(1) as f64);
                let y = yr.0 + (yr.1 - yr.0) * (j as f64 / (resolution - 1).max(1) as f64);
                points.push(LandscapePoint {
                    x, y,
                    fitness: self.evaluate_clean(x, y),
                });
            }
        }
        points
    }
}

/// Builder for constructing environments.
pub struct EnvironmentBuilder {
    env: Environment,
}

impl EnvironmentBuilder {
    pub fn new() -> Self {
        Self { env: Environment::new() }
    }

    pub fn peak(mut self, x: f64, y: f64, height: f64) -> Self {
        self.env.peaks.push((x, y, height));
        self
    }

    pub fn noise(mut self, level: f64) -> Self {
        self.env.noise_level = level;
        self
    }

    pub fn x_range(mut self, lo: f64, hi: f64) -> Self {
        self.env.x_range = (lo, hi);
        self
    }

    pub fn y_range(mut self, lo: f64, hi: f64) -> Self {
        self.env.y_range = (lo, hi);
        self
    }

    pub fn build(self) -> Environment {
        self.env
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_env_has_one_peak() {
        let env = Environment::new();
        assert_eq!(env.peaks.len(), 1);
    }

    #[test]
    fn peak_center_has_highest_fitness() {
        let env = Environment::new();
        let fitness = env.evaluate_clean(0.5, 0.5);
        let off = env.evaluate_clean(0.7, 0.7);
        assert!(fitness > off);
    }

    #[test]
    fn builder_adds_peaks() {
        let env = EnvironmentBuilder::new()
            .peak(0.2, 0.3, 0.8)
            .peak(0.7, 0.7, 1.0)
            .build();
        assert_eq!(env.peaks.len(), 3); // default + 2
    }

    #[test]
    fn sample_grid_returns_correct_count() {
        let env = Environment::new();
        let grid = env.sample_grid(5);
        assert_eq!(grid.len(), 25);
    }

    #[test]
    fn noise_perturbation_changes_fitness() {
        let env = EnvironmentBuilder::new().noise(0.1).build();
        let mut rng = SeededRng::new(42);
        let a = env.evaluate(0.5, 0.5, &mut rng);
        let b = env.evaluate(0.5, 0.5, &mut rng);
        // With noise, two evaluations at the same point should differ
        assert_ne!(a, b);
    }
}
