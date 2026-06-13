# ternary-sandbox

A safe sandbox for **evolutionary agent experiments** with configurable fitness landscapes, deterministic seeded RNG, structured result capture, and side-by-side comparison. Agents evolve positions on a 2D landscape using tournament selection, mutation, and multi-species dynamics.

## Why It Matters

Validating ternary agent strategies requires reproducible experimentation. Real-world deployments can't be tested blindly — you need a controlled environment where you can:

1. **Replay exactly** — same seed → same result, every time
2. **Measure conservation** — Shannon entropy of species diversity across generations
3. **Compare configurations** — run A vs B with automatic winner selection
4. **Enforce constraints** — pre/post-conditions abort invalid experiments

The sandbox provides all four, using a xoshiro256** PRNG for deterministic randomness and a multi-peak fitness landscape with optional Gaussian noise.

## How It Works

### Fitness Landscape

The landscape is a sum of radial basis functions (one per peak):

```
f(x, y) = maxᵢ  hᵢ / (1 + 10 · ‖(x,y) - (xᵢ,yᵢ)‖)
```

where each peak *i* has position `(xᵢ, yᵢ)` and height `hᵢ`. With noise level `σ > 0`:

```
f̃(x, y) = max(0, f(x,y) + U(-σ, σ))
```

**Complexity:** O(P) per evaluation, where P = number of peaks.

### Tournament Selection

Each generation, parents are selected via tournament of size *k*:

```
k = max(2, ⌊population · selection_pressure⌋)
```

*k* random agents compete; the fittest becomes a parent. Selection pressure ∈ [0, 1] controls how aggressively the best agents dominate.

**Complexity:** O(k) per selection, O(N·k) per generation.

### Mutation

Each child is mutated with probability `mutation_rate`:

```
x' = clamp(x + U(-s, s), x_min, x_max)
y' = clamp(y + U(-s, s), y_min, y_max)
```

where *s* = `mutation_strength` (bounded to landscape range).

### Species Diversity and Shannon Entropy

Each agent belongs to one of *S* species. Per-generation entropy:

```
H(g) = - Σᵢ pᵢ · ln(pᵢ)   where pᵢ = count(species_i) / N
```

`H = ln(S)` is maximum diversity (uniform); `H = 0` is monoculture (one species dominant). The sandbox tracks entropy history, average, minimum, and extinction events.

### Deterministic PRNG (xoshiro256**)

State initialization via SplitMix64:

```
s₀ = seed
sᵢ₊₁ = sᵢ + 0x9e3779b97f4a7c15
state[j] = splitmix64(sⱼ)
```

Each `next_u64()` rotates and scrambles state. Period: 2²⁵⁶ − 1.

**Complexity:** O(1) per random draw.

### Experiment Lifecycle

```
pre_condition(config, env) → run sandbox → post_condition(result)
```

Pre/post-conditions are closures that return `bool`. Failure aborts with an error.

### Comparison

Two experiments run independently; results compared by:

- **Primary:** `fitness_diff = final_best_A - final_best_B`
- **Tiebreaker:** `entropy_diff = avg_entropy_A - avg_entropy_B`
- **Speed:** `faster_ms = duration_A - duration_B`

Winner: A if `fitness_diff > 0`; B if `< 0`; tiebreaker on entropy if equal.

## Quick Start

```rust
use ternary_sandbox::{SandboxBuilder, EnvironmentBuilder, Experiment};

let config = SandboxBuilder::new()
    .seed(42)
    .generations(50)
    .population(100)
    .mutation_rate(0.1)
    .build();

let env = EnvironmentBuilder::new()
    .peak(0.5, 0.5, 1.0)
    .noise(0.01)
    .build();

let result = Experiment::new("baseline", config, env)
    .run()
    .expect("experiment succeeds");

assert_eq!(result.generations_run, 50);
assert!(result.conservation.avg_entropy > 0.0);
```

## API

| Type | Key Methods |
|------|-------------|
| `Sandbox` | `initialize()`, `step()`, `run()` |
| `SandboxBuilder` | `.seed()`, `.population()`, `.generations()`, `.mutation_rate()` |
| `Environment` | `evaluate(x,y,rng)`, `evaluate_clean(x,y)`, `sample_grid(res)` |
| `EnvironmentBuilder` | `.peak(x,y,h)`, `.noise(σ)`, `.x_range()`, `.y_range()` |
| `Experiment` | `.run()`, `.pre_condition(f)`, `.post_condition(f)`, `::replay(seed, config, env)` |
| `Comparison` | `.run()` → `ComparisonResult` |
| `SeededRng` | `next_u64()`, `next_f64()`, `next_range(lo,hi)`, `next_int(lo,hi)` |

## Architecture Notes

The **γ + η = C** invariant is central to the sandbox. *Generation* (γ) is the evolutionary process — selection, mutation, reproduction producing new agent distributions. *Entropy* (η) is the Shannon entropy of species diversity (`ConservationMetrics`). *Conservation* (C) is the invariant that species counts always sum to `population_size` — no agent is created or destroyed, only transformed. Extinction events (η → 0 for a species) signal that C is being maintained by removing diversity rather than reducing population.

## References

- **Tournament selection:** Blickle, T. & Thiele, L. "A Comparison of Selection Schemes" (1995)
- **xoshiro256:** Blackman, D. & Vigna, S. "Scrambled Linear Pseudorandom Number Generators" (2019)
- **Shannon entropy in ecology:** Shannon, C. E. "A Mathematical Theory of Communication" (1948)
- **Evolutionary dynamics:** Nowak, M. *Evolutionary Dynamics* (2006)

## License

MIT
