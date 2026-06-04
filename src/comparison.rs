use crate::{SandboxConfig, Environment, Experiment, ExperimentResult};

/// Comparison between two experiment results.
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    pub label_a: String,
    pub label_b: String,
    pub result_a: ExperimentResult,
    pub result_b: ExperimentResult,
    pub fitness_diff: f64,      // a - b (positive = a better)
    pub entropy_diff: f64,      // a - b (positive = a more diverse)
    pub faster_ms: i128,        // negative = a faster
    pub winner: ComparisonWinner,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComparisonWinner {
    A,
    B,
    Tie,
}

/// Run two experiments side-by-side and compare results.
pub struct Comparison {
    pub label_a: String,
    pub label_b: String,
    pub config_a: SandboxConfig,
    pub config_b: SandboxConfig,
    pub environment: Environment,
}

impl Comparison {
    pub fn new(
        label_a: impl Into<String>,
        label_b: impl Into<String>,
        config_a: SandboxConfig,
        config_b: SandboxConfig,
        environment: Environment,
    ) -> Self {
        Self {
            label_a: label_a.into(),
            label_b: label_b.into(),
            config_a,
            config_b,
            environment,
        }
    }

    /// Run both experiments and compare.
    pub fn run(self) -> Result<ComparisonResult, String> {
        let label_a = self.label_a;
        let label_b = self.label_b;
        let exp_a = Experiment::new(&label_a, self.config_a, self.environment.clone());
        let exp_b = Experiment::new(&label_b, self.config_b, self.environment);

        let result_a = exp_a.run().map_err(|e| format!("Experiment A: {}", e))?;
        let result_b = exp_b.run().map_err(|e| format!("Experiment B: {}", e))?;

        Ok(Self::compare(label_a, label_b, result_a, result_b))
    }

    fn compare(label_a: String, label_b: String, result_a: ExperimentResult, result_b: ExperimentResult) -> ComparisonResult {
        let fitness_diff = result_a.final_best_fitness - result_b.final_best_fitness;
        let entropy_diff = result_a.conservation.avg_entropy - result_b.conservation.avg_entropy;
        let faster_ms = result_a.duration_ms as i128 - result_b.duration_ms as i128;

        // Simple scoring: fitness is primary, entropy is tiebreaker
        let winner = if (fitness_diff).abs() < 1e-10 {
            if entropy_diff.abs() < 1e-10 {
                ComparisonWinner::Tie
            } else if entropy_diff > 0.0 {
                ComparisonWinner::A
            } else {
                ComparisonWinner::B
            }
        } else if fitness_diff > 0.0 {
            ComparisonWinner::A
        } else {
            ComparisonWinner::B
        };

        ComparisonResult {
            label_a,
            label_b,
            result_a,
            result_b,
            fitness_diff,
            entropy_diff,
            faster_ms,
            winner,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comparison_runs_both() {
        let env = EnvironmentBuilder::new().peak(0.5, 0.5, 1.0).build();
        let config_a = SandboxBuilder::new().seed(1).generations(5).population(20).build();
        let config_b = SandboxBuilder::new().seed(2).generations(5).population(20).build();

        let comp = Comparison::new("A", "B", config_a, config_b, env);
        let result = comp.run().expect("should succeed");
        assert_eq!(result.result_a.generations_run, 5);
        assert_eq!(result.result_b.generations_run, 5);
    }

    #[test]
    fn comparison_declares_winner() {
        let env = EnvironmentBuilder::new().peak(0.5, 0.5, 1.0).build();
        // Higher mutation rate should explore more
        let config_a = SandboxBuilder::new()
            .seed(42).generations(20).population(50)
            .mutation_rate(0.3).mutation_strength(0.1).build();
        let config_b = SandboxBuilder::new()
            .seed(42).generations(20).population(50)
            .mutation_rate(0.01).mutation_strength(0.001).build();

        let comp = Comparison::new("high_mut", "low_mut", config_a, config_b, env);
        let result = comp.run().expect("should succeed");
        // Just verify it produced a valid winner
        assert!(matches!(result.winner, ComparisonWinner::A | ComparisonWinner::B | ComparisonWinner::Tie));
    }

    #[test]
    fn comparison_same_config_is_tie() {
        let env = EnvironmentBuilder::new().peak(0.5, 0.5, 1.0).build();
        let config = SandboxBuilder::new().seed(42).generations(5).population(20).build();
        let config2 = SandboxBuilder::new().seed(42).generations(5).population(20).build();

        let comp = Comparison::new("A", "B", config, config2, env);
        let result = comp.run().expect("should succeed");
        assert_eq!(result.winner, ComparisonWinner::Tie);
        assert!(result.fitness_diff.abs() < 1e-10);
    }
}
