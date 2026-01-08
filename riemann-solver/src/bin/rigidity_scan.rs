use riemann_solver::analysis::unfolding::{load_zeros_from_file, unfold_zeros};
use riemann_solver::analysis::spectral::{number_variance, number_variance_gue, delta_3, delta_3_gue};
use std::env;

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: rigidity_scan <zeros_file> [max_count]");
        std::process::exit(1);
    }
    
    let file_path = &args[1];
    let max_count = if args.len() > 2 {
        args[2].parse::<usize>().unwrap_or(100000)
    } else {
        100000
    };
    
    println!("Loading zeros from {}...", file_path);
    let mut zeros = load_zeros_from_file(file_path)?;
    zeros.truncate(max_count);
    
    println!("Unfolding {} zeros...", zeros.len());
    let unfolded = unfold_zeros(&zeros);
    
    println!("\n=== Rigidity Scan: Σ²(L) and Δ₃(L) vs L ===\n");
    println!("L\tΣ²_obs\tΣ²_GUE\tRatio\tΔ₃_obs\tΔ₃_GUE\tRatio");
    println!("---\t------\t------\t-----\t------\t------\t-----");
    
    let l_values = vec![
        1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 15.0, 20.0, 30.0, 50.0, 75.0, 100.0
    ];
    
    for l in l_values {
        let sigma2_obs = number_variance(&unfolded, l);
        let sigma2_gue = number_variance_gue(l);
        let sigma2_ratio = sigma2_obs / sigma2_gue;
        
        let d3_obs = delta_3(&unfolded, l);
        let d3_gue = delta_3_gue(l);
        let d3_ratio = d3_obs / d3_gue;
        
        println!("{:.1}\t{:.4}\t{:.4}\t{:.3}\t{:.4}\t{:.4}\t{:.3}",
                 l, sigma2_obs, sigma2_gue, sigma2_ratio, d3_obs, d3_gue, d3_ratio);
    }
    
    println!("\n=== Interpretation ===");
    println!("Ratio < 1.0: Zeros MORE rigid than GUE (extra correlations)");
    println!("Ratio ≈ 1.0: Matches GUE prediction");
    println!("Ratio > 1.0: Zeros LESS rigid than GUE (weaker correlations)");
    
    Ok(())
}
