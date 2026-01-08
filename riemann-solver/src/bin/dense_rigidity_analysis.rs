/// Dense L scanning analysis for rigorous crossover detection
/// 
/// Addresses GPT-2.1 concerns:
/// 1. Dense L scan instead of sparse grid
/// 2. Interpolation for precise L* finding
/// 3. Bootstrap confidence intervals
/// 4. Both Σ²(L) and Δ₃(L) metrics

use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use riemann_solver::analysis::spectral::{number_variance, dyson_mehta, number_variance_gue};
use riemann_solver::analysis::unfolding;

#[derive(Parser)]
#[command(name = "dense_rigidity_analysis")]
#[command(about = "Dense L scanning for rigorous crossover detection")]
struct Args {
    /// File containing Riemann zeros (one per line)
    zeros_file: String,
    
    /// Maximum number of zeros to analyze
    #[arg(short, long, default_value_t = 10000)]
    max_zeros: usize,
    
    /// L range start
    #[arg(long, default_value_t = 1.0)]
    l_min: f64,
    
    /// L range end
    #[arg(long, default_value_t = 6.0)]
    l_max: f64,
    
    /// L step size
    #[arg(long, default_value_t = 0.05)]
    l_step: f64,
    
    /// Number of bootstrap samples
    #[arg(long, default_value_t = 100)]
    bootstrap_samples: usize,
    
    /// Output results to file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Dense Rigidity Analysis ===");
    println!("Addressing GPT-2.1 scientific rigor concerns");
    println!("Loading zeros from: {}", args.zeros_file);
    
    // Load zeros from file
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    if zeros.len() < 1000 {
        eprintln!("Error: Need at least 1000 zeros for dense analysis");
        std::process::exit(1);
    }
    
    // Generate dense L grid
    let l_values: Vec<f64> = (0..)
        .map(|i| args.l_min + i as f64 * args.l_step)
        .take_while(|&l| l <= args.l_max)
        .collect();
    
    println!("Dense L grid: {} points from {:.2} to {:.2}", l_values.len(), args.l_min, args.l_max);
    
    // Unfold zeros
    println!("Unfolding zeros...");
    let unfolded = unfolding(&zeros);
    
    // Compute dense Σ²(L) and Δ₃(L)
    println!("Computing dense rigidity metrics...");
    let sigma2_results: Vec<f64> = l_values.iter().map(|&l| number_variance(&unfolded, l)).collect();
    let delta3_results: Vec<f64> = l_values.iter().map(|&l| dyson_mehta(&unfolded, l)).collect();
    
    // Bootstrap for confidence intervals
    println!("Computing bootstrap confidence intervals...");
    let sigma2_bootstrap = bootstrap_rigidity(&unfolded, &l_values, args.bootstrap_samples, 
                                             |data, l_vals| l_vals.iter().map(|&l| number_variance(data, l)).collect());
    let delta3_bootstrap = bootstrap_rigidity(&unfolded, &l_values, args.bootstrap_samples,
                                             |data, l_vals| l_vals.iter().map(|&l| dyson_mehta(data, l)).collect());
    
    // Find crossover points with interpolation
    println!("Finding crossover points...");
    let sigma2_crossover = find_crossover_point(&l_values, &sigma2_results, &sigma2_bootstrap);
    let delta3_crossover = find_crossover_point(&l_values, &delta3_results, &delta3_bootstrap);
    
    // Print results
    print_results(&l_values, &sigma2_results, &delta3_results, 
                  &sigma2_crossover, &delta3_crossover);
    
    // Save results if requested
    if let Some(output_file) = &args.output {
        save_results(&args.zeros_file, &l_values, &sigma2_results, &delta3_results,
                     &sigma2_crossover, &delta3_crossover, output_file)?;
        println!("\nResults saved to: {}", output_file);
    }
    
    Ok(())
}

fn load_zeros(filename: &str, max_zeros: usize) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = io::BufReader::new(file);
    
    let mut zeros = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if let Ok(zero) = line.trim().parse::<f64>() {
            zeros.push(zero);
            if zeros.len() >= max_zeros {
                break;
            }
        }
    }
    
    Ok(zeros)
}

