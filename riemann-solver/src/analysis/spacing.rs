use crate::analysis::{SpectralStats, SpectrumAnalyzer};

/// Analyzer for nearest-neighbor spacing statistics.
pub struct SpacingAnalyzer;

impl SpacingAnalyzer {
    pub fn new() -> Self {
        Self
    }
}

impl SpectrumAnalyzer for SpacingAnalyzer {
    fn unfold_spectrum(&self, raw_eigenvalues: &[f64]) -> Vec<f64> {
        // Filter to bulk of spectrum (center region)
        let center_evs: Vec<f64> = raw_eigenvalues.iter()
            .filter(|&&x| x.abs() < 1.0)
            .cloned()
            .collect();
        
        if center_evs.is_empty() {
            return vec![];
        }
        
        // Compute spacings
        let mut spacings = Vec::new();
        for i in 0..center_evs.len()-1 {
            let diff = center_evs[i+1] - center_evs[i];
            spacings.push(diff);
        }
        
        if spacings.is_empty() {
            return vec![];
        }
        
        // Normalize to mean spacing = 1
        let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
        spacings.iter().map(|s| s / mean_spacing).collect()
    }
    
    fn analyze(&self, unfolded_spectrum: &[f64]) -> SpectralStats {
        if unfolded_spectrum.is_empty() {
            return SpectralStats {
                mean_spacing: 0.0,
                variance: 0.0,
                skewness: 0.0,
                kurtosis: 0.0,
                gue_match_confidence: 0.0,
            };
        }
        
        let n = unfolded_spectrum.len() as f64;
        let mean = unfolded_spectrum.iter().sum::<f64>() / n;
        
        let variance = unfolded_spectrum.iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f64>() / n;
        
        let std_dev = variance.sqrt();
        
        let skewness = if std_dev > 0.0 {
            unfolded_spectrum.iter()
                .map(|s| ((s - mean) / std_dev).powi(3))
                .sum::<f64>() / n
        } else {
            0.0
        };
        
        let kurtosis = if std_dev > 0.0 {
            unfolded_spectrum.iter()
                .map(|s| ((s - mean) / std_dev).powi(4))
                .sum::<f64>() / n - 3.0
        } else {
            0.0
        };
        
        // GUE theoretical variance is ~0.178
        let gue_variance = 0.178;
        let variance_diff = (variance - gue_variance).abs();
        let gue_match_confidence = (1.0 - (variance_diff / gue_variance).min(1.0)).max(0.0);
        
        SpectralStats {
            mean_spacing: mean,
            variance,
            skewness,
            kurtosis,
            gue_match_confidence,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unfolding() {
        let analyzer = SpacingAnalyzer::new();
        let eigenvalues = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let unfolded = analyzer.unfold_spectrum(&eigenvalues);
        
        // Mean spacing should be normalized to 1.0
        let mean = unfolded.iter().sum::<f64>() / unfolded.len() as f64;
        assert!((mean - 1.0).abs() < 1e-10);
    }
}
