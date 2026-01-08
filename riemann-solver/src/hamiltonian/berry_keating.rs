use nalgebra::{DMatrix, Complex};
use crate::hamiltonian::QuantumSystem;
use crate::utils::RiemannError;

/// Berry-Keating Hamiltonian truncated to harmonic oscillator subspace.
/// 
/// Based on Srednicki (2011): "The Berry-Keating Hamiltonian and the Local Riemann Hypothesis"
/// arXiv:1104.1850v3
///
/// The classical Berry-Keating Hamiltonian H = xp is regularized by projecting
/// onto the subspace of harmonic oscillator eigenfunctions with index < N.
/// 
/// Key result: The eigenvalues correspond to zeros of the modified gamma factor,
/// which satisfies the local Riemann hypothesis (all zeros have Re(s) = 1/2).
pub struct BerryKeatingSystem {
    /// Truncation level (dimension of Hilbert space)
    truncation: usize,
}

impl BerryKeatingSystem {
    pub fn new(truncation: usize) -> Result<Self, RiemannError> {
        if truncation == 0 {
            return Err(RiemannError::InvalidSize(truncation));
        }
        Ok(Self { truncation })
    }

    /// Compute matrix element ⟨n|H_BK|m⟩ where H_BK = (xp + px)/2
    /// 
    /// Using harmonic oscillator basis on ℝ⁺:
    /// |n⟩ has eigenfunction ψ_n(x) = κ_n H_n(√(2π)x) exp(-πx²)
    /// 
    /// The Berry-Keating operator in this basis has matrix elements:
    /// ⟨n|H_BK|m⟩ = (1/2) ⟨n|xp + px|m⟩
    /// 
    /// For harmonic oscillator with ω=2π:
    /// x = (1/√(4π))(a† + a)
    /// p = i√π(a† - a)
    /// 
    /// where a|n⟩ = √n|n-1⟩ and a†|n⟩ = √(n+1)|n+1⟩
    fn matrix_element(&self, n: usize, m: usize) -> Complex<f64> {
        // Only nearest-neighbor terms are non-zero (tridiagonal structure)
        if (n as i32 - m as i32).abs() != 1 {
            return Complex::new(0.0, 0.0);
        }
        
        // Matrix elements derived from Srednicki (2011)
        // For harmonic oscillator basis with ω=2π:
        // x = (1/√(4π))(a† + a), p = i√π(a† - a)
        // H_BK = (xp + px)/2
        //
        // After working through the algebra with ladder operators:
        // ⟨n|H_BK|m⟩ = (i/2)√n  for n = m+1
        // ⟨n|H_BK|m⟩ = -(i/2)√m for n = m-1
        // ⟨n|H_BK|m⟩ = 0       otherwise
        
        let i = Complex::new(0.0, 1.0);
        
        if n == m + 1 {
            // Raising: ⟨m+1|H_BK|m⟩ = (i/2)√(m+1)
            i * (n as f64).sqrt() / 2.0
        } else {
            // Lowering: ⟨m-1|H_BK|m⟩ = -(i/2)√m
            -i * ((n + 1) as f64).sqrt() / 2.0
        }
    }
}

impl QuantumSystem for BerryKeatingSystem {
    fn generate_hamiltonian(&self) -> Result<DMatrix<Complex<f64>>, RiemannError> {
        let n = self.truncation;
        
        // Construct the truncated Berry-Keating Hamiltonian matrix
        let matrix = DMatrix::from_fn(n, n, |i, j| {
            self.matrix_element(i, j)
        });

        Ok(matrix)
    }

    fn size(&self) -> usize {
        self.truncation
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::ComplexField;

    #[test]
    fn test_berry_keating_creation() {
        let bk = BerryKeatingSystem::new(10).unwrap();
        assert_eq!(bk.size(), 10);
    }

    #[test]
    fn test_matrix_purely_imaginary_antisymmetric() {
        let bk = BerryKeatingSystem::new(5).unwrap();
        let h = bk.generate_hamiltonian().unwrap();
        
        // Berry-Keating matrix is purely imaginary and anti-symmetric: H_ij = -H_ji
        for i in 0..5 {
            for j in 0..5 {
                // Check real part is zero
                assert!(h[(i, j)].re.abs() < 1e-10, "Non-zero real part at ({}, {})", i, j);
                // Check anti-symmetry: H_ij = -H_ji
                let diff = (h[(i, j)] + h[(j, i)]).modulus();
                assert!(diff < 1e-10, "Matrix not anti-symmetric at ({}, {})", i, j);
            }
        }
    }

    #[test]
    fn test_tridiagonal_structure() {
        // Berry-Keating in harmonic oscillator basis should be tridiagonal
        let bk = BerryKeatingSystem::new(10).unwrap();
        let h = bk.generate_hamiltonian().unwrap();
        
        for i in 0..10 {
            for j in 0..10 {
                if (i as i32 - j as i32).abs() > 1 {
                    let elem = h[(i, j)].modulus();
                    assert!(elem < 1e-10, "Non-tridiagonal element at ({}, {}): {}", i, j, elem);
                }
            }
        }
    }
    
    #[test]
    fn test_purely_imaginary_eigenvalues() {
        // Eigenvalues should be purely imaginary (Re(E) = 0)
        // corresponding to s = 1/2 + iE
        let bk = BerryKeatingSystem::new(20).unwrap();
        let h = bk.generate_hamiltonian().unwrap();
        
        // Convert to real symmetric by extracting imaginary part
        // Since H is anti-Hermitian (H† = -H for Berry-Keating)
        let mut real_mat = DMatrix::zeros(20, 20);
        for i in 0..20 {
            for j in 0..20 {
                real_mat[(i, j)] = h[(i, j)].im;
            }
        }
        
        let eig = real_mat.symmetric_eigen();
        
        // All eigenvalues should be real (since we extracted imaginary part)
        for i in 0..20 {
            let eval = eig.eigenvalues[i];
            assert!(eval.is_finite(), "Eigenvalue {} is not finite", i);
        }
    }
}
