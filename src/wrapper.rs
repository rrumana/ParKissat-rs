//! Safe Rust wrapper for ParKissat-RS SAT solver

use crate::ffi;
use crate::error::{ParkissatError, Result};
use std::ffi::CString;
use std::os::raw::c_int;
use std::ptr;
use std::time::Duration;

/// Configuration for the ParKissat solver
#[derive(Debug, Clone)]
pub struct SolverConfig {
    /// Number of parallel threads to use (default: 1, -1 = use all available CPUs)
    pub num_threads: isize,
    
    /// Timeout in seconds (0 = no timeout)
    pub timeout: Duration,
    
    /// Random seed for diversification (0 = use default)
    pub random_seed: u32,
    
    /// Enable preprocessing
    pub enable_preprocessing: bool,
    
    /// Verbosity level (0 = quiet)
    pub verbosity: u32,
}

impl Default for SolverConfig {
    fn default() -> Self {
        Self {
            num_threads: 1,
            timeout: Duration::from_secs(0),
            random_seed: 0,
            enable_preprocessing: false,
            verbosity: 0,
        }
    }
}

/// Result of SAT solving
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SolverResult {
    /// Formula is satisfiable
    Sat,
    /// Formula is unsatisfiable
    Unsat,
    /// Result is unknown (timeout, interrupted, etc.)
    Unknown,
}

impl From<ffi::ParkissatResult> for SolverResult {
    fn from(result: ffi::ParkissatResult) -> Self {
        match result {
            ffi::PARKISSAT_SAT => SolverResult::Sat,
            ffi::PARKISSAT_UNSAT => SolverResult::Unsat,
            ffi::PARKISSAT_UNKNOWN => SolverResult::Unknown,
            _ => SolverResult::Unknown,
        }
    }
}

/// Solver statistics
#[derive(Debug, Clone)]
pub struct SolverStatistics {
    /// Number of propagations
    pub propagations: u64,
    /// Number of decisions
    pub decisions: u64,
    /// Number of conflicts
    pub conflicts: u64,
    /// Number of restarts
    pub restarts: u64,
    /// Peak memory usage in KB
    pub memory_peak_kb: f64,
}

impl From<ffi::ParkissatStatistics> for SolverStatistics {
    fn from(stats: ffi::ParkissatStatistics) -> Self {
        Self {
            propagations: stats.propagations,
            decisions: stats.decisions,
            conflicts: stats.conflicts,
            restarts: stats.restarts,
            memory_peak_kb: stats.mem_peak,
        }
    }
}

/// Safe wrapper for ParKissat-RS SAT solver
pub struct ParkissatSolver {
    solver: *mut ffi::ParkissatSolver,
    configured: bool,
    last_result: Option<SolverResult>,
    variable_count: usize,
}

impl ParkissatSolver {
    /// Create a new solver instance
    pub fn new() -> Result<Self> {
        let solver = unsafe { ffi::parkissat_new() };
        
        if solver.is_null() {
            return Err(ParkissatError::SolverCreationFailed);
        }
        
        Ok(Self {
            solver,
            configured: false,
            last_result: None,
            variable_count: 0,
        })
    }
    
    /// Configure the solver with the given options
    pub fn configure(&mut self, config: &SolverConfig) -> Result<()> {
        if self.solver.is_null() {
            return Err(ParkissatError::SolverCreationFailed);
        }
        
        // Resolve thread count: -1 means use all available CPUs
        let actual_threads = if config.num_threads == -1 {
            num_cpus::get()
        } else if config.num_threads <= 0 {
            return Err(ParkissatError::InvalidConfiguration(
                "Number of threads must be positive or -1 for auto-detection".to_string()
            ));
        } else {
            config.num_threads as usize
        };
        
        let ffi_config = ffi::ParkissatConfig {
            num_threads: actual_threads as c_int,
            timeout_seconds: config.timeout.as_secs() as c_int,
            random_seed: config.random_seed as c_int,
            enable_preprocessing: config.enable_preprocessing,
            verbosity: config.verbosity as c_int,
        };
        
        unsafe {
            ffi::parkissat_configure(self.solver, &ffi_config);
        }
        
        self.configured = true;
        Ok(())
    }
    
