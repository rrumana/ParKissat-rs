//! Safe Rust bindings for ParKissat-RS SAT solver
//! 
//! This crate provides safe Rust bindings for the ParKissat-RS parallel SAT solver,
//! which won the SAT Competition 2022. The bindings expose a safe, idiomatic Rust
//! API while maintaining high performance through minimal overhead FFI calls.

pub mod ffi;
pub mod wrapper;
pub mod error;

pub use wrapper::{ParkissatSolver, SolverConfig, SolverResult, SolverStatistics};
pub use error::{ParkissatError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_creation() {
        let solver = ParkissatSolver::new();
        assert!(solver.is_ok());
    }

    #[test]
    fn test_simple_satisfiable() {
        let mut solver = ParkissatSolver::new().unwrap();
        
        // Configure solver
        let config = SolverConfig::default();
        solver.configure(&config).unwrap();
        
        // Add clause: x1 ∨ x2
        solver.add_clause(&[1, 2]).unwrap();
        
        // Add clause: ¬x1 ∨ x2  
        solver.add_clause(&[-1, 2]).unwrap();
        
        let result = solver.solve().unwrap();
        assert_eq!(result, SolverResult::Sat);
        
        // x2 should be true to satisfy both clauses
        assert_eq!(solver.get_model_value(2).unwrap(), true);
    }

    #[test]
    fn test_unsatisfiable() {
        let mut solver = ParkissatSolver::new().unwrap();
        
        let config = SolverConfig::default();
        solver.configure(&config).unwrap();
        
        // Add contradictory clauses: x1 and ¬x1
        solver.add_clause(&[1]).unwrap();
        solver.add_clause(&[-1]).unwrap();
        
        let result = solver.solve().unwrap();
        assert_eq!(result, SolverResult::Unsat);
    }

    #[test]
    fn test_solver_statistics() {
        let mut solver = ParkissatSolver::new().unwrap();
        
        let config = SolverConfig::default();
        solver.configure(&config).unwrap();
        
        solver.add_clause(&[1, 2]).unwrap();
        let _ = solver.solve();
        
        let stats = solver.get_statistics().unwrap();
        // Should have some activity
        assert!(stats.decisions > 0 || stats.propagations > 0);
    }
}