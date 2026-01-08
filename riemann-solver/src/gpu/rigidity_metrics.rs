/// Correct spectral rigidity metrics using properly unfolded eigenvalues.
/// 
/// The broken implementation used raw eigenvalues, causing 7M× errors.
/// This module computes metrics correctly using unfolded levels.

use crate::gpu::unfolding::unfold_eigenvalues;
use crate::utils::RiemannError;

/// Number variance Σ²(L) - measures level clustering
/// 
/// Theory: Σ²(L) = ⟨(n(E, E+L) - L)²⟩
/// where n(E, E+L) = count of unfolded levels in interval [E, E+L]
/// 
/// GUE prediction: Σ²_GUE(L) = (2/π²) log(2πL) + const
/// For L=5: Σ²_GUE ≈ 0.70
/// For L=10: Σ²_GUE ≈ 0.84
/// For L=20: Σ²_GUE ≈ 0.98
pub fn compute_number_variance(
    eigenvalues: &[f64],
    window_sizes: &[f64],
) -> Result<Vec<f64>, RiemannError> {
    // CRITICAL: Use unfolded eigenvalues, not raw eigenvalues
    let unfolded = unfold_eigenvalues(eigenvalues)?;

    let results = window_sizes
        .iter()
        .map(|&L| {
            let mut variance_sum = 0.0;
            let mut count = 0;

            // Slide window across unfolded spectrum
            for i in 0..unfolded.len() {
                let window_start = unfolded[i];
                let window_end = window_start + L;

                // Count levels in window
                let count_in_window = unfolded
                    .iter()
                    .filter(|&&x| x >= window_start && x < window_end)
                    .count() as f64;

                // Deviation from expected count (which is L for mean spacing = 1)
                let deviation = count_in_window - L;
                variance_sum += deviation * deviation;
                count += 1;
            }

            if count > 0 {
                variance_sum / count as f64
            } else {
                0.0
            }
        })
        .collect();

    Ok(results)
}

/// Dyson-Mehta Δ₃(L) - spectral rigidity metric
/// 
/// Theory: Δ₃(L) = min_{A,B} (1/L) ∫₀ᴸ [N(E) - AE - B]² dE
/// where N(E) = number of levels up to E
/// 
/// GUE prediction: Δ₃_GUE(L) = (1/π²) log(2πL) + const
/// For L=5: Δ₃_GUE ≈ 0.035
/// For L=10: Δ₃_GUE ≈ 0.042
/// For L=20: Δ₃_GUE ≈ 0.049
pub fn compute_delta3(
    eigenvalues: &[f64],
    window_sizes: &[f64],
) -> Result<Vec<f64>, RiemannError> {
    // CRITICAL: Use unfolded eigenvalues, not raw eigenvalues
    let unfolded = unfold_eigenvalues(eigenvalues)?;

    let results = window_sizes
        .iter()
        .map(|&L| {
            let mut min_deviation = f64::INFINITY;

            // Try all possible windows
            for start_idx in 0..unfolded.len() {
                let window_start = unfolded[start_idx];
                let window_end = window_start + L;

                // Collect levels in window
                let mut levels_in_window = Vec::new();
                for (idx, &x) in unfolded.iter().enumerate() {
                    if x >= window_start && x < window_end {
                        levels_in_window.push((idx as f64, x));
                    }
                }

                if levels_in_window.len() < 2 {
                    continue;
                }

                // Fit line N(E) = AE + B to minimize deviation
                let n = levels_in_window.len() as f64;
                let sum_idx: f64 = levels_in_window.iter().map(|(i, _)| i).sum();
                let sum_e: f64 = levels_in_window.iter().map(|(_, e)| e).sum();
                let sum_idx_e: f64 = levels_in_window.iter().map(|(i, e)| i * e).sum();
                let sum_e2: f64 = levels_in_window.iter().map(|(_, e)| e * e).sum();

                let denom = n * sum_e2 - sum_e * sum_e;
                if denom.abs() < 1e-10 {
                    continue;
                }

                let a = (n * sum_idx_e - sum_idx * sum_e) / denom;
                let b = (sum_idx - a * sum_e) / n;

                // Compute deviation
                let mut deviation_sum = 0.0;
                for (idx, e) in levels_in_window {
                    let predicted = a * e + b;
                    let actual = idx;
                    deviation_sum += (actual - predicted).powi(2);
                }

                let delta3_val = deviation_sum / L;
                min_deviation = min_deviation.min(delta3_val);
            }

            if min_deviation.is_infinite() {
                0.0
            } else {
                min_deviation
            }
        })
        .collect();

    Ok(results)
}

/// GUE theoretical predictions for validation
pub mod gue_theory {
    /// Σ²_GUE(L) = (2/π²) log(2πL) + 0.0077
    pub fn number_variance_gue(l: f64) -> f64 {
        (2.0 / std::f64::consts::PI.powi(2)) * (2.0 * std::f64::consts::PI * l).ln() + 0.0077
    }

    /// Δ₃_GUE(L) = (1/π²) log(2πL) + 0.0038
    pub fn delta3_gue(l: f64) -> f64 {
        (1.0 / std::f64::consts::PI.powi(2)) * (2.0 * std::f64::consts::PI * l).ln() + 0.0038
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_variance_gue_values() {
        // GUE theory predictions
        let sigma2_5 = gue_theory::number_variance_gue(5.0);
        let sigma2_10 = gue_theory::number_variance_gue(10.0);
        let sigma2_20 = gue_theory::number_variance_gue(20.0);

        // Should be in reasonable range (0.5-1.0)
        assert!(sigma2_5 > 0.5 && sigma2_5 < 1.0, "Σ²(5) = {}", sigma2_5);
        assert!(sigma2_10 > 0.5 && sigma2_10 < 1.0, "Σ²(10) = {}", sigma2_10);
        assert!(sigma2_20 > 0.5 && sigma2_20 < 1.0, "Σ²(20) = {}", sigma2_20);
    }

    #[test]
    fn test_delta3_gue_values() {
        // GUE theory predictions
        let d3_5 = gue_theory::delta3_gue(5.0);
        let d3_10 = gue_theory::delta3_gue(10.0);
        let d3_20 = gue_theory::delta3_gue(20.0);

        // Should be in reasonable range (0.01-0.1)
        assert!(d3_5 > 0.01 && d3_5 < 0.1, "Δ₃(5) = {}", d3_5);
        assert!(d3_10 > 0.01 && d3_10 < 0.1, "Δ₃(10) = {}", d3_10);
        assert!(d3_20 > 0.01 && d3_20 < 0.1, "Δ₃(20) = {}", d3_20);
    }

    #[test]
    fn test_number_variance_uniform_spectrum() {
        // Uniform spacing should give low variance
        let eigenvalues = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let windows = vec![5.0];

        let variance = compute_number_variance(&eigenvalues, &windows).unwrap();
        // For perfectly uniform spacing, variance should be very small
        assert!(variance[0] < 0.1, "Variance for uniform: {}", variance[0]);
    }
}
