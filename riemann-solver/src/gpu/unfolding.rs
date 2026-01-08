/// Proper eigenvalue unfolding for spectral rigidity metrics.
/// 
/// The key insight: rigidity metrics require UNFOLDED eigenvalues where
/// the mean spacing is normalized to 1.0. Using raw eigenvalues causes
/// the 7M× error seen in the broken implementation.

use crate::utils::RiemannError;

/// Unfold eigenvalues to mean spacing = 1.0
/// 
/// For a spectrum {λ₁, λ₂, ..., λₙ}, unfolding computes:
/// 1. Sort eigenvalues
/// 2. Compute spacings: sₙ = λₙ₊₁ - λₙ
/// 3. Normalize: s̃ₙ = sₙ / ⟨s⟩ where ⟨s⟩ is mean spacing
/// 4. Cumulative sum: x̃ₙ = Σ s̃ᵢ (unfolded levels)
pub fn unfold_eigenvalues(eigenvalues: &[f64]) -> Result<Vec<f64>, RiemannError> {
    if eigenvalues.len() < 2 {
        return Err(RiemannError::InvalidSize(eigenvalues.len()));
    }

    let mut sorted = eigenvalues.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Compute spacings
    let mut spacings = Vec::new();
    for i in 0..sorted.len() - 1 {
        let spacing = sorted[i + 1] - sorted[i];
        if spacing > 0.0 {
            spacings.push(spacing);
        }
    }

    if spacings.is_empty() {
        return Err(RiemannError::DiagonalizationError(
            "No positive spacings found".to_string(),
        ));
    }

    // Mean spacing
    let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;

    // Normalize spacings to mean = 1
    let normalized_spacings: Vec<f64> = spacings.iter().map(|s| s / mean_spacing).collect();

    // Cumulative sum = unfolded levels
    let mut unfolded = Vec::with_capacity(normalized_spacings.len() + 1);
    unfolded.push(0.0); // x₀ = 0

    let mut cumsum = 0.0;
    for &s in &normalized_spacings {
        cumsum += s;
        unfolded.push(cumsum);
    }

    Ok(unfolded)
}

/// Compute spacings from unfolded eigenvalues
pub fn compute_spacings_from_unfolded(unfolded: &[f64]) -> Vec<f64> {
    if unfolded.len() < 2 {
        return vec![];
    }

    let mut spacings = Vec::new();
    for i in 0..unfolded.len() - 1 {
        let spacing = unfolded[i + 1] - unfolded[i];
        if spacing > 0.0 {
            spacings.push(spacing);
        }
    }
    spacings
}

/// Verify unfolding: mean spacing should be ≈ 1.0
pub fn verify_unfolding(unfolded: &[f64]) -> f64 {
    let spacings = compute_spacings_from_unfolded(unfolded);
    if spacings.is_empty() {
        return 0.0;
    }
    spacings.iter().sum::<f64>() / spacings.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unfolding_mean_spacing() {
        // Create eigenvalues with known spacing
        let eigenvalues = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let unfolded = unfold_eigenvalues(&eigenvalues).unwrap();

        // Mean spacing should be 1.0
        let mean = verify_unfolding(&unfolded);
        assert!((mean - 1.0).abs() < 1e-10, "Mean spacing: {}", mean);
    }

    #[test]
    fn test_unfolding_non_uniform() {
        // Non-uniform spacing
        let eigenvalues = vec![0.0, 0.5, 2.0, 2.5, 5.0];
        let unfolded = unfold_eigenvalues(&eigenvalues).unwrap();

        // Should still normalize to mean = 1.0
        let mean = verify_unfolding(&unfolded);
        assert!((mean - 1.0).abs() < 1e-10, "Mean spacing: {}", mean);
    }
}
