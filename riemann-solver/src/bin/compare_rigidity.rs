use riemann_solver::hamiltonian::QuantumSystem;
use riemann_solver::hamiltonian::gue::GueSystem;
use riemann_solver::hamiltonian::berry_keating::BerryKeatingSystem;
use riemann_solver::solver::{EigenSolver, lapack::LapackSolver};
use riemann_solver::analysis::unfolding::{load_zeros_from_file, unfold_zeros};
use riemann_solver::analysis::spectral::{number_variance, number_variance_gue};

fn main() -> anyhow::Result<()> {
    println!("=== RIGIDITY COMPARISON: Zeros vs Models ===\n");
    
    // 1. Load and unfold zeros
    println!("Loading 10,000 Riemann zeros...");
    let zeros_file = "../data/zeros1_100k.txt";
    let mut zeros = load_zeros_from_file(zeros_file)?;
    zeros.truncate(10000);
    let zeros_unfolded = unfold_zeros(&zeros);
    println!("Zeros unfolded: {} levels\n", zeros_unfolded.len());
    
    // 2. Generate GUE eigenvalues
    println!("Generating GUE system (N=500)...");
    let gue = GueSystem::new(500, Some(42))?;
    let gue_hamiltonian = gue.generate_hamiltonian()?;
    let solver = LapackSolver::new();
    let mut gue_evals = solver.solve(&gue_hamiltonian)?;
    gue_evals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    // Scale to mean spacing = 1
    let gue_range = gue_evals[gue_evals.len()-1] - gue_evals[0];
    let gue_mean_spacing = gue_range / (gue_evals.len() - 1) as f64;
    let gue_unfolded: Vec<f64> = gue_evals.iter()
        .map(|&e| (e - gue_evals[0]) / gue_mean_spacing)
        .collect();
    println!("GUE eigenvalues: {} (range: {:.3})\n", gue_evals.len(), gue_range);
    
    // 3. Generate Berry-Keating eigenvalues
    println!("Generating Berry-Keating system (N=500)...");
    let bk = BerryKeatingSystem::new(500)?;
    let bk_hamiltonian = bk.generate_hamiltonian()?;
    let mut bk_evals = solver.solve(&bk_hamiltonian)?;
    bk_evals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let bk_range = bk_evals[bk_evals.len()-1] - bk_evals[0];
    let bk_mean_spacing = bk_range / (bk_evals.len() - 1) as f64;
    let bk_unfolded: Vec<f64> = bk_evals.iter()
        .map(|&e| (e - bk_evals[0]) / bk_mean_spacing)
        .collect();
    println!("Berry-Keating eigenvalues: {} (range: {:.3})\n", bk_evals.len(), bk_range);
    
    // 4. Compute rigidity for all systems
    let l_values = vec![1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0, 50.0];
    
    println!("L\tZeros\tGUE\tBK\tGUE_theory");
    println!("---\t-----\t-----\t-----\t----------");
    
    for l in l_values {
        let sigma2_zeros = number_variance(&zeros_unfolded, l);
        let sigma2_gue = number_variance(&gue_unfolded, l);
        let sigma2_bk = number_variance(&bk_unfolded, l);
        let sigma2_theory = number_variance_gue(l);
        
        let ratio_zeros = sigma2_zeros / sigma2_theory;
        let ratio_gue = sigma2_gue / sigma2_theory;
        let ratio_bk = sigma2_bk / sigma2_theory;
        
        println!("{:.1}\t{:.3}\t{:.3}\t{:.3}\t{:.3}",
                 l, ratio_zeros, ratio_gue, ratio_bk, 1.0);
    }
    
    println!("\n=== Interpretation ===");
    println!("Ratio < 1.0: MORE rigid than GUE (extra correlations)");
    println!("Ratio ≈ 1.0: Matches GUE prediction");
    println!("Ratio > 1.0: LESS rigid than GUE (weaker correlations)");
    
    println!("\n=== Key Question ===");
    println!("Does GUE ratio ≈ 1.0 for all L? (Should be, it's the baseline)");
    println!("Does BK show same crossover as Zeros at L≈3-5?");
    println!("If YES → Models capture arithmetic structure");
    println!("If NO → Zeros have unique property models miss");
    
    Ok(())
}
