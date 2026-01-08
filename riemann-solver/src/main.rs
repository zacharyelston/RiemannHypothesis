use clap::{Parser, Subcommand};
mod config;
mod hamiltonian;
mod solver;
mod analysis;
mod utils;

use config::SimulationConfig;
use hamiltonian::{QuantumSystem, gue::GueSystem};
use solver::{EigenSolver, lapack::LapackSolver};
use analysis::{SpectrumAnalyzer, spacing::SpacingAnalyzer};

#[derive(Parser)]
#[command(name = "riemann-solver")]
#[command(about = "A scientific solver for the Riemann Hypothesis using spectral geometry")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Run baseline GUE verification
    VerifyGue {
        #[arg(short, long, default_value_t = 300)]
        size: usize,
        #[arg(short, long, default_value_t = 1)]
        iterations: usize,
        #[arg(short, long)]
        seed: Option<u64>,
    },
    /// Run Berry-Keating simulation (Future)
    SimulateBk {
        #[arg(short, long, default_value_t = 50.0)]
        cutoff: f64,
    },
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::VerifyGue { size, iterations, seed } => {
            let config = SimulationConfig {
                matrix_size: size,
                iterations,
                seed,
            };
            tracing::info!("Starting GUE verification with config: {:?}", config);
            
            // Create GUE system
            let gue = GueSystem::new(config.matrix_size, config.seed)?;
            tracing::info!("Generating {}x{} Hermitian matrix...", gue.size(), gue.size());
            
            // Generate Hamiltonian
            let hamiltonian = gue.generate_hamiltonian()?;
            
            // Solve for eigenvalues
            tracing::info!("Computing eigenvalues...");
            let solver = LapackSolver::new();
            let eigenvalues = solver.solve(&hamiltonian)?;
            
            // Analyze spectrum
            tracing::info!("Analyzing spectrum...");
            let analyzer = SpacingAnalyzer::new();
            let unfolded = analyzer.unfold_spectrum(&eigenvalues);
            let stats = analyzer.analyze(&unfolded);
            
            // Display results
            println!("\n=== Analysis Results ===");
            println!("Number of spacings: {}", unfolded.len());
            println!("Mean spacing: {:.4} (Theory: 1.0)", stats.mean_spacing);
            println!("Variance: {:.4} (Theory GUE: ~0.178, Poisson: 1.0)", stats.variance);
            println!("Skewness: {:.4}", stats.skewness);
            println!("Kurtosis: {:.4}", stats.kurtosis);
            println!("GUE Match Confidence: {:.2}%", stats.gue_match_confidence * 100.0);
            
            if stats.gue_match_confidence > 0.8 {
                println!("\n✓ CONCLUSION: Matches GUE statistics (Quantum Chaos / Riemann Zeros)");
            } else {
                println!("\n✗ CONCLUSION: Deviates from GUE (Variance={:.4})", stats.variance);
            }
            
            Ok(())
        }
        Commands::SimulateBk { cutoff } => {
            tracing::info!("Berry-Keating simulation not yet implemented. Cutoff: {}", cutoff);
            println!("Berry-Keating Hamiltonian simulation is planned for future implementation.");
            Ok(())
        }
    }
}
