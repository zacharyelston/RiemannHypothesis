/// Kolmogorov-Smirnov test for comparing spacing distribution to Wigner surmise
/// 
/// The Wigner surmise for GUE is:
/// P(s) = (π/2) s exp(-πs²/4)
/// 
/// CDF: F(s) = 1 - exp(-πs²/4)

use std::f64::consts::PI;

/// Wigner surmise CDF for GUE spacing distribution
/// F(s) = 1 - exp(-πs²/4)
pub fn wigner_cdf(s: f64) -> f64 {
    if s <= 0.0 {
        return 0.0;
    }
    1.0 - (-PI * s * s / 4.0).exp()
}

/// Compute empirical CDF from sorted data
fn empirical_cdf(data: &[f64], x: f64) -> f64 {
    let count = data.iter().filter(|&&val| val <= x).count();
    count as f64 / data.len() as f64
}

/// Kolmogorov-Smirnov statistic
/// 
/// D = max |F_empirical(x) - F_theory(x)|
/// 
/// Returns (D_statistic, p_value_approx)
pub fn ks_test_wigner(spacings: &[f64]) -> (f64, f64) {
    if spacings.is_empty() {
        return (0.0, 0.0);
    }
    
    let mut sorted = spacings.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    let n = sorted.len() as f64;
    let mut d_max = 0.0;
    
    for (i, &s) in sorted.iter().enumerate() {
        let empirical = (i + 1) as f64 / n;
        let theoretical = wigner_cdf(s);
        let d = (empirical - theoretical).abs();
        
        if d > d_max {
            d_max = d;
        }
        
        // Also check at the point just before
        if i > 0 {
            let empirical_before = i as f64 / n;
            let d_before = (empirical_before - theoretical).abs();
            if d_before > d_max {
                d_max = d_before;
            }
        }
    }
    
    // Approximate p-value using Kolmogorov distribution
    // For large n: P(D > d) ≈ 2 * sum_{k=1}^∞ (-1)^{k-1} exp(-2k²d²n)
    // Simplified: p ≈ 2 * exp(-2 * d² * n)
    let p_value = 2.0 * (-2.0 * d_max * d_max * n).exp();
    let p_value = p_value.min(1.0); // Cap at 1.0
    
    (d_max, p_value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wigner_cdf() {
        // At s=0, CDF should be 0
        assert!((wigner_cdf(0.0) - 0.0).abs() < 1e-10);
        
        // CDF should be monotonically increasing
        assert!(wigner_cdf(1.0) > wigner_cdf(0.5));
        assert!(wigner_cdf(2.0) > wigner_cdf(1.0));
        
        // As s→∞, CDF→1
        assert!(wigner_cdf(10.0) > 0.99);
    }
    
    #[test]
    fn test_ks_test_perfect_match() {
        // Generate spacings from Wigner distribution (approximately)
        let spacings: Vec<f64> = (1..100)
            .map(|i| {
                let u = i as f64 / 100.0;
                // Inverse CDF sampling (approximate)
                ((-4.0 / PI) * (1.0 - u).ln()).sqrt()
            })
            .collect();
        
        let (d, p) = ks_test_wigner(&spacings);
        
        // D should be small for good match
        assert!(d < 0.2, "D = {} should be small", d);
        // p-value should be reasonably high
        assert!(p > 0.01, "p = {} should not reject null hypothesis", p);
    }
    
    #[test]
    fn test_ks_test_uniform() {
        // Uniform distribution should NOT match Wigner
        let spacings: Vec<f64> = (1..100).map(|i| i as f64 / 100.0).collect();
        
        let (d, _p) = ks_test_wigner(&spacings);
        
        // D should be large for poor match
        assert!(d > 0.1, "D = {} should be large for non-Wigner distribution", d);
    }
}
