# parkissat-sys

[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

Safe Rust bindings for the [ParKissat-RS](https://github.com/shaowei-cai-group/ParKissat-RS) parallel SAT solver, which won the SAT Competition 2022. This crate provides a safe, idiomatic Rust API while maintaining high performance through minimal overhead FFI calls.

## What is ParKissat-RS?

ParKissat-RS is a state-of-the-art parallel SAT solver that combines the efficiency of the Kissat solver with parallel processing capabilities. It's designed to solve Boolean satisfiability (SAT) problems efficiently using multiple threads and advanced heuristics.

## Features

- **Safe Rust API**: Memory-safe bindings with proper error handling
- **Parallel Solving**: Multi-threaded SAT solving with automatic CPU detection
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
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new solver instance
    let mut solver = ParkissatSolver::new()?;
    
    // Configure the solver
    let config = SolverConfig {
        num_threads: -1,  // Auto-detect available CPUs (or use positive number for explicit count)
        timeout: Duration::from_secs(300),
        random_seed: 42,
        enable_preprocessing: true,
        verbosity: 1,
    };
    solver.configure(&config)?;
    
    // Add clauses: (x1 ∨ x2) ∧ (¬x1 ∨ x2)
    solver.add_clause(&[1, 2])?;      // x1 ∨ x2
    solver.add_clause(&[-1, 2])?;     // ¬x1 ∨ x2
    
    // Solve the problem
    match solver.solve()? {
        SolverResult::Sat(assignment) => {
            println!("Formula is satisfiable!");
            println!("Assignment: {:?}", assignment);
            
            // Check specific variable values
            if let Some(&value) = assignment.get(&1) {
                println!("x1 = {}", value);
            }
            if let Some(&value) = assignment.get(&2) {
                println!("x2 = {}", value);
            }
        }
        SolverResult::Unsat => {
            println!("Formula is unsatisfiable");
        }
        SolverResult::Unknown => {
            println!("Result unknown (timeout or interrupted)");
        }
    }
    
    // Get solver statistics
    let stats = solver.statistics();
    println!("Variables: {}, Clauses: {}", stats.variables, stats.clauses);
    
    Ok(())
}
```

## Configuration Options

The solver can be configured with various parameters:

```rust
use parkissat_sys::SolverOptions;

let options = SolverOptions {
    timeout_seconds: Some(300.0),    // 5 minute timeout
    random_seed: Some(42),           // Fixed seed for reproducibility
    verbosity: 1,                    // Moderate verbosity (0=quiet, 1=normal, 2=verbose)
};

// Apply configuration to solver
solver.configure(&options)?;
```

## Threading Configuration

The number of threads is specified when creating the solver:

```rust
// Create solver with different thread counts
let solver_1t = ParkissatSolver::new(1)?;   // Single-threaded
let solver_4t = ParkissatSolver::new(4)?;   // 4 threads
let solver_8t = ParkissatSolver::new(8)?;   // 8 threads
```

## Loading DIMACS Files

You can load SAT problems from DIMACS format files:

```rust
let mut solver = ParkissatSolver::new(4)?;
let options = SolverOptions::default();
solver.configure(&options)?;

// Load problem from DIMACS file
solver.load_dimacs("problem.cnf")?;

let result = solver.solve()?;
```

## Error Handling

The crate provides comprehensive error handling through the `anyhow::Error` type:

```rust
use anyhow::Result;

match solver.add_clause(&[]) {
    Ok(_) => println!("Clause added successfully"),
    Err(e) => eprintln!("Error adding clause: {}", e),
}
```

## Performance Tips

1. **Use multiple threads**: Specify thread count when creating solver to match your CPU cores
2. **Batch clause additions**: Add multiple clauses before solving when possible
3. **Reuse solver instances**: Avoid creating new solvers for related problems
4. **Set appropriate timeout**: Use `timeout_seconds` to prevent infinite solving

## API Reference

### Core Types

- [`ParkissatSolver`](src/wrapper.rs): Main solver interface
- [`SolverOptions`](src/wrapper.rs): Configuration parameters
- [`SolverResult`](src/wrapper.rs): Solving results (Sat/Unsat/Unknown)
- [`SolverStatistics`](src/wrapper.rs): Solver performance statistics

### Key Methods

- `ParkissatSolver::new(num_threads)` - Create a new solver with specified thread count
- `configure(&options)` - Configure solver parameters
- `add_clause(&literals)` - Add a clause to the problem
- `solve()` - Solve the current problem
- `statistics()` - Get solver statistics
- `load_dimacs(path)` - Load problem from DIMACS file

## Requirements

- Rust 1.70 or later
- C++ compiler with C++17 support (GCC 7+, Clang 5+, or MSVC 2017+)
- GNU Make (for building ParKissat-RS components)
- OpenMP support (for parallel execution)
- zlib development libraries
- pthread support (on Unix systems)

### Build Dependencies

The build process automatically compiles the following components:
- **kissat_mab**: The core SAT solver library
- **painless-src**: The parallel solving framework
- **wrapper.cpp**: C++ bridge between Rust and ParKissat-RS

All dependencies are built automatically during `cargo build` using the included Makefiles.

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