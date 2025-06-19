# parkissat-sys

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Safe Rust bindings for the [ParKissat-RS](https://github.com/shaowei-cai-group/ParKissat-RS) parallel SAT solver, which won the SAT Competition 2022. This crate provides a safe, idiomatic Rust API while maintaining high performance through minimal overhead FFI calls.

## What is ParKissat-RS?

ParKissat-RS is a state-of-the-art parallel SAT solver that combines the efficiency of the Kissat solver with parallel processing capabilities. It's designed to solve Boolean satisfiability (SAT) problems efficiently using multiple threads and advanced heuristics.

## Features

- **Safe Rust API**: Memory-safe bindings with proper error handling
- **Parallel Solving**: Multi-threaded SAT solving for improved performance
- **Configurable**: Extensive configuration options for different use cases
- **DIMACS Support**: Load problems from standard DIMACS format files
- **Statistics**: Access to detailed solver statistics
- **Interruption Support**: Ability to interrupt long-running solves

## Installation

Since this crate is not published on crates.io, you need to use it as a Git dependency. Add this to your `Cargo.toml`:

```toml
[dependencies]
parkissat-sys = { git = "https://github.com/rrumana/ParKissat-rs.git" }
```

Or if you're working locally, clone the repository and use a path dependency:

```bash
git clone https://github.com/rrumana/ParKissat-rs.git
cd your-project
```

Then in your `Cargo.toml`:

```toml
[dependencies]
parkissat-sys = { path = "../path/to/parkissat-sys" }
```

### Building from Source

The crate includes the ParKissat-RS solver as a Git submodule. To build:

```bash
git clone --recursive https://github.com/rrumana/ParKissat-rs.git
cd ParKissat-rs/parkissat-sys
cargo build
```

If you've already cloned without `--recursive`, initialize the submodules:

```bash
git submodule update --init --recursive
```

## Quick Start

Here's a simple example of using parkissat-sys to solve a SAT problem:

```rust
use parkissat_sys::{ParkissatSolver, SolverConfig, SolverResult};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new solver instance
    let mut solver = ParkissatSolver::new()?;
    
    // Configure the solver
    let config = SolverConfig::default();
    solver.configure(&config)?;
    
    // Add clauses: (x1 ∨ x2) ∧ (¬x1 ∨ x2)
    solver.add_clause(&[1, 2])?;      // x1 ∨ x2
    solver.add_clause(&[-1, 2])?;     // ¬x1 ∨ x2
    
    // Solve the problem
    match solver.solve()? {
        SolverResult::Sat => {
            println!("Formula is satisfiable!");
            
            // Get the satisfying assignment
            let model = solver.get_model()?;
            println!("Model: {:?}", model);
            
            // Check specific variable values
            println!("x1 = {}", solver.get_model_value(1)?);
            println!("x2 = {}", solver.get_model_value(2)?);
        }
        SolverResult::Unsat => {
            println!("Formula is unsatisfiable");
        }
        SolverResult::Unknown => {
            println!("Result unknown (timeout or interrupted)");
        }
    }
    
    Ok(())
}
```

## Configuration Options

The solver can be configured with various parameters:

```rust
use std::time::Duration;
use parkissat_sys::SolverConfig;

let config = SolverConfig {
    num_threads: 4,                           // Use 4 parallel threads
    timeout: Duration::from_secs(300),        // 5 minute timeout
    random_seed: 42,                          // Fixed seed for reproducibility
    enable_preprocessing: true,               // Enable preprocessing
    verbosity: 1,                            // Moderate verbosity
};
```

## Loading DIMACS Files

You can also load SAT problems from DIMACS format files:

```rust
let mut solver = ParkissatSolver::new()?;
let config = SolverConfig::default();
solver.configure(&config)?;

// Load problem from DIMACS file
solver.load_dimacs("problem.cnf")?;

let result = solver.solve()?;
```

## Solving with Assumptions

For incremental solving, you can provide assumptions:

```rust
// Solve assuming x1 is true and x3 is false
let assumptions = vec![1, -3];
let result = solver.solve_with_assumptions(&assumptions)?;
```

## Error Handling

The crate provides comprehensive error handling through the [`ParkissatError`](src/error.rs) enum:

```rust
use parkissat_sys::{ParkissatError, Result};

match solver.add_clause(&[]) {
    Ok(_) => println!("Clause added successfully"),
    Err(ParkissatError::InvalidClause(msg)) => {
        eprintln!("Invalid clause: {}", msg);
    }
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Performance Tips

1. **Use multiple threads**: Set `num_threads` to match your CPU cores for parallel problems
2. **Enable preprocessing**: Can significantly reduce problem size for some instances
3. **Batch clause additions**: Add multiple clauses before solving when possible
4. **Reuse solver instances**: Avoid creating new solvers for related problems

## API Reference

### Core Types

- [`ParkissatSolver`](src/wrapper.rs): Main solver interface
- [`SolverConfig`](src/wrapper.rs): Configuration parameters
- [`SolverResult`](src/wrapper.rs): Solving results (Sat/Unsat/Unknown)
- [`ParkissatError`](src/error.rs): Error types

### Key Methods

- `ParkissatSolver::new()` - Create a new solver
- `configure(&config)` - Configure solver parameters
- `add_clause(&literals)` - Add a clause to the problem
- `solve()` - Solve the current problem
- `get_model()` - Get satisfying assignment (if SAT)
- `get_statistics()` - Get solver statistics

## Requirements

- Rust 1.70 or later
- C++ compiler (for building the native ParKissat-RS library)
- CMake (if building from source)

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Related Projects

- [ParKissat-RS](https://github.com/shaowei-cai-group/ParKissat-RS) - The underlying SAT solver
- [Kissat](https://github.com/arminbiere/kissat) - The base sequential SAT solver
- [SAT Competition](https://satcompetition.github.io/) - Annual SAT solver competition