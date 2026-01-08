/// Fixed spectral rigidity metrics for proper statistical analysis
/// 
/// Addresses the estimator artifacts found in validation
/// - Proper window sampling
/// - Correct theoretical comparisons
/// - Bootstrap confidence intervals

use std::f64::consts::PI;

/// Fixed number variance Σ²(L) computation
/// 
/// Σ²(L) = ⟨[n(s, s+L) - L]²⟩
/// 
/// where n(s, s+L) is the number of unfolded levels in [s, s+L]
pub fn number_variance_fixed(unfolded_levels: &[f64], l: f64) -> f64 {
    if unfolded_levels.len() < 10 || l <= 0.0 {
        return 0.0;
    }
    
    let mut sum_sq = 0.0;
    let mut count = 0;
    
    // Sample starting points properly
    let n_samples = (unfolded_levels.len() as f64 * 0.5) as usize;
    let step = (unfolded_levels.len() - 1) / n_samples.max(1);
    
    for i in (0..unfolded_levels.len() - 1).step_by(step) {
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

/// Fixed GUE prediction for number variance
/// Σ²_GUE(L) = (2/π²) ln(L) + γ + 1 - (π²/8)
pub fn number_variance_gue_fixed(l: f64) -> f64 {
    if l <= 0.0 {
        return 0.0;
    }
    
    let gamma = 0.5772156649015328606065120900824024310421; // Euler-Mascheroni constant
    
    if l < 1.0 {
        // Linear approximation for small L
        return l * (2.0 / PI);
    }
    
    (2.0 / (PI * PI)) * (l.ln() + gamma + 1.0 - (PI * PI) / 8.0)
}

/// Fixed Dyson-Mehta Δ₃(L) statistic
/// 
/// Δ₃(L) = min_{A,B} (1/L) ∫₀ᴸ [N(E) - AE - B]² dE
pub fn dyson_mehta_fixed(unfolded_levels: &[f64], l: f64) -> f64 {
    if unfolded_levels.len() < 10 || l <= 0.0 {
        return 0.0;
    }
    
    let mut min_deviation = f64::INFINITY;
    
    // Sample starting points
    let n_samples = (unfolded_levels.len() as f64 * 0.3) as usize;
    let step = (unfolded_levels.len() - 1) / n_samples.max(1);
    
    for i in (0..unfolded_levels.len() - 1).step_by(step) {
        let s_start = unfolded_levels[i];
        let s_end = s_start + l;
        
        // Find levels in [s_start, s_end]
        let levels_in_window: Vec<f64> = unfolded_levels.iter()
            .filter(|&&x| x >= s_start && x <= s_end)
            .cloned()
            .collect();
        
        if levels_in_window.len() < 3 {
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

/// Fixed GUE prediction for Δ₃(L)
/// Δ₃_GUE(L) = (1/π²) ln(L) + C
pub fn dyson_mehta_gue_fixed(l: f64) -> f64 {
    if l <= 0.0 {
        return 0.0;
    }
    
    let c = -0.007; // Empirical constant
    
    if l < 1.0 {
        return l * (1.0 / (PI * PI));
    }
    
    (1.0 / (PI * PI)) * l.ln() + c
}

/// Bootstrap confidence intervals for rigidity metrics
pub fn bootstrap_rigidity(
    unfolded: &[f64], 
    l_values: &[f64], 
    n_samples: usize,
    compute_fn: impl Fn(&[f64], f64) -> f64
) -> Vec<(f64, f64, f64)> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let mut results = Vec::new();
    
    for &l in l_values {
        let mut bootstrap_values = Vec::new();
        
        for _ in 0..n_samples {
            // Bootstrap sample with replacement
            let mut sample = Vec::with_capacity(unfolded.len());
            for _ in 0..unfolded.len() {
                let idx = rng.gen_range(0..unfolded.len());
                sample.push(unfolded[idx]);
            }
            
            let sample_result = compute_fn(&sample, l);
            bootstrap_values.push(sample_result);
        }
        
        bootstrap_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = bootstrap_values.iter().sum::<f64>() / bootstrap_values.len() as f64;
        let lower = bootstrap_values[(bootstrap_values.len() as f64 * 0.025) as usize];
        let upper = bootstrap_values[(bootstrap_values.len() as f64 * 0.975) as usize];
        
        results.push((mean, lower, upper));
    }
    
    results
}

/// Find crossover point with interpolation
pub fn find_crossover_fixed(
    l_values: &[f64], 
    measured: &[(f64, f64, f64)],
    theoretical: &[f64]
) -> CrossoverResultFixed {
    let mut crossings = Vec::new();
    
    for i in 1..l_values.len() {
        let ratio_prev = measured[i-1].0 / theoretical[i-1];
        let ratio_curr = measured[i].0 / theoretical[i];
        
        if (ratio_prev - 1.0) * (ratio_curr - 1.0) < 0.0 {
            // Linear interpolation
            let l_cross = l_values[i-1] + (l_values[i] - l_values[i-1]) * 
                (1.0 - ratio_prev) / (ratio_curr - ratio_prev);
            crossings.push(l_cross);
        }
    }
    
    if crossings.is_empty() {
        return CrossoverResultFixed {
            found: false,
            l_star: 0.0,
            confidence_lower: 0.0,
            confidence_upper: 0.0,
        };
    }
    
    // Use the first crossing (smallest L)
    let l_star = crossings[0];
    
    // Estimate confidence bounds
    let confidence_width = 0.2; // Rough estimate
    CrossoverResultFixed {
        found: true,
        l_star,
        confidence_lower: l_star - confidence_width,
        confidence_upper: l_star + confidence_width,
    }
}

#[derive(Debug)]
pub struct CrossoverResultFixed {
    pub found: bool,
    pub l_star: f64,
    pub confidence_lower: f64,
    pub confidence_upper: f64,
}
