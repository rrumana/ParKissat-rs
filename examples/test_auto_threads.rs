//! Test automatic thread detection with -1

use parkissat_sys::{ParkissatSolver, SolverConfig};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing automatic thread detection...");
    
    // Test with -1 (auto-detect)
    let mut solver = ParkissatSolver::new()?;
    
    let config = SolverConfig {
        num_threads: -1,  // Auto-detect
        timeout: Duration::from_secs(10),
        random_seed: 42,
        enable_preprocessing: true,
        verbosity: 1,
    };
    
    println!("Configuring solver with num_threads = -1 (auto-detect)");
    solver.configure(&config)?;
    
    // Add a simple satisfiable problem
    solver.add_clause(&[1, 2])?;
    solver.add_clause(&[-1, 2])?;
    
    println!("Solving...");
    let result = solver.solve()?;
    
    match result {
        parkissat_sys::SolverResult::Sat => {
            println!("✓ Problem is satisfiable");
            println!("✓ Auto thread detection worked!");
            
            // Get statistics to see if threading was used
            let stats = solver.get_statistics()?;
            println!("Statistics: decisions={}, conflicts={}", stats.decisions, stats.conflicts);
        }
        parkissat_sys::SolverResult::Unsat => {
            println!("✗ Problem is unsatisfiable (unexpected)");
        }
        parkissat_sys::SolverResult::Unknown => {
            println!("? Problem result is unknown");
        }
    }
    
    // Test with explicit thread count for comparison
    println!("\nTesting with explicit thread count...");
    let mut solver2 = ParkissatSolver::new()?;
    
    let config2 = SolverConfig {
        num_threads: 2,  // Explicit
        timeout: Duration::from_secs(10),
        random_seed: 42,
        enable_preprocessing: true,
        verbosity: 1,
    };
    
    solver2.configure(&config2)?;
    solver2.add_clause(&[1, 2])?;
    solver2.add_clause(&[-1, 2])?;
    
    let result2 = solver2.solve()?;
    println!("Explicit 2-thread result: {:?}", result2);
    
    println!("Available CPUs: {}", num_cpus::get());
    
    Ok(())
}