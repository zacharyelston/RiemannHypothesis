/// Explicit Formula Correlation Length Analysis
/// 
/// Tests hypothesis: L≈3-5 is the correlation length of Σ x^ρ/ρ terms
/// 
/// Mathematical framework:
/// ψ(x) = x - Σ_{ρ} x^ρ/ρ - log(2π)
/// Correlation: C(L) = ⟨(x^ρ₁/ρ₁)(x^ρ₂/ρ₂)⟩ where |ρ₁ - ρ₂| = L

use crate::utils::RiemannError;
use std::f64::consts::PI;

/// Explicit formula evaluator for ψ(x)
pub struct ExplicitFormula {
    zeros: Vec<f64>,  // Riemann zeros (imaginary parts)
}

impl ExplicitFormula {
    pub fn new(zeros: Vec<f64>) -> Self {
        Self { zeros }
    }

    /// Compute ψ(x) = x - Σ x^ρ/ρ - log(2π)
    /// where ρ = 1/2 + iγ (γ are the zeros)
    pub fn compute_psi(&self, x: f64) -> f64 {
        // Main term: x
        let mut psi = x;
        
        // Zero contribution: -Σ x^ρ/ρ
        for &gamma in &self.zeros {
            let rho_real = 0.5;
            let rho_imag = gamma;
            
            // x^ρ = x^(1/2 + iγ) = x^(1/2) * x^(iγ) = sqrt(x) * e^(iγ ln(x))
            let sqrt_x = x.sqrt();
            let ln_x = x.ln();
            
            // Complex contribution: x^ρ/ρ = (sqrt_x * e^(iγ ln(x))) / (0.5 + iγ)
            let numerator_real = sqrt_x * (gamma * ln_x).cos();
            let numerator_imag = sqrt_x * (gamma * ln_x).sin();
            
            let denominator = 0.5 * 0.5 + gamma * gamma;
            let contribution_real = (numerator_real * 0.5 + numerator_imag * gamma) / denominator;
            let contribution_imag = (numerator_imag * 0.5 - numerator_real * gamma) / denominator;
            
            // ψ(x) is real, so we take the real part
            psi -= contribution_real;
        }
        
        // Subtract log(2π)
        psi - (2.0 * PI).ln()
    }

    /// Compute correlation function C(L) for zero contributions
    /// C(L) = ⟨(x^ρ₁/ρ₁)(x^ρ₂/ρ₂)⟩ where |ρ₁ - ρ₂| = L
    pub fn compute_correlation(&self, max_l: f64, l_steps: usize, x_values: &[f64]) -> Vec<(f64, f64)> {
        let mut correlations = Vec::new();
        
        for i in 0..l_steps {
            let l = (i as f64 / l_steps as f64) * max_l;
            let mut correlation_sum = 0.0;
            let mut count = 0;
            
            // Find all pairs of zeros with |γ₁ - γ₂| ≈ L
            for (j, &gamma1) in self.zeros.iter().enumerate() {
                for &gamma2 in self.zeros.iter().skip(j) {
                    let delta_gamma = (gamma2 - gamma1).abs();
                    if (delta_gamma - l).abs() < 0.5 { // Within 0.5 of target L
                        // Compute correlation across x values
                        let mut pair_correlation = 0.0;
                        for &x in x_values {
                            let contrib1 = self.zero_contribution(x, gamma1);
                            let contrib2 = self.zero_contribution(x, gamma2);
                            pair_correlation += contrib1 * contrib2;
                        }
                        correlation_sum += pair_correlation / x_values.len() as f64;
                        count += 1;
                    }
                }
            }
            
            let avg_correlation = if count > 0 {
                correlation_sum / count as f64
            } else {
                0.0
            };
            
            correlations.push((l, avg_correlation));
        }
        
        correlations
    }

