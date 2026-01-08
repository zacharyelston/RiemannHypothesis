use serde::{Serialize, Deserialize};
use std::fs::File;
use std::io::Write;
use crate::utils::error::RiemannError;

/// Common statistics for spectral analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralStatistics {
    pub mean_spacing: f64,
    pub variance: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub ks_statistic: f64,
    pub ks_pvalue: f64,
}

/// Rigidity metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidityMetrics {
    #[serde(rename = "L")]
    pub l: f64,
    pub number_variance: RigidityValue,
    pub delta_3: RigidityValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RigidityValue {
    pub observed: f64,
    pub gue_predicted: f64,
    pub ratio: f64,
}

/// GUE verification result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GueResult {
    pub command: String,
    pub timestamp: String,
    pub parameters: GueParameters,
    pub eigenvalues: Vec<f64>,
    pub spacings: Vec<f64>,
    pub statistics: SpectralStatistics,
    pub metadata: GueMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GueParameters {
    pub size: usize,
    pub hbar: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GueMetadata {
    pub gue_theory: GueTheory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GueTheory {
    pub mean: f64,
    pub variance: f64,
}

/// Berry-Keating result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BerryKeatingResult {
    pub command: String,
    pub timestamp: String,
    pub parameters: BerryKeatingParameters,
    pub eigenvalues: Vec<f64>,
    pub statistics: BerryKeatingStatistics,
    pub metadata: BerryKeatingMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BerryKeatingParameters {
    pub truncation: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BerryKeatingStatistics {
    pub all_real: bool,
    pub min_eigenvalue: f64,
    pub max_eigenvalue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BerryKeatingMetadata {
    pub theorem: String,
    pub note: String,
}

/// Born oscillator result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BornOscillatorResult {
    pub command: String,
    pub timestamp: String,
    pub parameters: BornOscillatorParameters,
    pub eigenvalues: Vec<f64>,
    pub metadata: BornOscillatorMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BornOscillatorParameters {
    pub lambda: f64,
    pub truncation: usize,
    pub hbar: f64,
    pub order: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BornOscillatorMetadata {
    pub quantization: String,
    pub paper: String,
}

/// Zeta zeros result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZetaZerosResult {
    pub command: String,
    pub timestamp: String,
    pub parameters: ZetaZerosParameters,
    pub zeros: Vec<f64>,
    pub unfolded_levels: Vec<f64>,
    pub spacings: Vec<f64>,
    pub statistics: SpectralStatistics,
    pub rigidity: RigidityMetrics,
    pub metadata: ZetaZerosMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZetaZerosParameters {
    pub data_file: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZetaZerosMetadata {
    pub phenomenon: String,
    pub conclusion: String,
}

/// Write JSON result to file
pub fn write_json<T: Serialize>(result: &T, path: &str) -> Result<(), RiemannError> {
    let json = serde_json::to_string_pretty(result)
        .map_err(|e| RiemannError::SerializationError(format!("JSON serialization failed: {}", e)))?;
    
    let mut file = File::create(path)
        .map_err(|e| RiemannError::SerializationError(format!("Failed to create file {}: {}", path, e)))?;
    
    file.write_all(json.as_bytes())
        .map_err(|e| RiemannError::SerializationError(format!("Failed to write to file {}: {}", path, e)))?;
    
    Ok(())
}

/// Get current timestamp in ISO8601 format
pub fn get_timestamp() -> String {
    chrono::Utc::now().to_rfc3339()
}
