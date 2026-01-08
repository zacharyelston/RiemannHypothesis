use nalgebra::DMatrix;
use nalgebra::Complex;
use crate::utils::RiemannError;

/// Represents a physical or mathematical system that generates a spectrum.
pub trait QuantumSystem {
    /// Generates the Hamiltonian matrix representation.
    /// Returns `Result` to handle dimension/memory errors.
    fn generate_hamiltonian(&self) -> Result<DMatrix<Complex<f64>>, RiemannError>;
    
    /// Returns the system size/dimension.
    fn size(&self) -> usize;
}

pub mod gue;
pub mod berry_keating;
