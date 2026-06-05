# ternary-sandbox

A safe sandbox for running ternary agent experiments — configurable environments, repeatable seeds, and structured result capture.

Pure Rust, no unsafe code, no external dependencies.

## Quick Start

```rust
use ternary_sandbox::*;

// Build an environment with fitness peaks
let env = EnvironmentBuilder::new()
    .peak(0.3, 0.3, 0.8)   // (x, y, height)
    .peak(0.7, 0.7, 1.0)
    .noise(0.02)
    .build();

// Configure the sandbox
let config = SandboxBuilder::new()
    .seed(42)
    .population(100)
    .generations(50)
    .mutation_rate(0.1)
    .mutation_strength(0.05)
    .selection_pressure(0.5)
    .species(3)
    .build();

// Run an experiment
let experiment = Experiment::new("my-experiment", config, env);
let result = experiment.run().unwrap();

println!("Best fitness: {:.4}", result.final_best_fitness);
println!("Generations: {}", result.generations_run);
println!("Avg entropy: {:.4}", result.conservation.avg_entropy);
println!("Duration: {}ms", result.duration_ms);
```

## Core Concepts

### Environment

Defines the problem landscape with configurable peaks, valleys, and noise:

```rust
let env = EnvironmentBuilder::new()
    .peak(0.5, 0.5, 1.0)   // Center peak, height 1.0
    .peak(0.2, 0.8, 0.5)   // Off-center peak, height 0.5
    .noise(0.05)            // Add fitness evaluation noise
    .x_range(-1.0, 1.0)    // Custom bounds
    .y_range(-1.0, 1.0)
    .build();
```

### Sandbox

The evolutionary simulation engine:

- **Population**: N agents with positions and species tags
- **Selection**: Tournament selection with configurable pressure
- **Mutation**: Gaussian-ish perturbation with configurable rate and strength
- **Species**: Track diversity with multi-species populations

```rust
let config = SandboxBuilder::new()
    .seed(12345)            // Deterministic
    .population(200)
    .generations(100)
    .mutation_rate(0.15)
    .mutation_strength(0.08)
    .selection_pressure(0.3)
    .species(5)
    .build();

let mut sandbox = Sandbox::new(config, env);
sandbox.initialize();
let snapshot = sandbox.step();   // One generation
// or
let history = sandbox.run();     // Full run
```

### Experiment

Run a full experiment with pre/post conditions:

```rust
let result = Experiment::new("controlled", config, env)
    .pre_condition(|cfg, _env| cfg.population_size > 10)
    .post_condition(|result| result.final_best_fitness > 0.5)
    .run();
```

### Replay

Reproduce any experiment exactly from its seed:

```rust
let result = Experiment::replay(42, config, env);
```

### Comparison

Run two experiments side-by-side:

```rust
let comp = Comparison::new(
    "high-mutation",
    "low-mutation",
    config_high,
    config_low,
    env,
);
let result = comp.run().unwrap();
match result.winner {
    ComparisonWinner::A => println!("High mutation wins!"),
    ComparisonWinner::B => println!("Low mutation wins!"),
    ComparisonWinner::Tie => println!("It's a tie!"),
}
```

## Result Metrics

`ExperimentResult` includes:

- **Fitness history**: best, average, worst per generation
- **Conservation metrics**: Shannon entropy, extinction tracking
- **Species counts**: population breakdown per generation
- **Duration**: wall-clock timing

## Testing

```bash
cargo test
```

## License

MIT

## See Also
- **ternary-experiment** — related
- **ternary-fitness** — related
- **ternary-benchmark** — related
- **ternary-agent** — related
- **ternary-validation** — related

