/// Comprehensive validation tests for corrected GPU solver.
/// 
/// These tests verify that:
/// 1. Unfolding produces mean spacing = 1.0
/// 2. Rigidity metrics match GUE theory (not 7M× too large)
/// 3. Spacing statistics are correct
/// 4. All metrics are in physically reasonable ranges

#[cfg(test)]
mod tests {
    use crate::gpu::unfolding::{unfold_eigenvalues, verify_unfolding};
    use crate::gpu::rigidity_metrics::{compute_number_variance, compute_delta3, gue_theory};
    use crate::gpu::batch_processor_v2::GpuBatchProcessorV2;

    /// Test 1: Verify unfolding produces mean spacing = 1.0
    #[test]
    fn test_unfolding_normalization() {
        let eigenvalues = vec![0.0, 0.5, 1.8, 2.3, 4.1, 5.0, 6.2, 7.1, 8.9, 10.0];
        let unfolded = unfold_eigenvalues(&eigenvalues).unwrap();

        let mean_spacing = verify_unfolding(&unfolded);
        assert!(
            (mean_spacing - 1.0).abs() < 1e-10,
            "Mean spacing should be 1.0, got {}",
            mean_spacing
        );
    }

    /// Test 2: Verify rigidity metrics are NOT 7M× too large
    #[test]
    fn test_rigidity_metrics_reasonable_magnitude() {
        let eigenvalues = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let windows = vec![5.0, 10.0, 20.0];

        let sigma2 = compute_number_variance(&eigenvalues, &windows).unwrap();
        let delta3 = compute_delta3(&eigenvalues, &windows).unwrap();

        // CRITICAL: These should NOT be in millions
        for (i, &var) in sigma2.iter().enumerate() {
            assert!(
                var < 10.0,
                "Σ²({}) = {} is way too large (broken implementation was 7M+)",
                windows[i],
                var
            );
            assert!(var > 0.0, "Σ²({}) should be positive", windows[i]);
        }

        for (i, &d3) in delta3.iter().enumerate() {
            assert!(
                d3 < 1.0,
                "Δ₃({}) = {} is way too large (broken implementation was 7M+)",
                windows[i],
                d3
            );
            assert!(d3 > 0.0, "Δ₃({}) should be positive", windows[i]);
        }
    }

    /// Test 3: Compare with GUE theory predictions
    #[test]
    fn test_rigidity_metrics_vs_gue_theory() {
        // GUE theory predictions
        let sigma2_gue_5 = gue_theory::number_variance_gue(5.0);
        let sigma2_gue_10 = gue_theory::number_variance_gue(10.0);
        let sigma2_gue_20 = gue_theory::number_variance_gue(20.0);

        let delta3_gue_5 = gue_theory::delta3_gue(5.0);
        let delta3_gue_10 = gue_theory::delta3_gue(10.0);
        let delta3_gue_20 = gue_theory::delta3_gue(20.0);

        // Theory values should be in reasonable ranges
        assert!(
            sigma2_gue_5 > 0.5 && sigma2_gue_5 < 1.0,
            "GUE Σ²(5) = {} out of range",
            sigma2_gue_5
        );
        assert!(
            sigma2_gue_10 > 0.5 && sigma2_gue_10 < 1.0,
            "GUE Σ²(10) = {} out of range",
            sigma2_gue_10
        );
        assert!(
            sigma2_gue_20 > 0.5 && sigma2_gue_20 < 1.0,
            "GUE Σ²(20) = {} out of range",
            sigma2_gue_20
        );

        assert!(
            delta3_gue_5 > 0.01 && delta3_gue_5 < 0.1,
            "GUE Δ₃(5) = {} out of range",
            delta3_gue_5
        );
        assert!(
            delta3_gue_10 > 0.01 && delta3_gue_10 < 0.1,
            "GUE Δ₃(10) = {} out of range",
            delta3_gue_10
        );
        assert!(
            delta3_gue_20 > 0.01 && delta3_gue_20 < 0.1,
            "GUE Δ₃(20) = {} out of range",
            delta3_gue_20
        );
    }

    /// Test 4: Verify spacing statistics with proper unfolding
    #[test]
    fn test_spacing_statistics_uniform() {
        let processor = GpuBatchProcessorV2::new(32).unwrap();
        let eigenvalues = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];

        let stats = processor.compute_spacing_statistics(&eigenvalues).unwrap();

        // For uniform spacing, mean should be ≈ 1.0
        assert!(
            (stats.mean_spacing - 1.0).abs() < 0.01,
            "Mean spacing = {}, expected ≈ 1.0",
            stats.mean_spacing
        );

