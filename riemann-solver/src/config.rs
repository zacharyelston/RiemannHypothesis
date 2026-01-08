use serde::{Deserialize, Serialize};

/// Configuration for a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub matrix_size: usize,
    pub iterations: usize,
    pub seed: Option<u64>,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            matrix_size: 1000,
            iterations: 100,
            seed: None,
        }
    }
}