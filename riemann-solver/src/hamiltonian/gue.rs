use nalgebra::{DMatrix, Complex};
use rand::thread_rng;
use rand_distr::{Normal, Distribution};
use crate::hamiltonian::QuantumSystem;
use crate::utils::RiemannError;

/// Gaussian Unitary Ensemble (GUE) random matrix system.
pub struct GueSystem {
    size: usize,
    seed: Option<u64>,
}

impl GueSystem {
    pub fn new(size: usize, seed: Option<u64>) -> Result<Self, RiemannError> {
        if size == 0 {
            return Err(RiemannError::InvalidSize(size));
        }
        Ok(Self { size, seed })
    }
}

impl QuantumSystem for GueSystem {
    fn generate_hamiltonian(&self) -> Result<DMatrix<Complex<f64>>, RiemannError> {
        let n = self.size;
        let mut rng = thread_rng();
        let normal = Normal::new(0.0, 1.0 / (2.0 * n as f64).sqrt())
            .map_err(|e| RiemannError::DiagonalizationError(e.to_string()))?;

        // Generate random complex Hermitian matrix
        let mut matrix = DMatrix::from_fn(n, n, |i, j| {
            if i == j {
                // Diagonal: real values
                Complex::new(normal.sample(&mut rng), 0.0)
            } else if i < j {
                // Upper triangle: complex values
                Complex::new(normal.sample(&mut rng), normal.sample(&mut rng))
            } else {
                // Lower triangle: conjugate of upper
                Complex::new(0.0, 0.0) // Will be filled below
            }
        });

        // Make Hermitian: H_ij = conj(H_ji)
        for i in 0..n {
            for j in 0..i {
                matrix[(i, j)] = matrix[(j, i)].conj();
            }
        }

        Ok(matrix)
    }

    fn size(&self) -> usize {
        self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gue_creation() {
        let gue = GueSystem::new(10, None).unwrap();
        assert_eq!(gue.size(), 10);
    }

    #[test]
    fn test_hermiticity() {
        let gue = GueSystem::new(5, Some(42)).unwrap();
        let h = gue.generate_hamiltonian().unwrap();
        
        // Check Hermiticity: H = H†
        for i in 0..5 {
            for j in 0..5 {
                let diff = (h[(i, j)] - h[(j, i)].conj()).modulus();
                assert!(diff < 1e-10, "Matrix not Hermitian at ({}, {})", i, j);
            }
        }
    }
}
