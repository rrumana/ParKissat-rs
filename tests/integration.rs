//! Integration tests for ParKissat-RS bindings

use parkissat_sys::{ParkissatSolver, SolverConfig, SolverResult};

#[test]
fn test_basic_solver_functionality() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    // Add a simple satisfiable formula: (x1 ∨ x2) ∧ (¬x1 ∨ x2)
    solver.add_clause(&[1, 2]).expect("Failed to add clause");
    solver.add_clause(&[-1, 2]).expect("Failed to add clause");
    
    let result = solver.solve().expect("Failed to solve");
    assert_eq!(result, SolverResult::Sat);
    
    // x2 should be true to satisfy both clauses
    let x2_value = solver.get_model_value(2).expect("Failed to get model value");
    assert!(x2_value);
}

#[test]
fn test_unsatisfiable_formula() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    // Add contradictory clauses: x1 ∧ ¬x1
    solver.add_clause(&[1]).expect("Failed to add clause");
    solver.add_clause(&[-1]).expect("Failed to add clause");
    
    let result = solver.solve().expect("Failed to solve");
    assert_eq!(result, SolverResult::Unsat);
}

#[test]
fn test_solver_statistics() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    solver.add_clause(&[1, 2, 3]).expect("Failed to add clause");
    solver.add_clause(&[-1, -2]).expect("Failed to add clause");
    solver.add_clause(&[-1, -3]).expect("Failed to add clause");
    
    let _ = solver.solve();
    
    let stats = solver.get_statistics().expect("Failed to get statistics");
    
    // Should have some solving activity
    assert!(stats.decisions > 0 || stats.propagations > 0);
}

#[test]
fn test_multiple_solutions() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    // Add a formula with multiple solutions: x1 ∨ x2
    solver.add_clause(&[1, 2]).expect("Failed to add clause");
    
    let result = solver.solve().expect("Failed to solve");
    assert_eq!(result, SolverResult::Sat);
    
    let model = solver.get_model().expect("Failed to get model");
    assert!(!model.is_empty());
    
    // At least one of x1 or x2 should be true
    let x1_value = solver.get_model_value(1).unwrap_or(false);
    let x2_value = solver.get_model_value(2).unwrap_or(false);
    assert!(x1_value || x2_value);
}

#[test]
fn test_solver_with_assumptions() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    // Add clause: x1 ∨ x2
    solver.add_clause(&[1, 2]).expect("Failed to add clause");
    
    // Solve with assumption x1 = false
    let result = solver.solve_with_assumptions(&[-1]).expect("Failed to solve with assumptions");
    assert_eq!(result, SolverResult::Sat);
    
    // x2 must be true since x1 is assumed false
    let x2_value = solver.get_model_value(2).expect("Failed to get model value");
    assert!(x2_value);
}

#[test]
fn test_configuration_options() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig {
        num_threads: 2,
        timeout: std::time::Duration::from_secs(10),
        random_seed: 42,
        enable_preprocessing: true,
        verbosity: 1,
    };
    
    solver.configure(&config).expect("Failed to configure solver");
    
    assert!(solver.is_configured());
    
    // Add a simple clause and solve
    solver.add_clause(&[1]).expect("Failed to add clause");
    let result = solver.solve().expect("Failed to solve");
    assert_eq!(result, SolverResult::Sat);
}

#[test]
fn test_variable_count_tracking() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    assert_eq!(solver.variable_count(), 0);
    
    solver.add_clause(&[1, -5, 3]).expect("Failed to add clause");
    assert_eq!(solver.variable_count(), 5); // Highest variable is 5
    
    solver.add_clause(&[2, -7]).expect("Failed to add clause");
    assert_eq!(solver.variable_count(), 7); // Now highest is 7
}

#[test]
fn test_explicit_variable_count() {
    let mut solver = ParkissatSolver::new().expect("Failed to create solver");
    
    let config = SolverConfig::default();
    solver.configure(&config).expect("Failed to configure solver");
    
    solver.set_variable_count(10).expect("Failed to set variable count");
    assert_eq!(solver.variable_count(), 10);
    
    // Adding clauses with lower variables shouldn't decrease the count
    solver.add_clause(&[1, 2]).expect("Failed to add clause");
    assert_eq!(solver.variable_count(), 10);
}