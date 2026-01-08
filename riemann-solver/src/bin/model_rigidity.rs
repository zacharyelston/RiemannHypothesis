use riemann_solver::hamiltonian::QuantumSystem;
use riemann_solver::hamiltonian::gue::GueSystem;
use riemann_solver::hamiltonian::berry_keating::BerryKeatingSystem;
use riemann_solver::hamiltonian::born_oscillator::BornOscillator;
use riemann_solver::solver::{EigenSolver, lapack::LapackSolver};
use riemann_solver::analysis::{SpectrumAnalyzer, spacing::SpacingAnalyzer};
use riemann_solver::analysis::spectral::{number_variance, number_variance_gue};
use std::env;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: model_rigidity <model> <size>");
        eprintln!("Models: gue, berry-keating, born");
        std::process::exit(1);
    }
    
    let model = &args[1];
    let size: usize = args[2].parse()?;
    
    println!("Computing eigenvalues for {} (N={})...", model, size);
    
    let eigenvalues: Vec<f64> = match model.as_str() {
        "gue" => {
            let gue = GueSystem::new(size, None)?;
            let hamiltonian = gue.generate_hamiltonian()?;
            let solver = LapackSolver::new();
            solver.solve(&hamiltonian)?
        },
        "berry-keating" => {
            let bk = BerryKeatingSystem::new(size)?;
            let hamiltonian = bk.generate_hamiltonian()?;
            let solver = LapackSolver::new();
            solver.solve(&hamiltonian)?
        },
        "born" => {
            let lambda = 1.0;
            let bo = BornOscillator::new(lambda, size)?;
            // Use semiclassical (order 0) to avoid convergence issues
            bo.compute_eigenvalues_with_order(1.0, 0)?
        },
        _ => {
            eprintln!("Unknown model: {}", model);
            std::process::exit(1);
        }
    };
    
    println!("Got {} eigenvalues", eigenvalues.len());
    println!("Unfolding spectrum...");
    
    let analyzer = SpacingAnalyzer::new();
    let unfolded = analyzer.unfold_spectrum(&eigenvalues);
    
    println!("\n=== Rigidity Scan: {} ===\n", model.to_uppercase());
    println!("L\tΣ²_obs\tΣ²_GUE\tRatio");
    println!("---\t------\t------\t-----");
    
    let l_values = vec![
        1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0, 50.0
    ];
    
    for l in l_values {
        let sigma2_obs = number_variance(&unfolded, l);
        let sigma2_gue = number_variance_gue(l);
        let ratio = sigma2_obs / sigma2_gue;
        
        println!("{:.1}\t{:.4}\t{:.4}\t{:.3}", l, sigma2_obs, sigma2_gue, ratio);
    }
    
    println!("\n=== Interpretation ===");
    println!("Ratio < 1.0: MORE rigid than GUE (extra correlations)");
    println!("Ratio ≈ 1.0: Matches GUE prediction");
    println!("Ratio > 1.0: LESS rigid than GUE (weaker correlations)");
    
    Ok(())
}
