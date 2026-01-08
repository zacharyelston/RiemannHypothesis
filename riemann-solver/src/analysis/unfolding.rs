use std::f64::consts::PI;

/// Riemann-von Mangoldt counting function N(T)
/// 
/// N(T) = number of zeros with 0 < Im(ρ) ≤ T
/// 
/// Standard asymptotic formula (Riemann-von Mangoldt):
/// N(T) = (T/2π) log(T/2π) - T/2π + 7/8 + S(T) + O(1/T)
/// 
/// where S(T) is a small oscillating term we ignore for large T
pub fn riemann_von_mangoldt(t: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    
    let two_pi = 2.0 * PI;
    let theta = t / two_pi;
    
    theta * theta.ln() - theta + 7.0 / 8.0
}

/// Unfold a sequence of Riemann zeros to mean spacing = 1
/// 
/// Input: γ_n (imaginary parts of zeros)
/// Output: Unfolded levels x_n = N(γ_n)
/// 
/// The unfolded spacings s_n = x_{n+1} - x_n should have mean ≈ 1
/// and follow Wigner-Dyson statistics if RH zeros are GUE-like.
pub fn unfold_zeros(zeros: &[f64]) -> Vec<f64> {
    zeros.iter()
        .map(|&gamma| riemann_von_mangoldt(gamma))
        .collect()
}

/// Compute spacings from unfolded levels
pub fn compute_spacings(unfolded: &[f64]) -> Vec<f64> {
    if unfolded.len() < 2 {
        return Vec::new();
    }
    
    unfolded.windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

/// Load Riemann zeros from text file (one zero per line)
pub fn load_zeros_from_file(path: &str) -> Result<Vec<f64>, std::io::Error> {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    
    let mut zeros = Vec::new();
    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        
        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        if let Ok(value) = trimmed.parse::<f64>() {
            zeros.push(value);
        }
    }
    
    Ok(zeros)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_riemann_von_mangoldt() {
        // Test that N(T) is monotonically increasing
        let gamma_1 = 14.134725142;
        let gamma_10 = 49.773832478;
        let gamma_100 = 236.524229666;
        
        let n1 = riemann_von_mangoldt(gamma_1);
        let n10 = riemann_von_mangoldt(gamma_10);
        let n100 = riemann_von_mangoldt(gamma_100);
        
        assert!(n10 > n1, "N(T) should be monotonically increasing");
        assert!(n100 > n10, "N(T) should be monotonically increasing");
        
        // For large T, asymptotic formula should be accurate
        // N(γ_100) should be close to 100
        assert!((n100 - 100.0).abs() < 5.0, "N(γ_100) = {}, should be close to 100", n100);
    }
    
    #[test]
    fn test_unfold_zeros() {
        let zeros = vec![14.134725142, 21.022039639, 25.010857580];
        let unfolded = unfold_zeros(&zeros);
        
        assert_eq!(unfolded.len(), 3);
        
        // Unfolded values should be monotonically increasing
        assert!(unfolded[1] > unfolded[0], "Unfolded sequence should be increasing");
        assert!(unfolded[2] > unfolded[1], "Unfolded sequence should be increasing");
        
        // All values should be positive
        assert!(unfolded[0] > 0.0);
        assert!(unfolded[1] > 0.0);
        assert!(unfolded[2] > 0.0);
    }
    
    #[test]
    fn test_compute_spacings() {
        let unfolded = vec![1.0, 2.1, 3.0, 4.2];
        let spacings = compute_spacings(&unfolded);
        
        assert_eq!(spacings.len(), 3);
        assert!((spacings[0] - 1.1).abs() < 1e-10);
        assert!((spacings[1] - 0.9).abs() < 1e-10);
        assert!((spacings[2] - 1.2).abs() < 1e-10);
    }
    
    #[test]
    fn test_mean_spacing_near_one() {
        // Use first 10 known zeros
        let zeros = vec![
            14.134725142, 21.022039639, 25.010857580, 30.424876126,
            32.935061588, 37.586178159, 40.918719012, 43.327073281,
            48.005150881, 49.773832478,
        ];
        
        let unfolded = unfold_zeros(&zeros);
        let spacings = compute_spacings(&unfolded);
        
        let mean: f64 = spacings.iter().sum::<f64>() / spacings.len() as f64;
        
        // Mean spacing should be close to 1
        assert!((mean - 1.0).abs() < 0.1, "Mean spacing = {}, expected ≈ 1.0", mean);
    }
}