    /// Load a DIMACS file
    pub fn load_dimacs<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<()> {
        if !self.configured {
            return Err(ParkissatError::NotConfigured);
        }
        
        let path_str = path.as_ref().to_str()
            .ok_or_else(|| ParkissatError::IoError("Invalid path".to_string()))?;
        
        let c_path = CString::new(path_str)?;
        
        let success = unsafe {
            ffi::parkissat_load_dimacs(self.solver, c_path.as_ptr())
        };
        
        if !success {
            return Err(ParkissatError::IoError(format!("Failed to load DIMACS file: {}", path_str)));
        }
        
        Ok(())
    }
    
    /// Add a clause to the solver
    /// 
    /// # Arguments
    /// * `literals` - Array of literals (positive for variable, negative for negation)
    pub fn add_clause(&mut self, literals: &[i32]) -> Result<()> {
        if !self.configured {
            return Err(ParkissatError::NotConfigured);
        }
        
        if literals.is_empty() {
            return Err(ParkissatError::InvalidClause("Empty clause".to_string()));
        }
        
        // Validate literals
        for &lit in literals {
            if lit == 0 {
                return Err(ParkissatError::InvalidClause("Literal cannot be zero".to_string()));
            }
        }
        
        // Update variable count
        for &lit in literals {
            let var = lit.abs() as usize;
            if var > self.variable_count {
                self.variable_count = var;
            }
        }
        
        unsafe {
            ffi::parkissat_add_clause(
                self.solver,
                literals.as_ptr(),
                literals.len() as c_int
            );
        }
        
        Ok(())
    }
    
    /// Set the number of variables explicitly
    pub fn set_variable_count(&mut self, count: usize) -> Result<()> {
        if !self.configured {
            return Err(ParkissatError::NotConfigured);
        }
        
        if count == 0 {
            return Err(ParkissatError::InvalidConfiguration(
                "Variable count must be positive".to_string()
            ));
        }
        
        self.variable_count = count;
        unsafe {
            ffi::parkissat_set_variable_count(self.solver, count as c_int);
        }
        
        Ok(())
    }
    
    /// Solve the SAT problem
    pub fn solve(&mut self) -> Result<SolverResult> {
        if !self.configured {
            return Err(ParkissatError::NotConfigured);
        }
        
        let result = unsafe {
            ffi::parkissat_solve(self.solver)
        };
        
        let solver_result = SolverResult::from(result);
        self.last_result = Some(solver_result);
        
        Ok(solver_result)
    }
    
    /// Solve with assumptions
    pub fn solve_with_assumptions(&mut self, assumptions: &[i32]) -> Result<SolverResult> {
        if !self.configured {
            return Err(ParkissatError::NotConfigured);
        }
        
        // Validate assumptions
        for &lit in assumptions {
            if lit == 0 {
                return Err(ParkissatError::InvalidClause("Assumption cannot be zero".to_string()));
            }
        }
        
        let result = unsafe {
            ffi::parkissat_solve_with_assumptions(
                self.solver,
                assumptions.as_ptr(),
                assumptions.len() as c_int
            )
        };
        
        let solver_result = SolverResult::from(result);
        self.last_result = Some(solver_result);
        
        Ok(solver_result)
    }
    
    /// Get the truth value of a variable in the model (only valid after SAT result)
    pub fn get_model_value(&self, variable: i32) -> Result<bool> {
        if variable <= 0 {
            return Err(ParkissatError::InvalidVariable(variable));
        }
        
        match self.last_result {
            Some(SolverResult::Sat) => {
                let value = unsafe {
                    ffi::parkissat_get_model_value(self.solver, variable)
                };
                Ok(value)
            }
            Some(SolverResult::Unsat) | Some(SolverResult::Unknown) => {
                Err(ParkissatError::NoSolution)
            }
            None => Err(ParkissatError::NoSolution),
        }
    }
    
