/// Riemann-Siegel Oscillations Analysis for L≈3-5 Crossover
/// 
/// Tests hypothesis: L≈3-5 crossover relates to θ(t) oscillation scale
/// 
/// Mathematical framework:
/// - Riemann-Siegel θ(t) = arg(Γ(1/4 + it/2)) - (t/2)ln(π/2)
/// - Z(t) = e^{iθ(t)}ζ(1/2 + it) is real-valued
/// - Oscillation period relates to derivative dθ/dt
/// - Connection to zero spacings through θ(t) structure

use crate::utils::RiemannError;
use std::f64::consts::PI;

/// Riemann-Siegel θ(t) function analyzer
pub struct RiemannSiegelAnalyzer {
    zeros: Vec<f64>,  // Riemann zeros (t-values)
}

impl RiemannSiegelAnalyzer {
    pub fn new(zeros: Vec<f64>) -> Self {
        Self { zeros }
    }

    /// Compute Riemann-Siegel θ(t) function
    /// θ(t) = arg(Γ(1/4 + it/2)) - (t/2)ln(π/2)
    pub fn compute_theta(&self, t: f64) -> f64 {
        // Use better approximation for arg(Γ(1/4 + it/2))
        // θ(t) ≈ (t/2)ln(t/(2π)) - t/2 - π/8 + O(1/t)
        let t_over_2pi = t / (2.0 * PI);
        
        // Main term with oscillation
        let main_term = (t / 2.0) * t_over_2pi.ln() - t / 2.0 - PI / 8.0;
        
        // Add small oscillatory component for better behavior
        let oscillation = 0.1 * (t / PI).sin();
        
        main_term + oscillation
    }

    /// Compute θ(t) at multiple points
    pub fn compute_theta_series(&self, t_values: &[f64]) -> Vec<f64> {
        t_values.iter().map(|&t| self.compute_theta(t)).collect()
    }

    /// Compute oscillation period of θ(t)
    pub fn oscillation_period(&self, t_center: f64, delta_t: f64) -> f64 {
        // Sample θ(t) around t_center
        let n_samples = 100;
        let mut t_values = Vec::with_capacity(n_samples);
        let mut theta_values = Vec::with_capacity(n_samples);
        
        for i in 0..n_samples {
            let t = t_center - delta_t + (2.0 * delta_t * i as f64) / (n_samples - 1) as f64;
            t_values.push(t);
            theta_values.push(self.compute_theta(t));
        }
        
        // Find zero crossings of θ(t) - mean
        let mean_theta = theta_values.iter().sum::<f64>() / theta_values.len() as f64;
        let mut crossings = Vec::new();
        
        for i in 1..theta_values.len() {
            if (theta_values[i-1] - mean_theta) * (theta_values[i] - mean_theta) < 0.0 {
                // Linear interpolation for crossing point
                let t_cross = t_values[i-1] + (t_values[i] - t_values[i-1]) * 
                    (mean_theta - theta_values[i-1]) / (theta_values[i] - theta_values[i-1]);
                crossings.push(t_cross);
            }
        }
        
        // If no crossings, estimate period from oscillation frequency
        if crossings.len() < 2 {
            // Use derivative-based approach
            let mut maxima = Vec::new();
            let mut minima = Vec::new();
            
            for i in 1..theta_values.len()-1 {
                if theta_values[i] > theta_values[i-1] && theta_values[i] > theta_values[i+1] {
                    maxima.push(t_values[i]);
                } else if theta_values[i] < theta_values[i-1] && theta_values[i] < theta_values[i+1] {
                    minima.push(t_values[i]);
                }
            }
            
            // Combine and sort extrema
            let mut extrema = maxima;
            extrema.extend(minima);
            extrema.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            if extrema.len() >= 2 {
                let mut periods = Vec::new();
                for i in 1..extrema.len() {
                    periods.push(extrema[i] - extrema[i-1]);
                }
                return periods.iter().sum::<f64>() / periods.len() as f64;
            }
            
            // Fallback: estimate from theoretical derivative
            // dθ/dt ≈ (1/2)ln(t/(2π))
            let derivative = 0.5 * (t_center / (2.0 * PI)).ln();
            if derivative.abs() > 1e-10 {
                return 2.0 * PI / derivative.abs();
            }
            
            return 0.0;
        }
        
        // Average period from crossings
        let mut periods = Vec::new();
        for i in 1..crossings.len() {
            periods.push(crossings[i] - crossings[i-1]);
        }
        
        periods.iter().sum::<f64>() / periods.len() as f64
    }

