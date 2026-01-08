/// Prime Gap Analysis for L≈3-5 Crossover Investigation
/// 
/// Tests hypothesis: L≈3-5 crossover relates to typical prime gap scale
/// 
/// Mathematical framework:
/// - Prime gaps: g_n = p_{n+1} - p_n
/// - Prime density near x: ρ(x) ≈ 1/ln(x)
/// - Expected gap: ⟨g⟩ ≈ ln(x)
/// - Gap distribution: P(g) follows Poisson-like statistics

use crate::utils::RiemannError;
use std::f64::consts::PI;

/// Prime gap analyzer
pub struct PrimeGapAnalyzer {
    primes: Vec<u64>,  // Prime numbers
}

impl PrimeGapAnalyzer {
    pub fn new(primes: Vec<u64>) -> Self {
        Self { primes }
    }

    /// Compute prime gaps: g_n = p_{n+1} - p_n
    pub fn compute_gaps(&self) -> Vec<u64> {
        let mut gaps = Vec::new();
        for i in 0..self.primes.len() - 1 {
            gaps.push(self.primes[i + 1] - self.primes[i]);
        }
        gaps
    }

    /// Compute gap distribution statistics
    pub fn gap_statistics(&self) -> GapStats {
        let gaps = self.compute_gaps();
        
        if gaps.is_empty() {
            return GapStats::default();
        }

        let mean_gap = gaps.iter().sum::<u64>() as f64 / gaps.len() as f64;
        let variance = gaps.iter()
            .map(|g| (*g as f64 - mean_gap).powi(2))
            .sum::<f64>() / gaps.len() as f64;
        let std_dev = variance.sqrt();

        // Find most common gap (mode)
        let mut gap_counts = std::collections::HashMap::new();
        for &gap in &gaps {
            *gap_counts.entry(gap).or_insert(0) += 1;
        }
        let mode_gap = gap_counts.iter()
            .max_by_key(|(_, &count)| count)
            .map(|(&gap, _)| gap)
            .unwrap_or(0);

        // Compute percentiles
        let mut sorted_gaps = gaps.clone();
        sorted_gaps.sort_unstable();
        let p10 = sorted_gaps[(sorted_gaps.len() as f64 * 0.1) as usize];
        let p50 = sorted_gaps[(sorted_gaps.len() as f64 * 0.5) as usize];
        let p90 = sorted_gaps[(sorted_gaps.len() as f64 * 0.9) as usize];

        GapStats {
            count: gaps.len(),
            mean_gap,
            std_dev,
            mode_gap: mode_gap as f64,
            p10: p10 as f64,
            p50: p50 as f64,
            p90: p90 as f64,
        }
    }

    /// Analyze gaps near specific zero height
    pub fn gaps_near_zero(&self, zero_height: f64) -> GapStats {
        // Find primes near the scale of the zero height
        // Use the approximation: nth prime ≈ n ln(n)
        let target_n = (zero_height / zero_height.ln()) as usize;
        let range = (target_n as f64 * 0.1) as usize; // ±10% range
        
        let start = (target_n.saturating_sub(range)).max(0);
        let end = (target_n + range).min(self.primes.len());
        
        if end <= start {
            return self.gap_statistics();
        }

        let subset_primes = self.primes[start..end].to_vec();
        let analyzer = PrimeGapAnalyzer::new(subset_primes);
        analyzer.gap_statistics()
    }

    /// Compute correlation between gaps and zero spacings
    pub fn zero_gap_correlation(&self, zeros: &[f64]) -> f64 {
        if zeros.len() < 10 || self.primes.len() < 10 {
            return 0.0;
        }

        // Compute zero spacings
        let mut zero_spacings = Vec::new();
        for i in 0..zeros.len() - 1 {
            zero_spacings.push(zeros[i + 1] - zeros[i]);
        }

        // Compute prime gaps
        let gaps = self.compute_gaps();
        
        // Normalize both sequences
        let zero_mean = zero_spacings.iter().sum::<f64>() / zero_spacings.len() as f64;
        let gap_mean = gaps.iter().sum::<u64>() as f64 / gaps.len() as f64;
        
        let zero_normalized: Vec<f64> = zero_spacings.iter()
            .map(|s| (s - zero_mean) / zero_mean)
            .collect();
        let gap_normalized: Vec<f64> = gaps.iter()
            .map(|g| (*g as f64 - gap_mean) / gap_mean)
            .collect();

        // Compute correlation coefficient
        let n = zero_normalized.len().min(gap_normalized.len());
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;

        for i in 0..n {
            sum_xy += zero_normalized[i] * gap_normalized[i];
            sum_x2 += zero_normalized[i] * zero_normalized[i];
            sum_y2 += gap_normalized[i] * gap_normalized[i];
        }

        if sum_x2 > 0.0 && sum_y2 > 0.0 {
            sum_xy / (sum_x2.sqrt() * sum_y2.sqrt())
        } else {
            0.0
        }
    }