fn bootstrap_rigidity(
    unfolded: &[f64], 
    l_values: &[f64], 
    n_samples: usize,
    compute_fn: impl Fn(&[f64], &[f64]) -> Vec<f64>
) -> Vec<(f64, f64, f64)>
{
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    let mut results = Vec::new();
    
    for &l in l_values {
        let mut bootstrap_values = Vec::new();
        
        for _ in 0..n_samples {
            // Bootstrap sample with replacement
            let mut sample = Vec::with_capacity(unfolded.len());
            for _ in 0..unfolded.len() {
                let idx = rng.gen_range(0..unfolded.len());
                sample.push(unfolded[idx]);
            }
            
            let sample_results = compute_fn(&sample, &[l]);
            bootstrap_values.push(sample_results[0]);
        }
        
        bootstrap_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = bootstrap_values.iter().sum::<f64>() / bootstrap_values.len() as f64;
        let lower = bootstrap_values[(bootstrap_values.len() as f64 * 0.025) as usize];
        let upper = bootstrap_values[(bootstrap_values.len() as f64 * 0.975) as usize];
        
        results.push((mean, lower, upper));
    }
    
    results
}

fn find_crossover_point(
    l_values: &[f64], 
    results: &[(f64, f64, f64)],
    bootstrap: &[(f64, f64, f64)]
) -> CrossoverResult {
    // Find where ratio crosses 1.0 (assuming results are ratios to GUE theory)
    let mut crossings = Vec::new();
    
    for i in 1..l_values.len() {
        let ratio_prev = results[i-1].0;
        let ratio_curr = results[i].0;
        
        if (ratio_prev - 1.0) * (ratio_curr - 1.0) < 0.0 {
            // Linear interpolation
            let l_cross = l_values[i-1] + (l_values[i] - l_values[i-1]) * 
                (1.0 - ratio_prev) / (ratio_curr - ratio_prev);
            
            // Interpolate confidence bounds
            let lower_prev = bootstrap[i-1].1;
            let upper_prev = bootstrap[i-1].2;
            let lower_curr = bootstrap[i].1;
            let upper_curr = bootstrap[i].2;
            
            let lower_cross = if lower_curr != lower_prev {
                lower_prev + (lower_curr - lower_prev) * (1.0 - ratio_prev) / (ratio_curr - ratio_prev)
            } else {
                l_cross
            };
            
            let upper_cross = if upper_curr != upper_prev {
                upper_prev + (upper_curr - upper_prev) * (1.0 - ratio_prev) / (ratio_curr - ratio_prev)
            } else {
                l_cross
            };
            
            crossings.push((l_cross, lower_cross, upper_cross));
        }
    }
    
    if crossings.is_empty() {
        return CrossoverResult {
            found: false,
            l_star: 0.0,
            confidence_lower: 0.0,
            confidence_upper: 0.0,
        };
    }
    
    // Use the first crossing (smallest L)
    let (l_star, lower, upper) = crossings[0];
    
    CrossoverResult {
        found: true,
        l_star,
        confidence_lower: lower,
        confidence_upper: upper,
    }
}

