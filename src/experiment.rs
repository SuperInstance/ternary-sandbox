use std::time::Instant;
use crate::{Sandbox, SandboxConfig, Environment, PopulationSnapshot};

/// A single generation's fitness record.
#[derive(Debug, Clone, PartialEq)]
pub struct FitnessRecord {
    pub generation: usize,
    pub best_fitness: f64,
    pub avg_fitness: f64,
    pub worst_fitness: f64,
    pub species_counts: Vec<usize>,
}

/// Conservation metrics tracking species diversity over the experiment.
#[derive(Debug, Clone, PartialEq)]
pub struct ConservationMetrics {
    /// Shannon entropy at each generation
    pub entropy_history: Vec<f64>,
    /// Average entropy across the run
    pub avg_entropy: f64,
    /// Minimum entropy observed
    pub min_entropy: f64,
    /// Whether any species went extinct during the run
    pub any_extinction: bool,
    /// Generation at which first extinction occurred (if any)
    pub first_extinction_gen: Option<usize>,
}

/// Structured result from a completed experiment.
#[derive(Debug, Clone)]
pub struct ExperimentResult {
    pub seed: u64,
    pub config: SandboxConfig,
    pub fitness_history: Vec<FitnessRecord>,
    pub conservation: ConservationMetrics,
    pub duration_ms: u128,
    pub final_best_fitness: f64,
    pub generations_run: usize,
}

impl ExperimentResult {
    fn from_snapshots(seed: u64, config: SandboxConfig, snapshots: &[PopulationSnapshot], duration_ms: u128) -> Self {
        let fitness_history: Vec<FitnessRecord> = snapshots.iter().map(|s| {
            let worst = s.agents.iter().map(|a| a.fitness)
                .fold(f64::INFINITY, f64::min);
            FitnessRecord {
                generation: s.generation,
                best_fitness: s.best_fitness,
                avg_fitness: s.avg_fitness,
                worst_fitness: worst,
                species_counts: s.species_counts.clone(),
            }
        }).collect();

        let conservation = Self::compute_conservation(&fitness_history, config.num_species);
        let final_best = fitness_history.last().map(|f| f.best_fitness).unwrap_or(0.0);

        Self {
            seed,
            config,
            fitness_history,
            conservation,
            duration_ms,
            final_best_fitness: final_best,
            generations_run: snapshots.len(),
        }
    }

    fn compute_conservation(history: &[FitnessRecord], num_species: u8) -> ConservationMetrics {
        let mut entropy_history = Vec::new();
        let mut any_extinction = false;
        let mut first_extinction_gen = None;
        let n = num_species as usize;

        for record in history {
            let total: usize = record.species_counts.iter().sum();
            let entropy = if total == 0 {
                0.0
            } else {
                record.species_counts.iter().map(|&c| {
                    if c == 0 { 0.0 } else {
                        let p = c as f64 / total as f64;
                        -p * p.ln()
                    }
                }).sum()
            };
            entropy_history.push(entropy);

            let extinct = record.species_counts.iter().take(n).any(|&c| c == 0);
            if extinct && !any_extinction {
                any_extinction = true;
                first_extinction_gen = Some(record.generation);
            }
        }

        let avg_entropy = if entropy_history.is_empty() { 0.0 }
            else { entropy_history.iter().sum::<f64>() / entropy_history.len() as f64 };
        let min_entropy = entropy_history.iter().cloned().fold(f64::INFINITY, f64::min);

        ConservationMetrics {
            entropy_history,
            avg_entropy,
            min_entropy,
            any_extinction,
            first_extinction_gen,
        }
    }
}

/// An experiment: runs a full simulation with pre/post conditions and captures results.
pub struct Experiment {
    pub name: String,
    pub config: SandboxConfig,
    pub environment: Environment,
    /// Optional pre-condition check
    pub pre_condition: Option<Box<dyn Fn(&SandboxConfig, &Environment) -> bool>>,
    /// Optional post-condition check
    pub post_condition: Option<Box<dyn Fn(&ExperimentResult) -> bool>>,
}

