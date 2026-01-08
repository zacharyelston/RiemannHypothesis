use nalgebra::{DMatrix, Complex};
use crate::utils::RiemannError;

#[cfg(feature = "gpu")]
use cudarc::driver::CudaContext;

pub struct GpuBatchProcessor {
    batch_size: usize,
    gpu_enabled: bool,
}

impl GpuBatchProcessor {
    pub fn new(batch_size: usize) -> Result<Self, RiemannError> {
        #[cfg(feature = "gpu")]
        {
            match CudaContext::new(0) {
                Ok(_ctx) => {
                    log::info!("GPU batch processor initialized with batch size: {}", batch_size);
                    return Ok(Self {
                        batch_size,
                        gpu_enabled: true,
                    });
                }
                Err(e) => {
                    log::warn!("GPU initialization failed: {}. Using CPU batch processing.", e);
                }
            }
        }

        Ok(Self {
            batch_size,
            gpu_enabled: false,
        })
    }

    pub fn is_gpu_enabled(&self) -> bool {
        self.gpu_enabled
    }

    pub fn batch_size(&self) -> usize {
        self.batch_size
    }

    pub fn process_batch(
        &self,
        matrices: Vec<DMatrix<Complex<f64>>>,
    ) -> Result<Vec<Vec<f64>>, RiemannError> {
        use rayon::prelude::*;

        log::info!("Processing batch of {} matrices", matrices.len());

        let results: Result<Vec<_>, _> = matrices
            .into_par_iter()
            .map(|matrix| self.solve_single(&matrix))
            .collect();

        results
    }

    fn solve_single(&self, matrix: &DMatrix<Complex<f64>>) -> Result<Vec<f64>, RiemannError> {
        let n = matrix.nrows();
        let mut big_mat = DMatrix::zeros(2 * n, 2 * n);

        for i in 0..n {
            for j in 0..n {
                let c = matrix[(i, j)];
                big_mat[(i, j)] = c.re;
                big_mat[(i + n, j + n)] = c.re;
                big_mat[(i + n, j)] = c.im;
                big_mat[(i, j + n)] = -c.im;
            }
        }

        let eig = big_mat.symmetric_eigen();
        let mut evs = eig.eigenvalues.as_slice().to_vec();
        evs.sort_by(|a, b| a.partial_cmp(b).unwrap());

        Ok(evs.iter().step_by(2).cloned().collect())
    }

    pub fn compute_spacing_statistics(
        &self,
        eigenvalues: &[f64],
    ) -> Result<SpacingStats, RiemannError> {
        if eigenvalues.len() < 2 {
            return Ok(SpacingStats::default());
        }

        let mut sorted = eigenvalues.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let mut spacings = Vec::new();
        for i in 0..sorted.len() - 1 {
            let spacing = sorted[i + 1] - sorted[i];
            if spacing > 0.0 {
                spacings.push(spacing);
            }
        }

        if spacings.is_empty() {
            return Ok(SpacingStats::default());
        }

        let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
        let normalized: Vec<f64> = spacings.iter().map(|s| s / mean_spacing).collect();

        let n = normalized.len() as f64;
        let mean = normalized.iter().sum::<f64>() / n;
        let variance = normalized.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / n;

        Ok(SpacingStats {
            mean_spacing: mean,
            variance,
            count: normalized.len(),
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct SpacingStats {
    pub mean_spacing: f64,
    pub variance: f64,
    pub count: usize,
}
