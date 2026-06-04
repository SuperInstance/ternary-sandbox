//! # ternary-sandbox
//!
//! A safe sandbox for running ternary agent experiments with configurable
//! environments, repeatable seeds, and structured result capture.

mod rng;
mod environment;
mod sandbox;
mod experiment;
mod comparison;

pub use rng::SeededRng;
pub use environment::{Environment, EnvironmentBuilder, LandscapePoint};
pub use sandbox::{Sandbox, SandboxBuilder, SandboxConfig, PopulationSnapshot};
pub use experiment::{Experiment, ExperimentResult, FitnessRecord, ConservationMetrics};
pub use comparison::{Comparison, ComparisonResult, ComparisonWinner};

/// A ternary agent with a position in the landscape and a fitness score.
#[derive(Debug, Clone)]
pub struct Agent {
    pub id: u64,
    pub x: f64,
    pub y: f64,
    pub fitness: f64,
    pub species: u8,
}

impl Agent {
    fn new(id: u64, x: f64, y: f64, species: u8) -> Self {
        Self { id, x, y, fitness: 0.0, species }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_creation() {
        let a = Agent::new(1, 0.5, 0.5, 0);
        assert_eq!(a.id, 1);
        assert_eq!(a.species, 0);
        assert_eq!(a.fitness, 0.0);
    }
}