impl Experiment {
    pub fn new(name: impl Into<String>, config: SandboxConfig, environment: Environment) -> Self {
        Self {
            name: name.into(),
            config,
            environment,
            pre_condition: None,
            post_condition: None,
        }
    }

    pub fn pre_condition(mut self, f: impl Fn(&SandboxConfig, &Environment) -> bool + 'static) -> Self {
        self.pre_condition = Some(Box::new(f));
        self
    }

    pub fn post_condition(mut self, f: impl Fn(&ExperimentResult) -> bool + 'static) -> Self {
        self.post_condition = Some(Box::new(f));
        self
    }

    /// Run the experiment. Returns Err if pre/post conditions fail.
    pub fn run(self) -> Result<ExperimentResult, String> {
        // Pre-condition check
        if let Some(ref pre) = self.pre_condition {
            if !pre(&self.config, &self.environment) {
                return Err(format!("Experiment '{}' failed pre-condition check", self.name));
            }
        }

        let start = Instant::now();
        let mut sandbox = Sandbox::new(self.config.clone(), self.environment);
        let snapshots = sandbox.run();
        let duration_ms = start.elapsed().as_millis();

        let result = ExperimentResult::from_snapshots(self.config.seed, self.config.clone(), &snapshots, duration_ms);

        // Post-condition check
        if let Some(ref post) = self.post_condition {
            if !post(&result) {
                return Err(format!("Experiment '{}' failed post-condition check", self.name));
            }
        }

        Ok(result)
    }

    /// Replay an experiment deterministically given a seed.
    /// Equivalent to run() but emphasizes reproducibility.
    pub fn replay(seed: u64, config: SandboxConfig, environment: Environment) -> ExperimentResult {
        let config = SandboxConfig { seed, ..config };
        let mut sandbox = Sandbox::new(config.clone(), environment);
        let start = Instant::now();
        let snapshots = sandbox.run();
        let duration_ms = start.elapsed().as_millis();
        ExperimentResult::from_snapshots(seed, config, &snapshots, duration_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{SandboxBuilder, EnvironmentBuilder};

    #[test]
    fn experiment_runs_successfully() {
        let config = SandboxBuilder::new().generations(5).population(20).build();
        let env = Environment::new();
        let exp = Experiment::new("test", config, env);
        let result = exp.run().expect("should succeed");
        assert_eq!(result.generations_run, 5);
    }

    #[test]
    fn experiment_pre_condition_fails() {
        let config = SandboxBuilder::new().build();
        let env = Environment::new();
        let exp = Experiment::new("fail_pre", config, env)
            .pre_condition(|_c, _e| false);
        assert!(exp.run().is_err());
    }

    #[test]
    fn experiment_post_condition_fails() {
        let config = SandboxBuilder::new().generations(2).population(10).build();
        let env = Environment::new();
        let exp = Experiment::new("fail_post", config, env)
            .post_condition(|_r| false);
        assert!(exp.run().is_err());
    }

    #[test]
    fn experiment_post_condition_passes() {
        let config = SandboxBuilder::new().generations(2).population(10).build();
        let env = Environment::new();
        let exp = Experiment::new("pass_post", config, env)
            .post_condition(|r| r.generations_run == 2);
        assert!(exp.run().is_ok());
    }

    #[test]
    fn replay_is_deterministic() {
        let config = SandboxBuilder::new().generations(10).population(30).build();
        let env = EnvironmentBuilder::new().peak(0.3, 0.3, 0.9).build();
        let r1 = Experiment::replay(777, config.clone(), env.clone());
        let r2 = Experiment::replay(777, config, env);
        assert_eq!(r1.fitness_history.len(), r2.fitness_history.len());
        for (a, b) in r1.fitness_history.iter().zip(r2.fitness_history.iter()) {
            assert!((a.best_fitness - b.best_fitness).abs() < 1e-10);
        }
    }

    #[test]
    fn conservation_metrics_computed() {
        let config = SandboxBuilder::new().generations(5).population(30).species(3).build();
        let env = Environment::new();
        let exp = Experiment::new("conservation", config, env);
        let result = exp.run().unwrap();
        assert!(!result.conservation.entropy_history.is_empty());
        assert!(result.conservation.avg_entropy >= 0.0);
    }
}
