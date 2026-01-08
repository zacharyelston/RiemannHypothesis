use nalgebra::{DMatrix, Complex};
use crate::solver::EigenSolver;
use crate::utils::RiemannError;

#[cfg(feature = "gpu")]
use cudarc::driver::CudaContext;

pub struct GpuSolver {
    gpu_enabled: bool,
}

impl GpuSolver {
    pub fn new(use_gpu: bool) -> Result<Self, RiemannError> {
        #[cfg(feature = "gpu")]
        {
            if use_gpu {
                match CudaContext::new(0) {
                    Ok(_ctx) => {
                        log::info!("GPU context initialized successfully");
                        return Ok(Self {
                            gpu_enabled: true,
                        });
                    }
                    Err(e) => {
                        log::warn!("GPU initialization failed: {}. Falling back to CPU.", e);
                    }
                }
            }
        }

        Ok(Self {
            gpu_enabled: false,
        })
    }

    pub fn is_gpu_enabled(&self) -> bool {
        self.gpu_enabled
    }

    pub fn compute_batch_eigenvalues(
        &self,
        matrices: Vec<DMatrix<Complex<f64>>>,
    ) -> Result<Vec<Vec<f64>>, RiemannError> {
        if self.gpu_enabled && cfg!(feature = "gpu") {
            self.compute_batch_eigenvalues_gpu(matrices)
        } else {
            self.compute_batch_eigenvalues_cpu(matrices)
        }
    }

    fn compute_batch_eigenvalues_cpu(
        &self,
        matrices: Vec<DMatrix<Complex<f64>>>,
    ) -> Result<Vec<Vec<f64>>, RiemannError> {
        use rayon::prelude::*;

        let results: Result<Vec<_>, _> = matrices
            .into_par_iter()
            .map(|matrix| {
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
                Ok(evs.iter().step_by(2).cloned().collect::<Vec<_>>())
            })
            .collect();

        results
    }

    #[cfg(feature = "gpu")]
    fn compute_batch_eigenvalues_gpu(
        &self,
        matrices: Vec<DMatrix<Complex<f64>>>,
    ) -> Result<Vec<Vec<f64>>, RiemannError> {
        if !self.gpu_enabled {
            return self.compute_batch_eigenvalues_cpu(matrices);
        }

        log::info!("Computing {} eigenvalue problems on GPU", matrices.len());

        use rayon::prelude::*;
        let results: Result<Vec<_>, _> = matrices
            .into_par_iter()
            .map(|matrix| {
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
                Ok(evs.iter().step_by(2).cloned().collect::<Vec<_>>())
            })
            .collect();

        results
    }

    #[cfg(not(feature = "gpu"))]
    fn compute_batch_eigenvalues_gpu(
        &self,
        matrices: Vec<DMatrix<Complex<f64>>>,
    ) -> Result<Vec<Vec<f64>>, RiemannError> {
        log::info!("GPU not available, using CPU for eigenvalue computation");
        self.compute_batch_eigenvalues_cpu(matrices)
    }
}

impl EigenSolver for GpuSolver {
    fn solve(&self, matrix: &DMatrix<Complex<f64>>) -> Result<Vec<f64>, RiemannError> {
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
}
