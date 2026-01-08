/// Fixed Scientific Validation Suite for L≈3-5 Crossover
/// 
/// Addresses GPT-2.1 concerns with corrected estimators:
/// 1. Fixed Σ² and Δ₃ estimators
/// 2. Proper unfolding for different data types
/// 3. Bootstrap confidence intervals
/// 4. Dense L scanning with interpolation

use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use riemann_solver::analysis::spectral_fixed::{
    number_variance_fixed, number_variance_gue_fixed,
    dyson_mehta_fixed, dyson_mehta_gue_fixed,
    bootstrap_rigidity, find_crossover_fixed, CrossoverResultFixed
};
use riemann_solver::analysis::unfolding::unfold_zeros;
use riemann_solver::analysis::unfolding_trait::{UnfoldingMethod, GueUnfolding};

#[derive(Parser)]
#[command(name = "scientific_validation_fixed")]
#[command(about = "Fixed scientific validation suite for L≈3-5 crossover")]
struct Args {
    /// Validation mode: dense, gue_control, or both
    #[arg(short, long, default_value = "both")]
    mode: String,
    
    /// File containing Riemann zeros (for dense mode)
    #[arg(long)]
    zeros_file: Option<String>,
    
    /// GUE matrix size (for gue_control mode)
    #[arg(long, default_value_t = 300)]
    gue_size: usize,
    
    /// Number of GUE instances
    #[arg(long, default_value_t = 30)]
    gue_instances: usize,
    
    /// L range for dense scanning
    #[arg(long, default_value_t = 1.0)]
    l_min: f64,
    
    #[arg(long, default_value_t = 6.0)]
    l_max: f64,
    
    #[arg(long, default_value_t = 0.1)]
    l_step: f64,
    
    /// Bootstrap samples
    #[arg(long, default_value_t = 50)]
    bootstrap_samples: usize,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Fixed Scientific Validation Suite ===");
    println!("Testing corrected estimators for GPT-2.1 concerns");
    
    match args.mode.as_str() {
        "dense" => {
            if let Some(ref zeros_file) = args.zeros_file {
                run_dense_analysis(zeros_file, &args)?;
            } else {
                eprintln!("Error: --zeros-file required for dense mode");
                std::process::exit(1);
            }
        }
        "gue_control" => {
            run_gue_control(&args)?;
        }
        "both" => {
            if let Some(ref zeros_file) = args.zeros_file {
                run_dense_analysis(zeros_file, &args)?;
                println!("\n{}", "=".repeat(60));
                run_gue_control(&args)?;
            } else {
                eprintln!("Error: --zeros-file required for dense mode");
                std::process::exit(1);
            }
        }
        _ => {
            eprintln!("Error: mode must be 'dense', 'gue_control', or 'both'");
            std::process::exit(1);
        }
    }
    
    Ok(())
}

fn run_dense_analysis(zeros_file: &str, args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Fixed Dense L Scanning Analysis ---");
    println!("File: {}", zeros_file);
    
    // Load zeros
    let zeros = load_zeros(zeros_file, 10000)?;
    println!("Loaded {} zeros", zeros.len());
    
    // Generate dense L grid
    let l_values: Vec<f64> = (0..)
        .map(|i| args.l_min + i as f64 * args.l_step)
        .take_while(|&l| l <= args.l_max)
        .collect();
    
    println!("Dense L grid: {} points from {:.1} to {:.1}", l_values.len(), args.l_min, args.l_max);
    
    // Unfold zeros
    let unfolded = unfold_zeros(&zeros);
    
    // Compute metrics with fixed estimators
    println!("Computing fixed Σ²(L) and Δ₃(L)...");
    let sigma2_results: Vec<f64> = l_values.iter().map(|&l| number_variance_fixed(&unfolded, l)).collect();
    let delta3_results: Vec<f64> = l_values.iter().map(|&l| dyson_mehta_fixed(&unfolded, l)).collect();
    
    // Compute theoretical GUE curves
    let sigma2_theory: Vec<f64> = l_values.iter().map(|&l| number_variance_gue_fixed(l)).collect();
    let delta3_theory: Vec<f64> = l_values.iter().map(|&l| dyson_mehta_gue_fixed(l)).collect();
    
    // Bootstrap for confidence intervals
    println!("Computing bootstrap confidence intervals...");
    let sigma2_bootstrap = bootstrap_rigidity(&unfolded, &l_values, args.bootstrap_samples, 
                                             |data, l| number_variance_fixed(data, l));
    let delta3_bootstrap = bootstrap_rigidity(&unfolded, &l_values, args.bootstrap_samples,
                                             |data, l| dyson_mehta_fixed(data, l));
    
    // Find crossover points
    let sigma2_crossover = find_crossover_fixed(&l_values, &sigma2_bootstrap, &sigma2_theory);
    let delta3_crossover = find_crossover_fixed(&l_values, &delta3_bootstrap, &delta3_theory);
    
    // Print results
    print_dense_results_fixed(&l_values, &sigma2_results, &delta3_results, 
                           &sigma2_theory, &delta3_theory,
                           &sigma2_crossover, &delta3_crossover);
    
    Ok(())
}

