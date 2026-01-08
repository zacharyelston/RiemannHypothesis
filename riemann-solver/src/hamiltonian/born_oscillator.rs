use nalgebra::{DMatrix, Complex};
use crate::hamiltonian::QuantumSystem;
use crate::utils::RiemannError;

/// Born Oscillator - a regularization of Berry-Keating Hamiltonian
/// 
/// Based on Giordano, Negro, Tateo (2023): "The Generalized Born Oscillator 
/// and the Berry-Keating Hamiltonian" arXiv:2307.15025v2
///
/// Classical Hamiltonian: H = √(1 + λp²) √(1 + λq²)
/// 
/// Key properties:
/// - Time-reversal symmetric
/// - Closed classical trajectories (no cutoff needed)
/// - Reduces to H ≈ 1 + λpq/2 as λ→0 (Berry-Keating limit)
/// - Counting function reproduces N̄(T) for Riemann zeros
///
/// This is a dimensional reduction of Nambu-Goto theory with T̄T deformation.
pub struct BornOscillator {
    /// Deformation parameter λ
    lambda: f64,
    /// Truncation level for Weyl quantization
    truncation: usize,
}

impl BornOscillator {
    pub fn new(lambda: f64, truncation: usize) -> Result<Self, RiemannError> {
        if lambda <= 0.0 {
            return Err(RiemannError::InvalidSize(0));
        }
        if truncation == 0 {
            return Err(RiemannError::InvalidSize(truncation));
        }
        Ok(Self { lambda, truncation })
    }
    
    /// Classical Hamiltonian H(p,q) = √(1 + λp²) √(1 + λq²)
    pub fn classical_hamiltonian(&self, p: f64, q: f64) -> f64 {
        (1.0 + self.lambda * p * p).sqrt() * (1.0 + self.lambda * q * q).sqrt()
    }
    
    /// Classical trajectory: points on the curve H(p,q) = E
    pub fn classical_trajectory(&self, energy: f64, num_points: usize) -> Vec<(f64, f64)> {
        let mut points = Vec::with_capacity(num_points);
        
        // Parametric representation of the trajectory
        // For H = √(1+λp²)√(1+λq²) = E, we can solve for p(q)
        for i in 0..num_points {
            let theta = 2.0 * std::f64::consts::PI * (i as f64) / (num_points as f64);
            
            // Approximate parametrization (exact form is complex)
            // For small λ, trajectory is approximately hyperbolic
            let q = (energy / self.lambda.sqrt()) * theta.cos();
            let p_squared = (energy * energy / (1.0 + self.lambda * q * q) - 1.0) / self.lambda;
            
            if p_squared >= 0.0 {
                let p = p_squared.sqrt() * theta.sin().signum();
                points.push((p, q));
            }
        }
        
        points
    }
}

impl QuantumSystem for BornOscillator {
    fn generate_hamiltonian(&self) -> Result<DMatrix<Complex<f64>>, RiemannError> {
        // For Born oscillator, we need Weyl quantization
        // This is more complex than simple matrix elements
        // 
        // The paper uses an iterative procedure (Appendix C) to compute
        // the quantization condition to high orders in ℏ
        //
        // For now, implement a placeholder that will be filled with
        // the Weyl quantization scheme
        
        tracing::warn!("Born oscillator Weyl quantization not yet fully implemented");
        tracing::info!("This requires implementing the iterative procedure from Appendix C");
        
        // Placeholder: return a simple matrix for now
        // TODO: Implement full Weyl quantization
        let n = self.truncation;
        let matrix = DMatrix::from_fn(n, n, |i, j| {
            if i == j {
                // Diagonal approximation based on classical energy levels
                let level = i as f64;
                Complex::new(level, 0.0)
            } else {
                Complex::new(0.0, 0.0)
            }
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

    #[test]
    fn test_born_oscillator_creation() {
        let bo = BornOscillator::new(1.0, 10).unwrap();
        assert_eq!(bo.size(), 10);
    }

    #[test]
    fn test_classical_hamiltonian() {
        let bo = BornOscillator::new(1.0, 10).unwrap();
        
        // At p=0, q=0: H = √1 · √1 = 1
        let h = bo.classical_hamiltonian(0.0, 0.0);
        assert!((h - 1.0).abs() < 1e-10);
        
        // At p=1, q=0: H = √2 · √1 = √2
        let h = bo.classical_hamiltonian(1.0, 0.0);
        assert!((h - 2.0_f64.sqrt()).abs() < 1e-10);
    }
    
    #[test]
    fn test_berry_keating_limit() {
        // As λ→0, H ≈ 1 + λpq/2
        let lambda = 0.01;
        let bo = BornOscillator::new(lambda, 10).unwrap();
        
        let p = 2.0;
        let q = 3.0;
        
        let h_exact = bo.classical_hamiltonian(p, q);
        let h_approx = 1.0 + lambda * p * q / 2.0;
        
        // Should be close for small λ
        assert!((h_exact - h_approx).abs() < 0.1);
    }
}
