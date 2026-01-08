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
    
    /// Compute first quantum correction Σ₁(E)
    /// 
    /// Simplified formula from Weyl quantization:
    /// Σ₁(E) ≈ (1/24π) ∫₀^{q_t} dq [∂²H/∂p² + ∂²H/∂q²]
    /// 
    /// For Born oscillator H = √(1+λp²)√(1+λq²):
    /// ∂²H/∂p² = λ√(1+λq²) / (1+λp²)^(3/2)
    /// ∂²H/∂q² = λ√(1+λp²) / (1+λq²)^(3/2)
    fn sigma_1(&self, energy: f64) -> f64 {
        if energy <= 1.0 {
            return 0.0;
        }
        
        let q_turning = ((energy * energy - 1.0) / self.lambda).sqrt();
        
        // Numerical integration
        let n_points = 1000;
        let dq = q_turning / (n_points as f64);
        let mut integral = 0.0;
        
        for i in 0..=n_points {
            let q = (i as f64) * dq;
            let weight = if i == 0 || i == n_points {
                1.0
            } else if i % 2 == 0 {
                2.0
            } else {
                4.0
            };
            
            // Momentum on trajectory
            let denom_q = 1.0 + self.lambda * q * q;
            let p_squared = (energy * energy / denom_q - 1.0) / self.lambda;
            
            if p_squared >= 0.0 {
                let p = p_squared.sqrt();
                let denom_p = 1.0 + self.lambda * p * p;
                
                // Second derivatives
                let d2h_dp2 = self.lambda * denom_q.sqrt() / denom_p.powf(1.5);
                let d2h_dq2 = self.lambda * denom_p.sqrt() / denom_q.powf(1.5);
                
                let integrand = d2h_dp2 + d2h_dq2;
                integral += weight * integrand;
            }
        }
        
        integral *= dq / 3.0; // Simpson's rule
        
        // Σ₁ = (1/24π) × integral
        integral / (24.0 * std::f64::consts::PI)
    }
    
    /// Compute semiclassical phase space volume Σ₀(E)
    /// 
    /// Σ₀(E) = (1/2π) ∫∫ dpdq Θ(E - H(p,q))
    ///       = (2/π) ∫₀^{q_t(E)} dq [p(E,q) - p(0,q)]
    /// 
    /// For Born oscillator: p(E,q) = (1/√λ)√[(E²/(1+λq²)) - 1]
    fn sigma_0(&self, energy: f64) -> f64 {
        if energy <= 1.0 {
            return 0.0; // Below ground state
        }
        
        // Turning point: q_t where H(0,q_t) = E
        // √1 · √(1+λq_t²) = E  =>  q_t = √((E²-1)/λ)
        let q_turning = ((energy * energy - 1.0) / self.lambda).sqrt();
        
        // Numerical integration using Simpson's rule
        let n_points = 1000;
        let dq = q_turning / (n_points as f64);
        let mut integral = 0.0;
        
        for i in 0..=n_points {
            let q = (i as f64) * dq;
            let weight = if i == 0 || i == n_points {
                1.0
            } else if i % 2 == 0 {
                2.0
            } else {
                4.0
            };
            
            // p(E,q) = (1/√λ)√[(E²/(1+λq²)) - 1]
            let denom = 1.0 + self.lambda * q * q;
            let p_squared = (energy * energy / denom - 1.0) / self.lambda;
            
            if p_squared >= 0.0 {
                let p_e = p_squared.sqrt();
                // p(0,q) = 0 (ground state)
                let integrand = p_e;
                integral += weight * integrand;
            }
        }
        
        integral *= dq / 3.0; // Simpson's rule factor
        
        // Σ₀ = (2/π) × integral
        (2.0 / std::f64::consts::PI) * integral
    }
    
    /// Solve quantization condition with quantum corrections
    /// 
    /// n + 1/2 = Σ₀(E)/ℏ + Σ₁(E)ℏ + ...
    /// 
    /// order = 0: Semiclassical (Σ₀ only)
    /// order = 1: First quantum correction (Σ₀ + Σ₁)
    fn solve_quantization_order(&self, n: usize, hbar: f64, order: usize) -> Result<f64, RiemannError> {
        let target = n as f64 + 0.5;
        
        // Initial guess: use harmonic oscillator-like spacing
        let mut energy = 1.0 + (n as f64) * 0.5;
        
        // Newton's method
        for _iter in 0..100 {
            let sigma0 = self.sigma_0(energy);
            let sigma1 = if order >= 1 { self.sigma_1(energy) } else { 0.0 };
            
            // RHS = Σ₀(E)/ℏ + Σ₁(E)ℏ
            let rhs = sigma0 / hbar + sigma1 * hbar;
            let residual = rhs - target;
            
            if residual.abs() < 1e-10 {
                return Ok(energy);
            }
            
            // Numerical derivative of RHS
            let deps = 1e-6;
            let sigma0_plus = self.sigma_0(energy + deps);
            let sigma1_plus = if order >= 1 { self.sigma_1(energy + deps) } else { 0.0 };
            let rhs_plus = sigma0_plus / hbar + sigma1_plus * hbar;
            let derivative = (rhs_plus - rhs) / deps;
            
            if derivative.abs() < 1e-12 {
                return Err(RiemannError::DiagonalizationError(
                    "Newton's method failed: derivative too small".to_string()
                ));
            }
            
            energy -= residual / derivative;
            
            // Keep energy positive
            if energy < 1.0 {
                energy = 1.0 + 1e-6;
            }
        }
        
        Err(RiemannError::DiagonalizationError(
            "Newton's method did not converge".to_string()
        ))
    }
    
    /// Solve semiclassical quantization (backward compatible)
    fn solve_quantization(&self, n: usize, hbar: f64) -> Result<f64, RiemannError> {
        self.solve_quantization_order(n, hbar, 0)
    }
    
    /// Compute eigenvalues using WKB quantization
    pub fn compute_eigenvalues_wkb(&self, hbar: f64) -> Result<Vec<f64>, RiemannError> {
        self.compute_eigenvalues_with_order(hbar, 0)
    }
    
    /// Compute eigenvalues with quantum corrections
    /// 
    /// order = 0: Semiclassical (Σ₀ only)
    /// order = 1: First quantum correction (Σ₀ + Σ₁)
    pub fn compute_eigenvalues_with_order(&self, hbar: f64, order: usize) -> Result<Vec<f64>, RiemannError> {
        let mut eigenvalues = Vec::with_capacity(self.truncation);
        
        for n in 0..self.truncation {
            let energy = self.solve_quantization_order(n, hbar, order)?;
            eigenvalues.push(energy);
        }
        
        Ok(eigenvalues)
    }
}