    /// Compute single zero contribution x^ρ/ρ
    fn zero_contribution(&self, x: f64, gamma: f64) -> f64 {
        let sqrt_x = x.sqrt();
        let ln_x = x.ln();
        
        // x^ρ/ρ = (sqrt_x * e^(iγ ln(x))) / (0.5 + iγ)
        let numerator_real = sqrt_x * (gamma * ln_x).cos();
        let numerator_imag = sqrt_x * (gamma * ln_x).sin();
        
        let denominator = 0.5 * 0.5 + gamma * gamma;
        let contribution_real = (numerator_real * 0.5 + numerator_imag * gamma) / denominator;
        
        contribution_real
    }

    /// Estimate correlation length from correlation function
    /// Returns the L where correlation drops to 1/e of initial value
    pub fn estimate_correlation_length(&self, correlations: &[(f64, f64)]) -> Option<f64> {
        if correlations.is_empty() {
            return None;
        }
        
        // Find initial correlation value (at smallest L)
        let initial_corr = correlations[0].1.abs();
        if initial_corr < 1e-10 {
            return None;
        }
        
        // Find where correlation drops to 1/e of initial value
        let threshold = initial_corr / std::f64::consts::E;
        
        for &(l, corr) in correlations {
            if corr.abs() < threshold {
                return Some(l);
            }
        }
        
        None
    }
}

/// Analyze correlation length for a set of zeros
pub fn analyze_correlation_length(zeros: &[f64]) -> Result<f64, RiemannError> {
    if zeros.len() < 10 {
        return Err(RiemannError::InvalidSize(zeros.len()));
    }

    let formula = ExplicitFormula::new(zeros.to_vec());
    
    // Use logarithmically spaced x values
    let mut x_values = Vec::new();
    for i in 0..50 {
        let x = 10.0_f64.powi(i - 25); // From 10^-25 to 10^25
        x_values.push(x);
    }
    
    // Compute correlation up to L=10 (should capture L≈3-5 if present)
    let correlations = formula.compute_correlation(10.0, 100, &x_values);
    
    // Estimate correlation length
    match formula.estimate_correlation_length(&correlations) {
        Some(length) => Ok(length),
        None => Err(RiemannError::DiagonalizationError(
            "Could not estimate correlation length".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_explicit_formula_basic() {
        let zeros = vec![14.134725, 21.022040, 25.010858]; // First few zeros
        let formula = ExplicitFormula::new(zeros);
        
        // Test ψ(x) computation
        let psi_100 = formula.compute_psi(100.0);
        assert!(psi_100.is_finite());
        
        // Should be close to x for large x (zero contributions small)
        assert!((psi_100 - 100.0).abs() < 10.0);
    }

    #[test]
    fn test_correlation_computation() {
        let zeros = vec![14.134725, 21.022040, 25.010858, 30.424876];
        let formula = ExplicitFormula::new(zeros);
        
        let x_values = vec![10.0, 100.0, 1000.0];
        let correlations = formula.compute_correlation(5.0, 50, &x_values);
        
        assert_eq!(correlations.len(), 50);
        
        // Check that correlations are computed
        for &(l, corr) in &correlations {
            assert!(l >= 0.0 && l <= 5.0);
            assert!(corr.is_finite());
        }
    }

    #[test]
    fn test_correlation_length_estimation() {
        let zeros = vec![14.134725, 21.022040, 25.010858, 30.424876, 32.935062];
        let formula = ExplicitFormula::new(zeros);
        
        let x_values = vec![10.0, 100.0, 1000.0];
        let correlations = formula.compute_correlation(10.0, 100, &x_values);
        
        let length = formula.estimate_correlation_length(&correlations);
        
        // Should get some finite correlation length
        assert!(length.is_some());
        if let Some(l) = length {
            assert!(l > 0.0 && l < 10.0);
        }
    }

    #[test]
    fn test_analyze_correlation_length() {
        let zeros = vec![14.134725, 21.022040, 25.010858, 30.424876, 32.935062];
        
        let length = analyze_correlation_length(&zeros);
        assert!(length.is_ok());
        
        if Ok(length) = length {
            assert!(length > 0.0 && length < 10.0);
        }
    }
}
