//! Error types for ParKissat-RS bindings

use std::fmt;

/// Result type for ParKissat operations
pub type Result<T> = std::result::Result<T, ParkissatError>;

/// Error types that can occur when using ParKissat-RS
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParkissatError {
    /// Solver creation failed
    SolverCreationFailed,
    
    /// Invalid configuration parameter
    InvalidConfiguration(String),
    
    /// Invalid clause (e.g., empty clause)
    InvalidClause(String),
    
    /// Invalid variable number
    InvalidVariable(i32),
    
    /// Solver is not configured
    NotConfigured,
    
    /// No solution available (solver hasn't been run or returned UNSAT/UNKNOWN)
    NoSolution,
    
    /// File I/O error
    IoError(String),
    
    /// Solver was interrupted
    Interrupted,
    
    /// Internal solver error
    InternalError(String),
}

impl fmt::Display for ParkissatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParkissatError::SolverCreationFailed => {
                write!(f, "Failed to create ParKissat solver instance")
            }
            ParkissatError::InvalidConfiguration(msg) => {
                write!(f, "Invalid solver configuration: {}", msg)
            }
            ParkissatError::InvalidClause(msg) => {
                write!(f, "Invalid clause: {}", msg)
            }
            ParkissatError::InvalidVariable(var) => {
                write!(f, "Invalid variable number: {}", var)
            }
            ParkissatError::NotConfigured => {
                write!(f, "Solver is not configured")
            }
            ParkissatError::NoSolution => {
                write!(f, "No solution available")
            }
            ParkissatError::IoError(msg) => {
                write!(f, "I/O error: {}", msg)
            }
            ParkissatError::Interrupted => {
                write!(f, "Solver was interrupted")
            }
            ParkissatError::InternalError(msg) => {
                write!(f, "Internal solver error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ParkissatError {}

impl From<std::io::Error> for ParkissatError {
    fn from(err: std::io::Error) -> Self {
        ParkissatError::IoError(err.to_string())
    }
}

impl From<std::ffi::NulError> for ParkissatError {
    fn from(err: std::ffi::NulError) -> Self {
        ParkissatError::InvalidConfiguration(format!("String contains null byte: {}", err))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ParkissatError::SolverCreationFailed;
        assert_eq!(err.to_string(), "Failed to create ParKissat solver instance");

        let err = ParkissatError::InvalidVariable(42);
        assert_eq!(err.to_string(), "Invalid variable number: 42");

        let err = ParkissatError::InvalidClause("empty clause".to_string());
        assert_eq!(err.to_string(), "Invalid clause: empty clause");
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let parkissat_err: ParkissatError = io_err.into();
        
        match parkissat_err {
            ParkissatError::IoError(msg) => assert!(msg.contains("file not found")),
            _ => panic!("Expected IoError"),
        }
    }
}