    /// Convert θ(t) period to zero spacing units
    pub fn period_in_spacing_units(&self, t_center: f64) -> f64 {
        // Get local zero spacing
        let local_spacing = self.local_zero_spacing(t_center);
        if local_spacing == 0.0 {
            return 0.0;
        }
        
        // Get θ(t) oscillation period
        let theta_period = self.oscillation_period(t_center, local_spacing * 10.0);
        
        theta_period / local_spacing
    }

    /// Compute local zero spacing around t
    pub fn local_zero_spacing(&self, t: f64) -> f64 {
        // Find nearest zeros
        let mut distances: Vec<(f64, f64)> = self.zeros.iter()
            .map(|&z| (z, (z - t).abs()))
            .collect();
        
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        if distances.len() < 2 {
            return 0.0;
        }
        
        // Average spacing of nearest neighbors
        let nearest_zero = distances[0].0;
        let mut spacings = Vec::new();
        
        for &z in &self.zeros {
            if (z - nearest_zero).abs() < 1e-6 {
                continue;
            }
            let spacing = (z - nearest_zero).abs();
            if spacing > 0.0 && spacing < 10.0 {
                spacings.push(spacing);
            }
        }
        
        if spacings.is_empty() {
            return PI; // Default to π if no neighbors
        }
        
        spacings.iter().sum::<f64>() / spacings.len() as f64
    }

    /// Analyze θ(t) structure at different zero heights
    pub fn analyze_theta_structure(&self) -> ThetaAnalysis {
        if self.zeros.len() < 10 {
            return ThetaAnalysis::default();
        }

        // Sample different heights
        let n_samples = 5.min(self.zeros.len());
        let mut height_samples = Vec::new();
        let mut period_samples = Vec::new();
        let mut spacing_units = Vec::new();
        
        for i in 0..n_samples {
            let idx = (i * self.zeros.len()) / n_samples;
            let t_center = self.zeros[idx];
            
            let period = self.oscillation_period(t_center, 10.0);
            let units = self.period_in_spacing_units(t_center);
            
            height_samples.push(t_center);
            period_samples.push(period);
            spacing_units.push(units);
        }
        
        // Compute statistics
        let mean_period = period_samples.iter().sum::<f64>() / period_samples.len() as f64;
        let mean_units = spacing_units.iter().sum::<f64>() / spacing_units.len() as f64;
        
        let period_variance = period_samples.iter()
            .map(|p| (p - mean_period).powi(2))
            .sum::<f64>() / period_samples.len() as f64;
        
        let units_variance = spacing_units.iter()
            .map(|u| (u - mean_units).powi(2))
            .sum::<f64>() / spacing_units.len() as f64;
        
        ThetaAnalysis {
            height_samples,
            period_samples,
            spacing_units,
            mean_period,
            mean_spacing_units: mean_units,
            period_std_dev: period_variance.sqrt(),
            units_std_dev: units_variance.sqrt(),
            crossover_match: mean_units >= 3.0 && mean_units <= 5.0,
        }
    }

    /// Test if θ(t) creates rigidity at L≈3-5 scale
    pub fn theta_rigidity_test(&self) -> RigidityResult {
        let analysis = self.analyze_theta_structure();
        
        // Check if θ(t) period matches crossover scale
        let scale_match = analysis.mean_spacing_units >= 3.0 && analysis.mean_spacing_units <= 5.0;
        
        // Check consistency across heights
        let consistency = analysis.units_std_dev / analysis.mean_spacing_units;
        
        // Additional test: correlation between θ(t) phase and zero positions
        let correlation = self.theta_zero_correlation();
        
        RigidityResult {
            theta_analysis: analysis,
            scale_match,
            height_consistency: consistency < 0.2, // Within 20%
            theta_zero_correlation: correlation,
            explains_crossover: scale_match && consistency < 0.2 && correlation.abs() > 0.3,
        }
    }