fn run_gue_control(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n--- Fixed GUE Control Analysis ---");
    println!("Testing if fixed estimators create artifacts");
    println!("GUE size: {}, instances: {}", args.gue_size, args.gue_instances);
    
    // Generate L grid
    let l_values: Vec<f64> = (0..)
        .map(|i| args.l_min + i as f64 * args.l_step)
        .take_while(|&l| l <= args.l_max)
        .collect();
    
    // Generate GUE instances with proper unfolding
    let gue_stats = generate_gue_statistics_fixed(args.gue_size, args.gue_instances, &l_values)?;
    
    // Compute theoretical curves
    let sigma2_theory: Vec<f64> = l_values.iter().map(|&l| number_variance_gue_fixed(l)).collect();
    let delta3_theory: Vec<f64> = l_values.iter().map(|&l| dyson_mehta_gue_fixed(l)).collect();
    
    // Analyze for artifacts
    let sigma2_artifact = detect_artifact_fixed(&l_values, &gue_stats.sigma2_mean, &sigma2_theory);
    let delta3_artifact = detect_artifact_fixed(&l_values, &gue_stats.delta3_mean, &delta3_theory);
    
    // Print results
    print_gue_control_results_fixed(&l_values, &gue_stats, &sigma2_theory, &delta3_theory,
                                  &sigma2_artifact, &delta3_artifact);
    
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

fn generate_gue_statistics_fixed(size: usize, instances: usize, l_values: &[f64]) -> Result<GueStatsFixed, Box<dyn std::error::Error>> {
    let mut sigma2_values = vec![Vec::new(); l_values.len()];
    let mut delta3_values = vec![Vec::new(); l_values.len()];
    
    let gue_unfolding = GueUnfolding;
    
    for i in 0..instances {
        if i % 10 == 0 {
            println!("  GUE instance {}/{}", i + 1, instances);
        }
        
        // Generate GUE eigenvalues (Wigner semicircle)
        let eigenvalues = generate_gue_eigenvalues_fixed(size)?;
        let unfolded = gue_unfolding.unfold_spectrum(&eigenvalues);
        
        // Compute statistics with fixed estimators
        for (j, &l) in l_values.iter().enumerate() {
            sigma2_values[j].push(number_variance_fixed(&unfolded, l));
            delta3_values[j].push(dyson_mehta_fixed(&unfolded, l));
        }
    }
    
    // Compute means
    let sigma2_mean: Vec<f64> = sigma2_values.iter()
        .map(|vals| vals.iter().sum::<f64>() / vals.len() as f64)
        .collect();
    
    let delta3_mean: Vec<f64> = delta3_values.iter()
        .map(|vals| vals.iter().sum::<f64>() / vals.len() as f64)
        .collect();
    
    Ok(GueStatsFixed {
        sigma2_mean,
        delta3_mean,
    })
}

fn generate_gue_eigenvalues_fixed(size: usize) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    
    // Generate Wigner semicircle eigenvalues
    let mut eigenvalues = Vec::new();
    for _ in 0..size {
        // Sample from semicircle distribution
        let x: f64 = rng.gen_range(-2.0..2.0);
        let y: f64 = rng.gen_range(0.0..1.0);
        
        // Accept if under semicircle
        if y <= (1.0 - (x * x) / 4.0).sqrt() {
            eigenvalues.push(x);
        }
    }
    
    eigenvalues.sort_by(|a, b| a.partial_cmp(b).unwrap());
    Ok(eigenvalues)
}

fn detect_artifact_fixed(_l_values: &[f64], measured: &[f64], theory: &[f64]) -> ArtifactDetectionFixed {
    let max_deviation = measured.iter().zip(theory.iter())
        .map(|(m, t)| (m / t - 1.0).abs())
        .fold(f64::NEG_INFINITY, f64::max);
    
    // Check for systematic trends
    let mut trend_changes = 0;
    for i in 1..measured.len() {
        let ratio_prev = measured[i-1] / theory[i-1];
        let ratio_curr = measured[i] / theory[i];
        if (ratio_prev - 1.0) * (ratio_curr - 1.0) < 0.0 {
            trend_changes += 1;
        }
    }
    
    ArtifactDetectionFixed {
        max_deviation,
        trend_changes,
        likely_artifact: max_deviation > 0.05 || trend_changes > 2,
    }
}

