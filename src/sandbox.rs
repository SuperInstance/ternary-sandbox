use crate::{Agent, Environment, SeededRng};

/// Snapshot of the population at a given generation.
#[derive(Debug, Clone)]
pub struct PopulationSnapshot {
    pub generation: usize,
    pub agents: Vec<Agent>,
    pub best_fitness: f64,
    pub avg_fitness: f64,
    pub species_counts: Vec<usize>,
}

/// Configuration for the sandbox environment.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub population_size: usize,
    pub generations: usize,
    pub mutation_rate: f64,
    pub mutation_strength: f64,
    pub selection_pressure: f64, // tournament size fraction
    pub seed: u64,
    pub num_species: u8,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            population_size: 100,
            generations: 50,
            mutation_rate: 0.1,
            mutation_strength: 0.05,
            selection_pressure: 0.5,
            seed: 42,
            num_species: 3,
        }
    }
}

/// The sandbox: runs an evolutionary simulation.
pub struct Sandbox {
    pub config: SandboxConfig,
    pub environment: Environment,
    rng: SeededRng,
    agents: Vec<Agent>,
    generation: usize,
    next_id: u64,
}

impl Sandbox {
    pub fn new(config: SandboxConfig, environment: Environment) -> Self {
        let rng = SeededRng::new(config.seed);
        Self {
            config,
            environment,
            rng,
            agents: Vec::new(),
            generation: 0,
            next_id: 0,
        }
    }

    /// Initialize the population randomly within environment bounds.
    pub fn initialize(&mut self) {
        self.agents.clear();
        self.generation = 0;
        self.next_id = 0;
        for _ in 0..self.config.population_size {
            let x = self.rng.next_range(self.environment.x_range.0, self.environment.x_range.1);
            let y = self.rng.next_range(self.environment.y_range.0, self.environment.y_range.1);
            let species = self.rng.next_int(0, (self.config.num_species - 1) as i64) as u8;
            let mut agent = Agent::new(self.next_id, x, y, species);
            self.next_id += 1;
            agent.fitness = self.environment.evaluate(x, y, &mut self.rng);
            self.agents.push(agent);
        }
    }

    /// Run one generation: evaluate, select, reproduce, mutate.
    pub fn step(&mut self) -> PopulationSnapshot {
        // Evaluate fitness
        for agent in &mut self.agents {
            agent.fitness = self.environment.evaluate(agent.x, agent.y, &mut self.rng);
        }

        let snapshot = self.snapshot();

        // Selection & reproduction
        let mut new_agents = Vec::with_capacity(self.config.population_size);
        let tournament_size = ((self.config.population_size as f64 * self.config.selection_pressure)
            .max(2.0)) as usize;

        for _ in 0..self.config.population_size {
            // Tournament selection
            let parent = self.tournament_select(tournament_size);
            let mut child = parent.clone();
            child.id = self.next_id;
            self.next_id += 1;

            // Mutation
            if self.rng.next_bool(self.config.mutation_rate) {
                let xr = self.environment.x_range;
                let yr = self.environment.y_range;
                child.x = (child.x + self.rng.next_range(
                    -self.config.mutation_strength, self.config.mutation_strength))
                    .clamp(xr.0, xr.1);
                child.y = (child.y + self.rng.next_range(
                    -self.config.mutation_strength, self.config.mutation_strength))
                    .clamp(yr.0, yr.1);
            }

            new_agents.push(child);
        }

        self.agents = new_agents;
        self.generation += 1;
        snapshot
    }

    /// Run the full simulation and return all snapshots.
    pub fn run(&mut self) -> Vec<PopulationSnapshot> {
        self.initialize();
        let mut snapshots = Vec::new();
        for _ in 0..self.config.generations {
            snapshots.push(self.step());
        }
        snapshots
    }

    fn tournament_select(&mut self, k: usize) -> Agent {
        let n = self.agents.len();
        let mut best_idx = self.rng.next_int(0, (n - 1) as i64) as usize;
        for _ in 1..k {
            let idx = self.rng.next_int(0, (n - 1) as i64) as usize;
            if self.agents[idx].fitness > self.agents[best_idx].fitness {
                best_idx = idx;
            }
        }
        self.agents[best_idx].clone()
    }

    fn snapshot(&self) -> PopulationSnapshot {
        let fitnesses: Vec<f64> = self.agents.iter().map(|a| a.fitness).collect();
        let best = fitnesses.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let avg = if fitnesses.is_empty() { 0.0 } else { fitnesses.iter().sum::<f64>() / fitnesses.len() as f64 };
        let mut species_counts = vec![0usize; self.config.num_species as usize];
        for a in &self.agents {
            species_counts[a.species as usize] += 1;
        }
        PopulationSnapshot {
            generation: self.generation,
            agents: self.agents.clone(),
            best_fitness: best,
            avg_fitness: avg,
            species_counts,
        }
    }
}

/// Builder for sandbox configuration.
pub struct SandboxBuilder {
    config: SandboxConfig,
}

impl SandboxBuilder {
    pub fn new() -> Self {
        Self { config: SandboxConfig::default() }
    }

    pub fn seed(mut self, seed: u64) -> Self { self.config.seed = seed; self }
    pub fn population(mut self, size: usize) -> Self { self.config.population_size = size; self }
    pub fn generations(mut self, gens: usize) -> Self { self.config.generations = gens; self }
    pub fn mutation_rate(mut self, rate: f64) -> Self { self.config.mutation_rate = rate; self }
    pub fn mutation_strength(mut self, s: f64) -> Self { self.config.mutation_strength = s; self }
    pub fn selection_pressure(mut self, p: f64) -> Self { self.config.selection_pressure = p; self }
    pub fn species(mut self, n: u8) -> Self { self.config.num_species = n; self }

    pub fn build(self) -> SandboxConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::environment::Environment;

    #[test]
    fn sandbox_initialization() {
        let config = SandboxConfig::default();
        let env = Environment::new();
        let mut sb = Sandbox::new(config, env);
        sb.initialize();
        assert_eq!(sb.agents.len(), 100);
    }

    #[test]
    fn sandbox_step_advances_generation() {
        let config = SandboxConfig { generations: 2, ..Default::default() };
        let env = Environment::new();
        let mut sb = Sandbox::new(config, env);
        sb.initialize();
        sb.step();
        assert_eq!(sb.generation, 1);
    }

    #[test]
    fn sandbox_run_completes() {
        let config = SandboxBuilder::new().generations(10).population(20).build();
        let env = Environment::new();
        let mut sb = Sandbox::new(config, env);
        let results = sb.run();
        assert_eq!(results.len(), 10);
    }

    #[test]
    fn sandbox_deterministic_with_same_seed() {
        let config1 = SandboxBuilder::new().seed(99).generations(5).population(30).build();
        let config2 = SandboxBuilder::new().seed(99).generations(5).population(30).build();
        let env = Environment::new();
        let mut sb1 = Sandbox::new(config1, env.clone());
        let mut sb2 = Sandbox::new(config2, env);
        let r1 = sb1.run();
        let r2 = sb2.run();
        for (a, b) in r1.iter().zip(r2.iter()) {
            assert!((a.best_fitness - b.best_fitness).abs() < 1e-10);
            assert_eq!(a.species_counts, b.species_counts);
        }
    }

    #[test]
    fn species_counts_sum_to_population() {
        let config = SandboxBuilder::new().population(50).species(3).build();
        let env = Environment::new();
        let mut sb = Sandbox::new(config, env);
        let snaps = sb.run();
        for snap in snaps {
            assert_eq!(snap.species_counts.iter().sum::<usize>(), 50);
        }
    }
}
