use nalgebra::{DMatrix, Complex};
use crate::utils::RiemannError;

pub mod lapack;

/// Trait for eigenvalue solvers.
pub trait EigenSolver {
    /// Computes eigenvalues of a Hermitian matrix.
    fn solve(&self, matrix: &DMatrix<Complex<f64>>) -> Result<Vec<f64>, RiemannError>;
}