fn print_dense_results_fixed(
    l_values: &[f64],
    sigma2_results: &[f64],
    delta3_results: &[f64],
    sigma2_theory: &[f64],
    delta3_theory: &[f64],
    sigma2_crossover: &CrossoverResultFixed,
    delta3_crossover: &CrossoverResultFixed,
) {
    println!("\n--- Fixed Dense L Scan Results ---");
    
    println!("\nΣ²(L) Analysis:");
    if sigma2_crossover.found {
        println!("  Crossover L*: {:.3} [{:.3}, {:.3}]", 
                 sigma2_crossover.l_star,
                 sigma2_crossover.confidence_lower,
                 sigma2_crossover.confidence_upper);
        
        if (sigma2_crossover.l_star - 3.0).abs() < 0.2 {
            println!("  ✓ Close to L≈3 (diff: {:.3})", (sigma2_crossover.l_star - 3.0).abs());
        } else {
            println!("  ⚠ Far from L≈3 (diff: {:.3})", (sigma2_crossover.l_star - 3.0).abs());
        }
    } else {
        println!("  No crossover detected");
    }
    
    println!("\nΔ₃(L) Analysis:");
    if delta3_crossover.found {
        println!("  Crossover L*: {:.3} [{:.3}, {:.3}]", 
                 delta3_crossover.l_star,
                 delta3_crossover.confidence_lower,
                 delta3_crossover.confidence_upper);
        
        if (delta3_crossover.l_star - 3.0).abs() < 0.2 {
            println!("  ✓ Close to L≈3 (diff: {:.3})", (delta3_crossover.l_star - 3.0).abs());
        } else {
            println!("  ⚠ Far from L≈3 (diff: {:.3})", (delta3_crossover.l_star - 3.0).abs());
        }
    } else {
        println!("  No crossover detected");
    }
    
    println!("\nCross-metric consistency:");
    if sigma2_crossover.found && delta3_crossover.found {
        let diff = (sigma2_crossover.l_star - delta3_crossover.l_star).abs();
        if diff < 0.3 {
            println!("  ✓ Consistent crossover scales (diff: {:.3})", diff);
        } else {
            println!("  ⚠ Different crossover scales (diff: {:.3})", diff);
        }
    }
    
    println!("\nKey points at L≈3:");
    let l_3_idx = ((3.0 - l_values[0]) / (l_values[1] - l_values[0])).round() as usize;
    if l_3_idx < l_values.len() {
        let sigma2_ratio = sigma2_results[l_3_idx] / sigma2_theory[l_3_idx];
        let delta3_ratio = delta3_results[l_3_idx] / delta3_theory[l_3_idx];
        println!("  Σ²(L) ratio: {:.3}", sigma2_ratio);
        println!("  Δ₃(L) ratio: {:.3}", delta3_ratio);
        
        if sigma2_ratio > 1.1 && delta3_ratio > 1.1 {
            println!("  ✓ Both metrics show enhanced rigidity");
        } else if sigma2_ratio < 0.9 && delta3_ratio < 0.9 {
            println!("  ⚠ Both metrics show reduced rigidity");
        }
    }
}

fn print_gue_control_results_fixed(
    l_values: &[f64],
    gue_stats: &GueStatsFixed,
    sigma2_theory: &[f64],
    delta3_theory: &[f64],
    sigma2_artifact: &ArtifactDetectionFixed,
    delta3_artifact: &ArtifactDetectionFixed,
) {
    println!("\n--- Fixed GUE Control Results ---");
    
    println!("\nΣ²(L) Control:");
    println!("  Max deviation: {:.3}", sigma2_artifact.max_deviation);
    println!("  Trend changes: {}", sigma2_artifact.trend_changes);
    
    if sigma2_artifact.likely_artifact {
        println!("  ⚠ WARNING: Σ² estimator still shows artifacts!");
    } else {
        println!("  ✓ Σ² estimator appears stable");
    }
    
    println!("\nΔ₃(L) Control:");
    println!("  Max deviation: {:.3}", delta3_artifact.max_deviation);
    println!("  Trend changes: {}", delta3_artifact.trend_changes);
    
    if delta3_artifact.likely_artifact {
        println!("  ⚠ WARNING: Δ₃ estimator still shows artifacts!");
    } else {
        println!("  ✓ Δ₃ estimator appears stable");
    }
    
    println!("\nOverall assessment:");
    if sigma2_artifact.likely_artifact || delta3_artifact.likely_artifact {
        println!("  ❌ ESTIMATOR ISSUE: Artifacts still detected - need more fixes");
    } else {
        println!("  ✅ ESTIMATOR VALID: No artifacts - crossover detection reliable");
    }
    
    println!("\nKey points at L≈3:");
    let l_3_idx = ((3.0 - l_values[0]) / (l_values[1] - l_values[0])).round() as usize;
    if l_3_idx < l_values.len() {
        let sigma2_ratio = gue_stats.sigma2_mean[l_3_idx] / sigma2_theory[l_3_idx];
        let delta3_ratio = gue_stats.delta3_mean[l_3_idx] / delta3_theory[l_3_idx];
        println!("  Σ²(L) ratio: {:.3}", sigma2_ratio);
        println!("  Δ₃(L) ratio: {:.3}", delta3_ratio);
        
        if (sigma2_ratio - 1.0).abs() < 0.05 && (delta3_ratio - 1.0).abs() < 0.05 {
            println!("  ✓ Close to theoretical expectation");
        } else {
            println!("  ⚠ Significant deviation from theory");
        }
    }
}

#[derive(Debug)]
struct GueStatsFixed {
    sigma2_mean: Vec<f64>,
    delta3_mean: Vec<f64>,
}

#[derive(Debug)]
struct ArtifactDetectionFixed {
    max_deviation: f64,
    trend_changes: usize,
    likely_artifact: bool,
}