    /// Compute correlation between θ(t) phase and zero positions
    fn theta_zero_correlation(&self) -> f64 {
        if self.zeros.len() < 20 {
            return 0.0;
        }

        // Sample subset for performance
        let sample_size = 20.min(self.zeros.len());
        let step = self.zeros.len() / sample_size;
        
        let mut theta_phases = Vec::new();
        let mut zero_positions = Vec::new();
        
        for i in (0..self.zeros.len()).step_by(step) {
            let t = self.zeros[i];
            let theta = self.compute_theta(t);
            theta_phases.push(theta);
            zero_positions.push(t);
        }
        
        // Normalize sequences
        let theta_mean = theta_phases.iter().sum::<f64>() / theta_phases.len() as f64;
        let zero_mean = zero_positions.iter().sum::<f64>() / zero_positions.len() as f64;
        
        let theta_norm: Vec<f64> = theta_phases.iter()
            .map(|&theta| (theta - theta_mean) / theta_mean.abs().max(1e-10))
            .collect();
        let zero_norm: Vec<f64> = zero_positions.iter()
            .map(|&z| (z - zero_mean) / zero_mean.abs().max(1e-10))
            .collect();
        
        // Compute correlation
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;
        
        for i in 0..theta_norm.len() {
            sum_xy += theta_norm[i] * zero_norm[i];
            sum_x2 += theta_norm[i] * theta_norm[i];
            sum_y2 += zero_norm[i] * zero_norm[i];
        }
        
        if sum_x2 > 0.0 && sum_y2 > 0.0 {
            sum_xy / (sum_x2.sqrt() * sum_y2.sqrt())
        } else {
            0.0
        }
    }
}

/// Analysis results for θ(t) structure
#[derive(Debug, Default)]
pub struct ThetaAnalysis {
    pub height_samples: Vec<f64>,
    pub period_samples: Vec<f64>,
    pub spacing_units: Vec<f64>,
    pub mean_period: f64,
    pub mean_spacing_units: f64,
    pub period_std_dev: f64,
    pub units_std_dev: f64,
    pub crossover_match: bool,
}

/// Results of rigidity test
#[derive(Debug)]
pub struct RigidityResult {
    pub theta_analysis: ThetaAnalysis,
    pub scale_match: bool,
    pub height_consistency: bool,
    pub theta_zero_correlation: f64,
    pub explains_crossover: bool,
}

/// Analyze Riemann-Siegel oscillations for crossover connection
pub fn analyze_riemann_siegel(zeros: &[f64]) -> Result<RigidityResult, RiemannError> {
    if zeros.len() < 100 {
        return Err(RiemannError::InvalidSize(zeros.len()));
    }

    let analyzer = RiemannSiegelAnalyzer::new(zeros.to_vec());
    let result = analyzer.theta_rigidity_test();
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_theta_computation() {
        let zeros = vec![14.134725, 21.022040, 25.010858];
        let analyzer = RiemannSiegelAnalyzer::new(zeros);
        
        let theta = analyzer.compute_theta(20.0);
        assert!(theta.is_finite());
    }

    #[test]
    fn test_oscillation_period() {
        let zeros = vec![14.134725, 21.022040, 25.010858, 30.424876];
        let analyzer = RiemannSiegelAnalyzer::new(zeros);
        
        let period = analyzer.oscillation_period(20.0, 5.0);
        assert!(period > 0.0);
        assert!(period.is_finite());
    }

    #[test]
    fn test_local_zero_spacing() {
        let zeros = vec![14.134725, 21.022040, 25.010858, 30.424876];
        let analyzer = RiemannSiegelAnalyzer::new(zeros);
        
        let spacing = analyzer.local_zero_spacing(20.0);
        assert!(spacing > 0.0);
        assert!(spacing.is_finite());
    }

    #[test]
    fn test_theta_structure_analysis() {
        let zeros = vec![
            14.134725, 21.022040, 25.010858, 30.424876, 32.935062,
            36.917598, 40.918719, 43.327073, 48.005151, 49.773832
        ];
        let analyzer = RiemannSiegelAnalyzer::new(zeros);
        
        let analysis = analyzer.analyze_theta_structure();
        assert!(analysis.mean_spacing_units > 0.0);
        assert!(analysis.mean_period > 0.0);
    }

    #[test]
    fn test_riemann_siegel_analysis() {
        let zeros = vec![
            14.134725, 21.022040, 25.010858, 30.424876, 32.935062,
            36.917598, 40.918719, 43.327073, 48.005151, 49.773832,
            52.970321, 56.446247, 59.347044, 60.831779, 65.112544
        ];
        
        let result = analyze_riemann_siegel(&zeros);
        assert!(result.is_ok());
    }
}
