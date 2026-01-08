use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use riemann_solver::analysis::unfolding::{riemann_von_mangoldt, unfold_zeros};
use riemann_solver::analysis::spectral::{number_variance, number_variance_gue};

#[derive(Parser)]
#[command(name = "energy-bin-rigidity")]
#[command(about = "Analyze rigidity crossover across energy bins")]
struct Args {
    /// Path to zeros file
    #[arg(short, long)]
    zeros: String,

    /// Number of energy bins
    #[arg(short, long, default_value_t = 5)]
    bins: usize,

    /// Zeros per bin
    #[arg(short, long, default_value_t = 10000)]
    per_bin: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("=== Energy-Dependent Rigidity Crossover Analysis ===\n");
    println!("Loading zeros from: {}", args.zeros);
    println!("Energy bins: {}", args.bins);
    println!("Zeros per bin: {}\n", args.per_bin);

    // Load all zeros
    let file = File::open(&args.zeros)?;
    let reader = BufReader::new(file);
    let mut all_zeros: Vec<f64> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Ok(t) = line.parse::<f64>() {
            all_zeros.push(t);
        }
    }

    println!("Loaded {} zeros", all_zeros.len());
    all_zeros.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // Define L values to test
    let l_values = vec![1.0, 2.0, 3.0, 5.0, 7.0, 10.0, 20.0, 50.0, 100.0];

    println!("\n{:<10} {:<15} {:<15} {:<10}", "Bin", "Energy Range", "Crossover L", "Ratio@L=10");
    println!("{}", "-".repeat(60));

    // Process each energy bin
    for bin_idx in 0..args.bins {
        let start_idx = bin_idx * args.per_bin;
        let end_idx = ((bin_idx + 1) * args.per_bin).min(all_zeros.len());

        if start_idx >= all_zeros.len() {
            break;
        }

        let bin_zeros = &all_zeros[start_idx..end_idx];
        
        if bin_zeros.len() < 100 {
            println!("Bin {} too small ({} zeros), skipping", bin_idx + 1, bin_zeros.len());
            continue;
        }

        let energy_min = bin_zeros.first().unwrap();
        let energy_max = bin_zeros.last().unwrap();

        // Unfold zeros using N(T)
        let unfolded = unfold_zeros(bin_zeros);

        // Compute Σ²(L) for each L value
        let mut ratios = Vec::new();
        let mut crossover_l = 0.0;
        let mut found_crossover = false;

        for &l in &l_values {
            let sigma2_obs = number_variance(&unfolded, l);
            let sigma2_gue = number_variance_gue(l);
            let ratio = sigma2_obs / sigma2_gue;
            ratios.push((l, ratio));

            // Detect crossover (ratio crosses 1.0)
            if !found_crossover && ratio < 1.0 {
                crossover_l = l;
                found_crossover = true;
            }
        }

        // Get ratio at L=10
        let ratio_at_10 = ratios.iter()
            .find(|(l, _)| (*l - 10.0).abs() < 0.1)
            .map(|(_, r)| *r)
            .unwrap_or(0.0);

        let crossover_str = if found_crossover {
            format!("{:.1}", crossover_l)
        } else {
            ">100".to_string()
        };

        println!(
            "{:<10} {:<15} {:<15} {:<10.3}",
            format!("Bin {}", bin_idx + 1),
            format!("{:.0}-{:.0}", energy_min, energy_max),
            crossover_str,
            ratio_at_10
        );
    }

    println!("\n=== Detailed Rigidity Curves ===\n");

    // Print detailed curves for first and last bins
    for (bin_label, bin_idx) in [("Low Energy", 0), ("High Energy", args.bins - 1)] {
        let start_idx = bin_idx * args.per_bin;
        let end_idx = ((bin_idx + 1) * args.per_bin).min(all_zeros.len());

        if start_idx >= all_zeros.len() {
            continue;
        }

        let bin_zeros = &all_zeros[start_idx..end_idx];
        
        if bin_zeros.len() < 100 {
            continue;
        }

        let unfolded = unfold_zeros(bin_zeros);

        println!("{} (Bin {}):", bin_label, bin_idx + 1);
        println!("{:<10} {:<12} {:<12} {:<10}", "L", "Σ²(obs)", "Σ²(GUE)", "Ratio");
        println!("{}", "-".repeat(50));

        for &l in &l_values {
            let sigma2_obs = number_variance(&unfolded, l);
            let sigma2_gue = number_variance_gue(l);
            let ratio = sigma2_obs / sigma2_gue;

            println!(
                "{:<10.1} {:<12.4} {:<12.4} {:<10.4}",
                l, sigma2_obs, sigma2_gue, ratio
            );
        }
        println!();
    }

    println!("=== Analysis Complete ===\n");
    println!("Interpretation:");
    println!("- If crossover L is CONSTANT across bins → Universal property");
    println!("- If crossover L VARIES with energy → Energy-dependent mechanism");
    println!("- Ratio < 1.0 means zeros are MORE rigid than GUE");

    Ok(())
}