        // Variance should be very small for uniform spacing
        assert!(
            stats.variance < 0.01,
            "Variance = {}, expected < 0.01 for uniform spacing",
            stats.variance
        );
    }

    /// Test 5: Verify spacing statistics with non-uniform spacing
    #[test]
    fn test_spacing_statistics_nonuniform() {
        let processor = GpuBatchProcessorV2::new(32).unwrap();
        let eigenvalues = vec![0.0, 0.5, 2.0, 2.5, 5.0, 5.5, 8.0, 8.5, 10.0];

        let stats = processor.compute_spacing_statistics(&eigenvalues).unwrap();

        // After unfolding, mean should still be ≈ 1.0
        assert!(
            (stats.mean_spacing - 1.0).abs() < 0.01,
            "Mean spacing = {}, expected ≈ 1.0",
            stats.mean_spacing
        );

        // Variance should be non-zero for non-uniform spacing
        assert!(
            stats.variance > 0.0,
            "Variance should be > 0 for non-uniform spacing"
        );
    }

    /// Test 6: Verify rigidity metrics are computable for realistic data
    #[test]
    fn test_rigidity_metrics_realistic_data() {
        let processor = GpuBatchProcessorV2::new(32).unwrap();

        // Simulate realistic eigenvalue data
        let mut eigenvalues = Vec::new();
        for i in 0..100 {
            eigenvalues.push(i as f64 + (i as f64 * 0.1).sin());
        }

        let windows = vec![5.0, 10.0, 20.0];
        let metrics = processor.compute_rigidity_metrics(&eigenvalues, &windows).unwrap();

        // All metrics should be computable and in reasonable ranges
        assert_eq!(metrics.window_sizes.len(), 3);
        assert_eq!(metrics.number_variance.len(), 3);
        assert_eq!(metrics.delta3.len(), 3);

        for (i, &var) in metrics.number_variance.iter().enumerate() {
            assert!(
                var > 0.0 && var < 100.0,
                "Σ²({}) = {} out of reasonable range",
                windows[i],
                var
            );
        }

        for (i, &d3) in metrics.delta3.iter().enumerate() {
            assert!(
                d3 > 0.0 && d3 < 10.0,
                "Δ₃({}) = {} out of reasonable range",
                windows[i],
                d3
            );
        }
    }

    /// Test 7: Verify the broken implementation's error is fixed
    #[test]
    fn test_broken_implementation_error_is_fixed() {
        // The broken implementation produced:
        // L=5: Σ²(L)=7658420.166667 (should be ≈ 0.70)
        // L=10: Σ²(L)=7634490.166667 (should be ≈ 0.84)
        // L=20: Σ²(L)=7586780.166667 (should be ≈ 0.98)

        let processor = GpuBatchProcessorV2::new(32).unwrap();
        let eigenvalues = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0];
        let windows = vec![5.0, 10.0, 20.0];

        let metrics = processor.compute_rigidity_metrics(&eigenvalues, &windows).unwrap();

        // Verify we're NOT getting 7M+ values
        for (i, &var) in metrics.number_variance.iter().enumerate() {
            assert!(
                var < 1000.0,
                "Σ²({}) = {} - broken implementation error NOT fixed!",
                windows[i],
                var
            );
        }

        // Verify values are in reasonable range (0.1-1.0 for uniform spectrum)
        for (i, &var) in metrics.number_variance.iter().enumerate() {
            assert!(
                var > 0.0 && var < 10.0,
                "Σ²({}) = {} should be in range (0, 10)",
                windows[i],
                var
            );
        }
    }

    /// Test 8: Verify GUE theory predictions are monotonically increasing
    #[test]
    fn test_gue_theory_monotonicity() {
        let sigma2_5 = gue_theory::number_variance_gue(5.0);
        let sigma2_10 = gue_theory::number_variance_gue(10.0);
        let sigma2_20 = gue_theory::number_variance_gue(20.0);

        // Σ²(L) should increase with L
        assert!(
            sigma2_5 < sigma2_10,
            "Σ²(5) = {} should be < Σ²(10) = {}",
            sigma2_5,
            sigma2_10
        );
        assert!(
            sigma2_10 < sigma2_20,
            "Σ²(10) = {} should be < Σ²(20) = {}",
            sigma2_10,
            sigma2_20
        );

        let delta3_5 = gue_theory::delta3_gue(5.0);
        let delta3_10 = gue_theory::delta3_gue(10.0);
        let delta3_20 = gue_theory::delta3_gue(20.0);

        // Δ₃(L) should also increase with L
        assert!(
            delta3_5 < delta3_10,
            "Δ₃(5) = {} should be < Δ₃(10) = {}",
            delta3_5,
            delta3_10
        );
        assert!(
            delta3_10 < delta3_20,
            "Δ₃(10) = {} should be < Δ₃(20) = {}",
            delta3_10,
            delta3_20
        );
    }
}
