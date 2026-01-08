/// Spectral rigidity metrics for analyzing level statistics
/// 
/// Implements:
/// - Number variance Σ²(L)
/// - Dyson-Mehta Δ₃(L) statistic
/// 
/// These metrics measure long-range correlations in eigenvalue sequences
/// and distinguish between GUE (quantum chaos) and Poisson (integrable) systems.

use std::f64::consts::PI;

/// Number variance Σ²(L)
/// 
/// Measures fluctuations in the number of levels in intervals of length L.
/// 
/// Σ²(L) = ⟨[n(s, s+L) - L]²⟩
/// 
/// where n(s, s+L) is the number of unfolded levels in [s, s+L]
/// 
/// GUE prediction: Σ²_GUE(L) ≈ (2/π²) log(2πL) + const
/// Poisson: Σ²_Poisson(L) = L
pub fn number_variance(unfolded_levels: &[f64], l: f64) -> f64 {
    if unfolded_levels.len() < 2 || l <= 0.0 {
        return 0.0;
    }
    
    let mut sum_sq = 0.0;
    let mut count = 0;
    
    // Sample starting points
    let n_samples = (unfolded_levels.len() as f64 * 0.8) as usize;
    
    for i in 0..n_samples {
        let s = unfolded_levels[i];
        let s_plus_l = s + l;
        
        // Count levels in [s, s+L]
        let n_in_interval = unfolded_levels.iter()
            .filter(|&&x| x >= s && x < s_plus_l)
            .count() as f64;
        
        // Deviation from expected (L)
        let deviation = n_in_interval - l;
        sum_sq += deviation * deviation;
        count += 1;
    }
    
    if count > 0 {
        sum_sq / count as f64
    } else {
        0.0
    }
}

/// GUE prediction for number variance
/// Σ²_GUE(L) = (2/π²) log(2πL) + C
pub fn number_variance_gue(l: f64) -> f64 {
    if l <= 0.0 {
        return 0.0;
    }
    let c = 0.0; // Constant term (varies by convention)
    (2.0 / (PI * PI)) * (2.0 * PI * l).ln() + c
}

/// Dyson-Mehta Δ₃(L) statistic
/// 
/// Measures deviation of staircase function from best-fit straight line.
/// 
/// Δ₃(L) = min_{A,B} (1/L) ∫₀ᴸ [N(E) - AE - B]² dE
/// 
/// where N(E) is the integrated level density (staircase function)
/// 
/// GUE prediction: Δ₃_GUE(L) ≈ (1/π²) log(2πL) + const
/// Poisson: Δ₃_Poisson(L) = L/15
pub fn delta_3(unfolded_levels: &[f64], l: f64) -> f64 {
    if unfolded_levels.len() < 10 || l <= 0.0 {
        return 0.0;
    }
    
    let mut sum_delta3 = 0.0;
    let mut count = 0;
    
    // Sample starting points
    let n_samples = ((unfolded_levels.len() as f64 - l) * 0.5) as usize;
    if n_samples == 0 {
        return 0.0;
    }
    
    for i in 0..n_samples.min(unfolded_levels.len() - 1) {
        let s_start = unfolded_levels[i];
        let s_end = s_start + l;
        
        // Get levels in [s_start, s_end]
        let levels_in_range: Vec<f64> = unfolded_levels.iter()
            .filter(|&&x| x >= s_start && x <= s_end)
            .map(|&x| x - s_start) // Shift to start at 0
            .collect();
        
        if levels_in_range.len() < 3 {
            continue;
        }
        
        // Build staircase function N(E)
        // N(E) = number of levels ≤ E
        
        // Find best-fit line: minimize ∫[N(E) - AE - B]² dE
        // Solution: A = mean slope, B = offset
        let n = levels_in_range.len() as f64;
        let a = n / l; // Average density
        let b = 0.0;   // Offset (simplified)
        
        // Compute integral using trapezoidal rule
        let mut integral = 0.0;
        let n_points = 100;
        let de = l / n_points as f64;
        
        for j in 0..=n_points {
            let e = j as f64 * de;
            let n_e = levels_in_range.iter().filter(|&&x| x <= e).count() as f64;
            let deviation = n_e - a * e - b;
            let weight = if j == 0 || j == n_points { 0.5 } else { 1.0 };
            integral += weight * deviation * deviation * de;
        }
        
        sum_delta3 += integral / l;
        count += 1;
    }
    
    if count > 0 {
        sum_delta3 / count as f64
    } else {
        0.0
    }
}

/// GUE prediction for Δ₃(L)
/// Δ₃_GUE(L) = (1/π²) log(2πL) + C
pub fn delta_3_gue(l: f64) -> f64 {
    if l <= 0.0 {
        return 0.0;
    }
    let c = -0.007; // Empirical constant
    (1.0 / (PI * PI)) * (2.0 * PI * l).ln() + c
}

/// Compute rigidity statistics over a range of L values
pub fn compute_rigidity_curve(
    unfolded_levels: &[f64],
    l_min: f64,
    l_max: f64,
    n_points: usize,
) -> Vec<(f64, f64, f64)> {
    let mut results = Vec::with_capacity(n_points);
    
    for i in 0..n_points {
        let l = l_min + (l_max - l_min) * (i as f64) / (n_points as f64);
        let sigma2 = number_variance(unfolded_levels, l);
        let d3 = delta_3(unfolded_levels, l);
        results.push((l, sigma2, d3));
    }
    
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_variance_uniform() {
        // Uniform spacing (Poisson-like)
        let levels: Vec<f64> = (0..100).map(|i| i as f64).collect();
        
        let sigma2 = number_variance(&levels, 10.0);
        
        // For uniform spacing, variance should be small
        assert!(sigma2 < 2.0, "Σ²(10) = {} for uniform spacing", sigma2);
    }
    
    #[test]
    fn test_number_variance_gue_prediction() {
        // GUE prediction should increase logarithmically
        let l1 = 5.0;
        let l2 = 50.0;
        
        let sigma2_1 = number_variance_gue(l1);
        let sigma2_2 = number_variance_gue(l2);
        
        assert!(sigma2_2 > sigma2_1, "Σ²_GUE should increase with L");
        
        // Check logarithmic growth
        let ratio = sigma2_2 / sigma2_1;
        let log_ratio = (l2 / l1).ln();
        assert!((ratio - log_ratio).abs() < 1.0, "Should grow logarithmically");
    }
    
    #[test]
    fn test_delta_3_uniform() {
        // Uniform spacing
        let levels: Vec<f64> = (0..100).map(|i| i as f64).collect();
        
        let d3 = delta_3(&levels, 10.0);
        
        // For uniform spacing, Δ₃ should be relatively small
        // (but not zero due to discreteness)
        assert!(d3 < 1.0, "Δ₃(10) = {} for uniform spacing", d3);
        assert!(d3 >= 0.0, "Δ₃ should be non-negative");
    }
    
    #[test]
    fn test_delta_3_gue_prediction() {
        // GUE prediction should increase logarithmically
        let l1 = 5.0;
        let l2 = 50.0;
        
        let d3_1 = delta_3_gue(l1);
        let d3_2 = delta_3_gue(l2);
        
        assert!(d3_2 > d3_1, "Δ₃_GUE should increase with L");
    }
}
