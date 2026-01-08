/// Unfolding trait for different types of spectral data
/// 
/// Different systems require different unfolding methods:
/// - Riemann zeros: Riemann-von Mangoldt counting function
/// GUE matrices: Semicircle density
/// - General eigenvalues: Local density estimation

use crate::analysis::unfolding::unfold_zeros;

/// Trait for unfolding spectral data to unit mean spacing
pub trait UnfoldingMethod {
    fn unfold_spectrum(&self, data: &[f64]) -> Vec<f64>;
}

/// Riemann zeros unfolding using Riemann-von Mangoldt
pub struct RiemannUnfolding;

impl UnfoldingMethod for RiemannUnfolding {
    fn unfold_spectrum(&self, data: &[f64]) -> Vec<f64> {
        unfold_zeros(data)
    }
}

/// GUE unfolding using semicircle density
pub struct GueUnfolding;

impl UnfoldingMethod for GueUnfolding {
    fn unfold_spectrum(&self, data: &[f64]) -> Vec<f64> {
        // For GUE, eigenvalues follow Wigner semicircle distribution
        // Use local density estimation
        unfold_local_density(data)
    }
}

/// Local density unfolding for general eigenvalue sequences
pub fn unfold_local_density(data: &[f64]) -> Vec<f64> {
    if data.len() < 10 {
        return data.to_vec();
    }
    
    // Compute local density using sliding window
    let window_size = (data.len() as f64 * 0.1).max(10.0);
    let mut unfolded = Vec::new();
    
    for i in 0..data.len() {
        let start = i;
        let end = (i + window_size as usize).min(data.len());
        
        if end > start {
            let window: Vec<f64> = data[start..end].to_vec();
            let range = window.last().unwrap() - window.first().unwrap();
            if range > 0.0 {
                let density = window.len() as f64 / range;
                let mean_spacing = 1.0 / density;
                
                for &x in window.iter() {
                    unfolded.push(x / mean_spacing);
                }
            }
        }
    }
    
    unfolded
}
