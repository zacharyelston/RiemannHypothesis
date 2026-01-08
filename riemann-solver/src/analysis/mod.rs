use serde::{Deserialize, Serialize};

/// Results from a spectral analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralStats {
    pub mean_spacing: f64,
    pub variance: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub gue_match_confidence: f64, // 0.0 to 1.0
}

/// Analyzer trait for verifying hypotheses against the spectrum.
pub trait SpectrumAnalyzer {
    /// Normalizes raw eigenvalues (unfolding) based on local density of states.
    fn unfold_spectrum(&self, raw_eigenvalues: &[f64]) -> Vec<f64>;
    
    /// Computes statistical properties of the unfolded spectrum.
    fn analyze(&self, unfolded_spectrum: &[f64]) -> SpectralStats;
}

pub mod spacing;
pub mod spectral;
pub mod unfolding;
pub mod explicit_formula;
pub mod primes;
pub mod riemann_siegel;
pub mod ks_test;
