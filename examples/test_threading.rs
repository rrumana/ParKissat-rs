use parkissat_sys::{ParkissatSolver, SolverConfig, SolverResult};

fn main() {
    println!("Testing threading functionality...");
    
    // Test with different thread counts
    for num_threads in [1, 2, 4, 8] {
        println!("\n=== Testing with {} threads ===", num_threads);
        
        let mut solver = ParkissatSolver::new().expect("Failed to create solver");
        
        let mut config = SolverConfig::default();
        config.num_threads = num_threads;
        solver.configure(&config).expect("Failed to configure solver");
        
        // Add a moderately complex formula
        // (x1 ∨ x2 ∨ x3) ∧ (¬x1 ∨ x4) ∧ (¬x2 ∨ x5) ∧ (¬x3 ∨ x6) ∧ (¬x4 ∨ ¬x5 ∨ ¬x6)
        solver.add_clause(&[1, 2, 3]).expect("Failed to add clause");
        solver.add_clause(&[-1, 4]).expect("Failed to add clause");
        solver.add_clause(&[-2, 5]).expect("Failed to add clause");
        solver.add_clause(&[-3, 6]).expect("Failed to add clause");
        solver.add_clause(&[-4, -5, -6]).expect("Failed to add clause");
        
        let start = std::time::Instant::now();
        let result = solver.solve().expect("Failed to solve");
        let duration = start.elapsed();
        
        println!("Result: {:?}, Time: {:?}", result, duration);
        
        if result == SolverResult::Sat {
            println!("Model found!");
        }
    }
}