    /// Get the complete model (only valid after SAT result)
    pub fn get_model(&self) -> Result<Vec<i32>> {
        match self.last_result {
            Some(SolverResult::Sat) => {
                let size = unsafe {
                    ffi::parkissat_get_model_size(self.solver)
                };
                
                if size <= 0 {
                    return Ok(Vec::new());
                }
                
                let mut model = vec![0; size as usize];
                unsafe {
                    ffi::parkissat_get_model(self.solver, model.as_mut_ptr(), size);
                }
                
                Ok(model)
            }
            Some(SolverResult::Unsat) | Some(SolverResult::Unknown) => {
                Err(ParkissatError::NoSolution)
            }
            None => Err(ParkissatError::NoSolution),
        }
    }
    
    /// Get solver statistics
    pub fn get_statistics(&self) -> Result<SolverStatistics> {
        if !self.configured {
            return Err(ParkissatError::NotConfigured);
        }
        
        let stats = unsafe {
            ffi::parkissat_get_statistics(self.solver)
        };
        
        Ok(SolverStatistics::from(stats))
    }
    
    /// Interrupt the solver
    pub fn interrupt(&mut self) {
        if !self.solver.is_null() {
            unsafe {
                ffi::parkissat_interrupt(self.solver);
            }
        }
    }
    
    /// Clear the interrupt flag
    pub fn clear_interrupt(&mut self) {
        if !self.solver.is_null() {
            unsafe {
                ffi::parkissat_clear_interrupt(self.solver);
            }
        }
    }
    
    /// Get the number of variables
    pub fn variable_count(&self) -> usize {
        self.variable_count
    }
    
    /// Check if the solver is configured
    pub fn is_configured(&self) -> bool {
        self.configured
    }
    
    /// Get the last solve result
    pub fn last_result(&self) -> Option<SolverResult> {
        self.last_result
    }
}

impl Drop for ParkissatSolver {
    fn drop(&mut self) {
        if !self.solver.is_null() {
            unsafe {
                ffi::parkissat_delete(self.solver);
            }
            self.solver = ptr::null_mut();
        }
    }
}

// Note: ParkissatSolver is not Send/Sync due to the raw pointer to C++ object
// This is automatically handled by Rust's type system since raw pointers are !Send + !Sync

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_config_default() {
        let config = SolverConfig::default();
        assert_eq!(config.num_threads, 1);
        assert_eq!(config.timeout, Duration::from_secs(0));
        assert_eq!(config.random_seed, 0);
        assert!(!config.enable_preprocessing);
        assert_eq!(config.verbosity, 0);
    }

    #[test]
    fn test_solver_result_conversion() {
        assert_eq!(SolverResult::from(ffi::PARKISSAT_SAT), SolverResult::Sat);
        assert_eq!(SolverResult::from(ffi::PARKISSAT_UNSAT), SolverResult::Unsat);
        assert_eq!(SolverResult::from(ffi::PARKISSAT_UNKNOWN), SolverResult::Unknown);
    }

    #[test]
    fn test_invalid_configuration() {
        let mut solver = ParkissatSolver::new().unwrap();
        
        let mut config = SolverConfig::default();
        config.num_threads = 0;
        
        let result = solver.configure(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParkissatError::InvalidConfiguration(_)));
    }

    #[test]
    fn test_not_configured_error() {
        let mut solver = ParkissatSolver::new().unwrap();
        
        // Try to add clause without configuration
        let result = solver.add_clause(&[1, 2]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), ParkissatError::NotConfigured);
    }

    #[test]
    fn test_empty_clause_error() {
        let mut solver = ParkissatSolver::new().unwrap();
        let config = SolverConfig::default();
        solver.configure(&config).unwrap();
        
        let result = solver.add_clause(&[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParkissatError::InvalidClause(_)));
    }

    #[test]
    fn test_zero_literal_error() {
        let mut solver = ParkissatSolver::new().unwrap();
        let config = SolverConfig::default();
        solver.configure(&config).unwrap();
        
        let result = solver.add_clause(&[1, 0, 2]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParkissatError::InvalidClause(_)));
    }
}