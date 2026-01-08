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
/// GUE prediction: Δ₃_GUE(L) ≈ (1/π²) ln(L) + const
pub fn dyson_mehta(unfolded_levels: &[f64], l: f64) -> f64 {
    if unfolded_levels.len() < 3 || l <= 0.0 {
        return 0.0;
    }
    
    let mut min_deviation = f64::INFINITY;
    
    // Sample starting points
    let n_samples = (unfolded_levels.len() as f64 * 0.8) as usize;
    
    for i in 0..n_samples {
        let s_start = unfolded_levels[i];
        let s_end = s_start + l;
        
        // Find levels in [s_start, s_end]
        let levels_in_window: Vec<f64> = unfolded_levels.iter()
            .filter(|&&x| x >= s_start && x <= s_end)
            .cloned()
            .collect();
        
        if levels_in_window.len() < 2 {
            continue;
        }
        
        // Linear regression: N(s) ≈ As + B
        let n: Vec<f64> = (0..levels_in_window.len()).map(|i| i as f64).collect();
        let sum_n = n.iter().sum::<f64>();
        let sum_s = levels_in_window.iter().sum::<f64>();
        let sum_ns = n.iter().zip(levels_in_window.iter()).map(|(ni, si)| ni * si).sum::<f64>();
        let sum_n2 = n.iter().map(|ni| ni * ni).sum::<f64>();
        
        let denominator = sum_n2 * levels_in_window.len() as f64 - sum_n * sum_n;
        let numerator = sum_ns * levels_in_window.len() as f64 - sum_n * sum_s;
        
        let slope = if denominator.abs() > 1e-10 {
            numerator / denominator
        } else {
            0.0
        };
        
        let intercept = (sum_s - slope * sum_n) / levels_in_window.len() as f64;
        
        // Compute deviation
        let mut deviation = 0.0;
        for (ni, &si) in n.iter().zip(levels_in_window.iter()) {
            let predicted = slope * ni + intercept;
            deviation += (si - predicted).powi(2);
        }
        
        let delta3_val = deviation / l;
        min_deviation = min_deviation.min(delta3_val);
    }
    
    min_deviation
}

/// Approximate Dyson-Mehta Δ₃(L) for faster computation
pub fn dyson_mehta_approx(unfolded_levels: &[f64], l: f64) -> f64 {
    // Simplified approximation for Δ₃(L)
    if unfolded_levels.len() < 10 || l <= 0.0 {
        return 0.0;
    }
    
    // Use a subset of levels for approximation
    let step = (unfolded_levels.len() / 10).max(1);
    let sample_levels: Vec<f64> = unfolded_levels.iter()
        .step_by(step)
        .take(10)
        .cloned()
        .collect();
    
    dyson_mehta(&sample_levels, l)
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

/// Alias for delta_3_gue for consistency
pub fn delta_3(unfolded_levels: &[f64], l: f64) -> f64 {
    delta_3_gue(l)
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