fn print_results(
    l_values: &[f64],
    sigma2_results: &[(f64, f64, f64)],
    delta3_results: &[(f64, f64, f64)],
    sigma2_crossover: &CrossoverResult,
    delta3_crossover: &CrossoverResult,
) {
    println!("\n=== Dense L Scan Results ===");
    
    println!("\n--- Σ²(L) Results ---");
    if sigma2_crossover.found {
        println!("Crossover L*: {:.3} [{:.3}, {:.3}]", 
                 sigma2_crossover.l_star,
                 sigma2_crossover.confidence_lower,
                 sigma2_crossover.confidence_upper);
        
        let error_width = sigma2_crossover.confidence_upper - sigma2_crossover.confidence_lower;
        println!("95% CI width: {:.3}", error_width);
        
        if error_width < 0.1 {
            println!("✓ Precise crossover detection");
        } else {
            println!("⚠ Wide confidence interval");
        }
    } else {
        println!("No crossover detected in range {:.1}-{:.1}", l_values[0], l_values[l_values.len()-1]);
    }
    
    println!("\n--- Δ₃(L) Results ---");
    if delta3_crossover.found {
        println!("Crossover L*: {:.3} [{:.3}, {:.3}]", 
                 delta3_crossover.l_star,
                 delta3_crossover.confidence_lower,
                 delta3_crossover.confidence_upper);
        
        let error_width = delta3_crossover.confidence_upper - delta3_crossover.confidence_lower;
        println!("95% CI width: {:.3}", error_width);
        
        if error_width < 0.1 {
            println!("✓ Precise crossover detection");
        } else {
            println!("⚠ Wide confidence interval");
        }
    } else {
        println!("No crossover detected in range {:.1}-{:.1}", l_values[0], l_values[l_values.len()-1]);
    }
    
    println!("\n--- Cross-Metric Consistency ---");
    if sigma2_crossover.found && delta3_crossover.found {
        let diff = (sigma2_crossover.l_star - delta3_crossover.l_star).abs();
        println!("L* difference: {:.3}", diff);
        
        if diff < 0.2 {
            println!("✓ Consistent crossover scales across metrics");
        } else {
            println!("⚠ Different crossover scales between metrics");
        }
    } else {
        println!("Cannot compare - missing crossover in at least one metric");
    }
    
    println!("\n--- Key Points at L ≈ 3 ---");
    let l_3_idx = ((3.0 - l_values[0]) / (l_values[1] - l_values[0])).round() as usize;
    if l_3_idx < l_values.len() {
        println!("At L = 3.00:");
        println!("  Σ²(L) ratio: {:.3} [{:.3}, {:.3}]", 
                 sigma2_results[l_3_idx].0,
                 sigma2_results[l_3_idx].1,
                 sigma2_results[l_3_idx].2);
        println!("  Δ₃(L) ratio: {:.3} [{:.3}, {:.3}]", 
                 delta3_results[l_3_idx].0,
                 delta3_results[l_3_idx].1,
                 delta3_results[l_3_idx].2);
    }
}

fn save_results(
    zeros_file: &str,
    l_values: &[f64],
    sigma2_results: &[(f64, f64, f64)],
    delta3_results: &[(f64, f64, f64)],
    sigma2_crossover: &CrossoverResult,
    delta3_crossover: &CrossoverResult,
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let mut file = File::create(output_file)?;
    
    writeln!(file, "# Dense Rigidity Analysis Results")?;
    writeln!(file, "# Zeros file: {}", zeros_file)?;
    writeln!(file, "# L grid: {:.2} to {:.2} step {:.2}", l_values[0], l_values[l_values.len()-1], l_values[1] - l_values[0])?;
    writeln!(file, "")?;
    
    writeln!(file, "# Crossover Points")?;
    writeln!(file, "sigma2_crossover_found = {}", sigma2_crossover.found)?;
    if sigma2_crossover.found {
        writeln!(file, "sigma2_l_star = {:.6}", sigma2_crossover.l_star)?;
        writeln!(file, "sigma2_confidence_lower = {:.6}", sigma2_crossover.confidence_lower)?;
        writeln!(file, "sigma2_confidence_upper = {:.6}", sigma2_crossover.confidence_upper)?;
    }
    
    writeln!(file, "delta3_crossover_found = {}", delta3_crossover.found)?;
    if delta3_crossover.found {
        writeln!(file, "delta3_l_star = {:.6}", delta3_crossover.l_star)?;
        writeln!(file, "delta3_confidence_lower = {:.6}", delta3_crossover.confidence_lower)?;
        writeln!(file, "delta3_confidence_upper = {:.6}", delta3_crossover.confidence_upper)?;
    }
    
    writeln!(file, "")?;
    writeln!(file, "# Dense Data Points")?;
    writeln!(file, "# L, sigma2_mean, sigma2_lower, sigma2_upper, delta3_mean, delta3_lower, delta3_upper")?;
    
    for (i, &l) in l_values.iter().enumerate() {
        writeln!(file, "{:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}",
                 l,
                 sigma2_results[i].0, sigma2_results[i].1, sigma2_results[i].2,
                 delta3_results[i].0, delta3_results[i].1, delta3_results[i].2)?;
    }
    
    Ok(())
}

#[derive(Debug)]
struct CrossoverResult {
    found: bool,
    l_star: f64,
    confidence_lower: f64,
    confidence_upper: f64,
}
