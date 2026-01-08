#[cfg(feature = "gpu")]
use cudarc::driver::CudaContext;

pub struct GpuKernels;

impl GpuKernels {
    pub fn new() -> Self {
        #[cfg(feature = "gpu")]
        {
            if let Ok(_ctx) = CudaContext::new(0) {
                return Self;
            }
        }

        Self
    }

    pub fn compute_spacing_histogram(
        &self,
        spacings: &[f64],
        bins: usize,
    ) -> Vec<usize> {
        if spacings.is_empty() {
            return vec![0; bins];
        }

        let max_spacing = spacings.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_spacing = spacings.iter().cloned().fold(f64::INFINITY, f64::min);
        let bin_width = (max_spacing - min_spacing) / bins as f64;

        let mut histogram = vec![0; bins];

        for &spacing in spacings {
            if spacing >= min_spacing && spacing <= max_spacing {
                let bin_idx = ((spacing - min_spacing) / bin_width).floor() as usize;
                let bin_idx = bin_idx.min(bins - 1);
                histogram[bin_idx] += 1;
            }
        }

        histogram
    }

    pub fn compute_number_variance(
        &self,
        eigenvalues: &[f64],
        window_sizes: &[f64],
    ) -> Vec<f64> {
        // CRITICAL FIX: Unfold eigenvalues before computing variance
        // Previous implementation used raw eigenvalues, causing 7M× error
        
        let mut sorted = eigenvalues.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Step 1: Compute spacings
        let mut spacings = Vec::new();
        for i in 0..sorted.len() - 1 {
            let spacing = sorted[i + 1] - sorted[i];
            if spacing > 0.0 {
                spacings.push(spacing);
            }
        }

        if spacings.is_empty() {
            return vec![0.0; window_sizes.len()];
        }

        // Step 2: Normalize spacings to mean = 1.0
        let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
        let normalized_spacings: Vec<f64> = spacings.iter().map(|s| s / mean_spacing).collect();

        // Step 3: Compute unfolded levels (cumulative sum)
        let mut unfolded = vec![0.0];
        let mut cumsum = 0.0;
        for &s in &normalized_spacings {
            cumsum += s;
            unfolded.push(cumsum);
        }

        // Step 4: Compute Σ²(L) on unfolded levels
        window_sizes
            .iter()
            .map(|&L| {
                let mut variance_sum = 0.0;
                let mut count = 0;

                for i in 0..unfolded.len() {
                    let window_start = unfolded[i];
                    let window_end = window_start + L;

                    let count_in_window = unfolded
                        .iter()
                        .filter(|&&x| x >= window_start && x < window_end)
                        .count() as f64;

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
            .collect()
    }

    pub fn compute_delta3(
        &self,
        eigenvalues: &[f64],
        window_sizes: &[f64],
    ) -> Vec<f64> {
        // CRITICAL FIX: Unfold eigenvalues before computing delta3
        // Previous implementation used raw eigenvalues, causing 7M× error
        
        let mut sorted = eigenvalues.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Step 1: Compute spacings
        let mut spacings = Vec::new();
        for i in 0..sorted.len() - 1 {
            let spacing = sorted[i + 1] - sorted[i];
            if spacing > 0.0 {
                spacings.push(spacing);
            }
        }

        if spacings.is_empty() {
            return vec![0.0; window_sizes.len()];
        }

        // Step 2: Normalize spacings to mean = 1.0
        let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
        let normalized_spacings: Vec<f64> = spacings.iter().map(|s| s / mean_spacing).collect();

        // Step 3: Compute unfolded levels (cumulative sum)
        let mut unfolded = vec![0.0];
        let mut cumsum = 0.0;
        for &s in &normalized_spacings {
            cumsum += s;
            unfolded.push(cumsum);
        }

        // Step 4: Compute Δ₃(L) on unfolded levels
        window_sizes
            .iter()
            .map(|&L| {
                let mut min_deviation = f64::INFINITY;

                for start_idx in 0..unfolded.len() {
                    let window_start = unfolded[start_idx];
                    let window_end = window_start + L;

                    let mut sum_x = 0.0;
                    let mut sum_x2 = 0.0;
                    let mut sum_n = 0.0;
                    let mut count = 0;

                    for (i, &x) in unfolded.iter().enumerate() {
                        if x >= window_start && x < window_end {
                            sum_x += x;
                            sum_x2 += x * x;
                            sum_n += i as f64;
                            count += 1;
                        }
                    }

                    if count > 1 {
                        let n_mean = sum_n / count as f64;
                        let x_mean = sum_x / count as f64;

                        let slope = if sum_x2 - sum_x * sum_x / count as f64 > 1e-10 {
                            (sum_n * sum_x - count as f64 * sum_x * n_mean) / (sum_x2 - sum_x * sum_x / count as f64)
                        } else {
                            0.0
                        };

                        let intercept = n_mean - slope * x_mean;

                        let mut deviation_sum = 0.0;
                        for (i, &x) in unfolded.iter().enumerate() {
                            if x >= window_start && x < window_end {
                                let predicted = slope * x + intercept;
                                let actual = i as f64;
                                deviation_sum += (actual - predicted).powi(2);
                            }
                        }

                        let delta3_val = deviation_sum / L;
                        min_deviation = min_deviation.min(delta3_val);
                    }
                }

                if min_deviation.is_infinite() {
                    0.0
                } else {
                    min_deviation
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spacing_histogram() {
        let kernels = GpuKernels::new();
        let spacings = vec![0.5, 1.0, 1.5, 2.0, 2.5];
        let histogram = kernels.compute_spacing_histogram(&spacings, 5);
        assert_eq!(histogram.len(), 5);
        assert!(histogram.iter().sum::<usize>() > 0);
    }

    #[test]
    fn test_number_variance() {
        let kernels = GpuKernels::new();
        let eigenvalues = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let windows = vec![1.0, 2.0];
        let variance = kernels.compute_number_variance(&eigenvalues, &windows);
        assert_eq!(variance.len(), 2);
    }
}
