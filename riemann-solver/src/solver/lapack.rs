use nalgebra::{DMatrix, Complex};
use crate::solver::EigenSolver;
use crate::utils::RiemannError;

/// LAPACK-based eigenvalue solver using nalgebra's symmetric_eigen.
pub struct LapackSolver;

impl LapackSolver {
    pub fn new() -> Self {
        Self
    }
}

impl EigenSolver for LapackSolver {
    fn solve(&self, matrix: &DMatrix<Complex<f64>>) -> Result<Vec<f64>, RiemannError> {
        // For complex Hermitian matrices, we need to convert to real symmetric
        // by representing as a 2N x 2N real matrix
        let n = matrix.nrows();
        
        // Extract real and imaginary parts
        let mut big_mat = DMatrix::zeros(2 * n, 2 * n);
        for i in 0..n {
            for j in 0..n {
                let c = matrix[(i, j)];
                big_mat[(i, j)] = c.re;
                big_mat[(i + n, j + n)] = c.re;
                big_mat[(i + n, j)] = c.im;
                big_mat[(i, j + n)] = -c.im;
            }
        }
        
        // Compute eigenvalues
        let eig = big_mat.symmetric_eigen();
        let mut evs = eig.eigenvalues.as_slice().to_vec();
        
        // Sort and take every second eigenvalue (they come in pairs)
        evs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let distinct_evs: Vec<f64> = evs.iter().step_by(2).cloned().collect();
        
        Ok(distinct_evs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solver_basic() {
        let solver = LapackSolver::new();
        
        // Create a simple 2x2 Hermitian matrix
        let mut matrix = DMatrix::zeros(2, 2);
        matrix[(0, 0)] = Complex::new(1.0, 0.0);
        matrix[(1, 1)] = Complex::new(2.0, 0.0);
        matrix[(0, 1)] = Complex::new(0.5, 0.5);
        matrix[(1, 0)] = Complex::new(0.5, -0.5);
        
        let eigenvalues = solver.solve(&matrix).unwrap();
        assert_eq!(eigenvalues.len(), 2);
    }
}