impl QuantumSystem for BornOscillator {
    fn generate_hamiltonian(&self) -> Result<DMatrix<Complex<f64>>, RiemannError> {
        // For Born oscillator, we use semiclassical (WKB) quantization
        // rather than direct matrix diagonalization
        //
        // The eigenvalues are computed from the quantization condition:
        // n + 1/2 = Σ₀(E)/ℏ
        //
        // This is a diagonal representation in the energy eigenbasis
        
        tracing::info!("Computing Born oscillator eigenvalues via WKB quantization");
        
        // Use ℏ = 1 for natural units
        let hbar = 1.0;
        let eigenvalues = self.compute_eigenvalues_wkb(hbar)?;
        
        // Return diagonal matrix with eigenvalues
        let n = self.truncation;
        let matrix = DMatrix::from_fn(n, n, |i, j| {
            if i == j {
                Complex::new(eigenvalues[i], 0.0)
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
    
    #[test]
    fn test_sigma_0() {
        let bo = BornOscillator::new(1.0, 10).unwrap();
        
        // At E=1 (ground state), Σ₀ should be 0
        let sigma = bo.sigma_0(1.0);
        assert!(sigma.abs() < 1e-6);
        
        // At higher energies, Σ₀ should increase
        let sigma_2 = bo.sigma_0(2.0);
        let sigma_3 = bo.sigma_0(3.0);
        assert!(sigma_2 > 0.0);
        assert!(sigma_3 > sigma_2);
    }
    
    #[test]
    fn test_wkb_quantization() {
        let bo = BornOscillator::new(1.0, 5).unwrap();
        let eigenvalues = bo.compute_eigenvalues_wkb(1.0).unwrap();
        
        // Should have 5 eigenvalues
        assert_eq!(eigenvalues.len(), 5);
        
        // Eigenvalues should be increasing
        for i in 1..eigenvalues.len() {
            assert!(eigenvalues[i] > eigenvalues[i-1]);
        }
        
        // Ground state should be close to 1 (classical minimum)
        assert!(eigenvalues[0] >= 1.0);
        assert!(eigenvalues[0] < 2.0);
    }
}