    /// Estimate "prime gap scale" in units of mean zero spacing
    pub fn prime_gap_scale(&self, zeros: &[f64]) -> f64 {
        if zeros.is_empty() || self.primes.is_empty() {
            return 0.0;
        }

        // Get typical zero spacing (should be ~π after unfolding)
        let zero_spacings: Vec<f64> = zeros.iter().zip(zeros.iter().skip(1))
            .map(|(z1, z2)| z2 - z1)
            .collect();
        let mean_zero_spacing = zero_spacings.iter().sum::<f64>() / zero_spacings.len() as f64;

        // Get typical prime gap
        let gap_stats = self.gap_statistics();
        let typical_prime_gap = gap_stats.p50; // Use median as typical

        // Convert prime gap to zero spacing units
        // Rough approximation: zero density ~ (1/2π) * ln(T/2π)
        let avg_zero = zeros.iter().sum::<f64>() / zeros.len() as f64;
        let zero_density = (1.0 / (2.0 * PI)) * avg_zero.ln();
        
        typical_prime_gap * zero_density / mean_zero_spacing
    }
}

/// Statistics for prime gaps
#[derive(Debug, Default)]
pub struct GapStats {
    pub count: usize,
    pub mean_gap: f64,
    pub std_dev: f64,
    pub mode_gap: f64,
    pub p10: f64,  // 10th percentile
    pub p50: f64,  // 50th percentile (median)
    pub p90: f64,  // 90th percentile
}

/// Generate primes up to limit using sieve of Eratosthenes
pub fn generate_primes(limit: u64) -> Vec<u64> {
    if limit < 2 {
        return Vec::new();
    }

    let mut sieve = vec![true; (limit + 1) as usize];
    sieve[0] = false;
    sieve[1] = false;

    for p in 2..=(limit as f64).sqrt() as usize {
        if sieve[p] {
            for multiple in (p * p..=limit as usize).step_by(p) {
                sieve[multiple] = false;
            }
        }
    }

    (2..=limit as usize)
        .filter(|&i| sieve[i])
        .map(|i| i as u64)
        .collect()
}

/// Analyze prime gap connection to L≈3-5 crossover
pub fn analyze_prime_gap_connection(zeros: &[f64]) -> Result<PrimeGapResult, RiemannError> {
    if zeros.len() < 100 {
        return Err(RiemannError::InvalidSize(zeros.len()));
    }

    // Generate primes up to a reasonable limit
    // Use approximation: nth zero ~ (n/2π) * ln(n/2π)
    let max_zero = zeros.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let prime_limit = (max_zero * 2.0 * PI) as u64;
    
    let primes = generate_primes(prime_limit.min(1_000_000)); // Cap at 1M for performance
    let analyzer = PrimeGapAnalyzer::new(primes);

    // Get gap statistics
    let overall_stats = analyzer.gap_statistics();
    
    // Analyze gaps at different zero heights
    let low_height = zeros[zeros.len() / 4]; // 25th percentile
    let mid_height = zeros[zeros.len() / 2]; // 50th percentile
    let high_height = zeros[3 * zeros.len() / 4]; // 75th percentile

    let low_stats = analyzer.gaps_near_zero(low_height);
    let mid_stats = analyzer.gaps_near_zero(mid_height);
    let high_stats = analyzer.gaps_near_zero(high_height);

    // Compute prime gap scale
    let prime_gap_scale = analyzer.prime_gap_scale(zeros);
    
    // Compute correlation
    let correlation = analyzer.zero_gap_correlation(zeros);

    Ok(PrimeGapResult {
        overall_stats,
        low_height_stats: (low_height, low_stats),
        mid_height_stats: (mid_height, mid_stats),
        high_height_stats: (high_height, high_stats),
        prime_gap_scale,
        zero_gap_correlation: correlation,
        crossover_match: prime_gap_scale >= 3.0 && prime_gap_scale <= 5.0,
    })
}

/// Results of prime gap analysis
#[derive(Debug)]
pub struct PrimeGapResult {
    pub overall_stats: GapStats,
    pub low_height_stats: (f64, GapStats),
    pub mid_height_stats: (f64, GapStats),
    pub high_height_stats: (f64, GapStats),
    pub prime_gap_scale: f64,
    pub zero_gap_correlation: f64,
    pub crossover_match: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prime_generation() {
        let primes = generate_primes(100);
        assert_eq!(primes, vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67, 71, 73, 79, 83, 89, 97]);
    }

    #[test]
    fn test_gap_computation() {
        let primes = vec![2, 3, 5, 7, 11, 13];
        let analyzer = PrimeGapAnalyzer::new(primes);
        let gaps = analyzer.compute_gaps();
        assert_eq!(gaps, vec![1, 2, 2, 4, 2]);
    }

    #[test]
    fn test_gap_statistics() {
        let primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        let analyzer = PrimeGapAnalyzer::new(primes);
        let stats = analyzer.gap_statistics();
        
        assert!(stats.mean_gap > 0.0);
        assert!(stats.std_dev > 0.0);
        assert_eq!(stats.count, 9);
    }

    #[test]
    fn test_prime_gap_scale() {
        let primes = generate_primes(1000);
        let analyzer = PrimeGapAnalyzer::new(primes);
        
        // Mock zeros (should be roughly π spacing)
        let zeros: Vec<f64> = (0..100).map(|i| i as f64 * PI).collect();
        
        let scale = analyzer.prime_gap_scale(&zeros);
        assert!(scale > 0.0);
    }
}
