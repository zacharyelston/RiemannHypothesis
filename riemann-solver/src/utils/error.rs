use thiserror::Error;

/// A specific error type for the solver.
#[derive(Error, Debug)]
pub enum RiemannError {
    #[error("Invalid matrix size: {0}")]
    InvalidSize(usize),
    #[error("LAPACK diagonalization failed: {0}")]
    DiagonalizationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
