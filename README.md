# Ternary Sandbox — Safe Experimentation Environment for Ternary Agent Systems

**Ternary Sandbox** provides a controlled, repeatable environment for running ternary agent experiments. It includes a seeded RNG for deterministic reproduction, configurable fitness landscapes, population snapshots, experiment comparison, and conservation metric tracking — all without side effects on production systems.

## Why It Matters

Scientific rigor requires controlled experiments: same inputs → same outputs. For ternary agent research, this means reproducible RNG seeds, isolated environments, and structured result capture. The sandbox provides all three. Without it, experiments are non-reproducible: random seeds vary between runs, side effects leak between experiments, and results can't be compared. The sandbox makes ternary research *scientific* rather than anecdotal.

## How It Works

### Seeded RNG

`SeededRng` provides deterministic pseudo-random number generation. Same seed → identical sequence across platforms and runs. Used for: agent initialization, stochastic transitions, and landscape generation.

### Environment

`EnvironmentBuilder` constructs a fitness landscape with configurable dimensions, peaks, valleys, and noise. Agents are placed on the landscape and their fitness is evaluated each tick. The landscape is a 2D function over (x, y) returning a fitness value.

### Sandbox

`Sandbox` is the main container:

1. Initialize population with seeded RNG
2. Each tick: evaluate fitness, apply transitions, record snapshot
3. Collect metrics: mean fitness, diversity, entropy, conservation

The sandbox runs for a configured number of ticks, producing `PopulationSnapshot` records at each step.

### Experiment

`Experiment` wraps a sandbox with a parameter set and runs to completion, producing `ExperimentResult` with:
- `FitnessRecord`: Time-series of population fitness
- `ConservationMetrics`: γ + η = C verification at each tick

### Comparison

`Comparison` runs multiple experiments and ranks them by outcome. The `ComparisonResult` reports which parameter set produced the best outcome and by how much.

## Quick Start

```rust
use ternary_sandbox::{SandboxBuilder, SandboxConfig};

let config = SandboxConfig {
    population: 300,
    ticks: 1000,
    seed: 42,
};

let mut sandbox = SandboxBuilder::new()
    .config(config)
    .build();

sandbox.run();

let result = sandbox.result();
println!("Final fitness: {:.3}", result.mean_fitness);
println!("Conservation satisfied: {}", result.conservation_verified);
```

```bash
cargo add ternary-sandbox
```

## API

| Type / Function | Description |
|---|---|
| `Sandbox` | Main experiment container: `run()`, `result()` |
| `SandboxBuilder` | Fluent builder for sandbox configuration |
| `SeededRng` | Deterministic RNG: `new(seed)`, `next() → f64` |
| `Environment` | Fitness landscape with configurable topology |
| `ExperimentResult` | Fitness records + conservation metrics |
| `Comparison` | Multi-experiment comparison |

## Architecture Notes

The sandbox is the experimental testbed for **SuperInstance** fleet dynamics. Every parameter claim, every conservation verification, every optimization decision is validated in the sandbox before fleet deployment. The γ + η = C conservation law is checked at every tick — if violated, the experiment is flagged. See [Architecture](https://github.com/SuperInstance/SuperInstance/blob/main/ARCHITECTURE.md).

## References

- Axelrod, Robert. *The Complexity of Cooperation*, Princeton UP, 1997 — agent-based modeling.
| Wilensky, Uri & Rand, William. *An Introduction to Agent-Based Modeling*, MIT Press, 2015.
| Sanfilippo, Francesco et al. "Ternary Quantum Computers," *arXiv*, 2018.

## License

MIT
