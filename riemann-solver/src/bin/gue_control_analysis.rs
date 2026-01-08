/// GUE Control Analysis - Tests if Σ² estimator creates artifacts
/// 
/// Addresses GPT-2.1 concern: "Your Σ² estimator might have bias that creates a faux crossover"
/// 
/// Control experiment:
/// 1. Generate GUE eigenvalues
/// 2. Unfold them the same way as zeros
/// 3. Run identical Σ²(L) estimator
/// 4. Compare to theoretical GUE curve
/// 5. Expected: ratio should stay ~1 for all L

use std::fs::File;
use std::io::{self, BufRead, Write};
use clap::Parser;
use riemann_solver::analysis::spectral::{NumberVariance, DysonMehta};
use riemann_solver::analysis::unfolding::UnfoldingMethod;
use riemann_solver::hamiltonian::gue::GueSystem;
use riemann_solver::solver::lapack::LapackSolver;

#[derive(Parser)]
#[command(name = "gue_control_analysis")]
#[command(about = "GUE control analysis for estimator validation")]
struct Args {
    /// GUE matrix size
    #[arg(short, long, default_value_t = 300)]
    size: usize,
    
    /// Number of GUE instances to generate
    #[arg(short, long, default_value_t = 100)]
    instances: usize,
    
    /// L range start
    #[arg(long, default_value_t = 1.0)]
    l_min: f64,
    
    /// L range end
    #[arg(long, default_value_t = 6.0)]
    l_max: f64,
    
    /// L step size
    #[arg(long, default_value_t = 0.1)]
    l_step: f64,
    
    /// Output results to file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== GUE Control Analysis ===");
    println!("Testing if Σ² estimator creates crossover artifacts");
    println!("GUE size: {}, instances: {}", args.size, args.instances);
    
    // Generate L grid
    let l_values: Vec<f64> = (0..)
        .map(|i| args.l_min + i as f64 * args.l_step)
        .take_while(|&l| l <= args.l_max)
        .collect();
    
    println!("L grid: {} points from {:.1} to {:.1}", l_values.len(), args.l_min, args.l_max);
    
    // Generate GUE instances and compute statistics
    println!("Generating {} GUE instances...", args.instances);
    let gue_results = generate_gue_statistics(args.size, args.instances, &l_values)?;
    
    // Compute theoretical GUE curves
    println!("Computing theoretical GUE curves...");
    let theoretical_sigma2 = compute_theoretical_sigma2(&l_values);
    let theoretical_delta3 = compute_theoretical_delta3(&l_values);
    
    // Compute ratios
    let sigma2_ratios = compute_ratios(&gue_results.sigma2_mean, &theoretical_sigma2);
    let delta3_ratios = compute_ratios(&gue_results.delta3_mean, &theoretical_delta3);
    
    // Analyze for artificial crossovers
    println!("Analyzing for artificial crossovers...");
    let sigma2_crossover = detect_artificial_crossover(&l_values, &sigma2_ratios);
    let delta3_crossover = detect_artificial_crossover(&l_values, &delta3_ratios);
    
    // Print results
    print_control_results(&l_values, &sigma2_ratios, &delta3_ratios, 
                          &sigma2_crossover, &delta3_crossover);
    
    // Save results if requested
    if let Some(output_file) = &args.output {
        save_control_results(&args, &l_values, &gue_results, &theoretical_sigma2, &theoretical_delta3,
                           &sigma2_ratios, &delta3_ratios, output_file)?;
        println!("\nResults saved to: {}", output_file);
    }
    
    Ok(())
}

#[derive(Debug)]
struct GueResults {
    sigma2_mean: Vec<f64>,
    sigma2_std: Vec<f64>,
    delta3_mean: Vec<f64>,
    delta3_std: Vec<f64>,
}

fn generate_gue_statistics(size: usize, instances: usize, l_values: &[f64]) -> Result<GueResults, Box<dyn std::error::Error>> {
    let mut sigma2_values = vec![Vec::new(); l_values.len()];
    let mut delta3_values = vec![Vec::new(); l_values.len()];
    
    let solver = LapackSolver::new();
    let number_variance = NumberVariance::new();
    let dyson_mehta = DysonMehta::new();
    let unfolding = UnfoldingMethod::LocalDensity;
    
    for i in 0..instances {
        if i % 10 == 0 {
            println!("  Instance {}/{}", i + 1, instances);
        }
        
        // Generate GUE matrix
        let gue = GueSystem::new(size, None)?;
        let matrix = gue.generate_hamiltonian()?;
        
        // Compute eigenvalues
        let eigenvalues = solver.solve(&matrix)?;
        
        // Unfold eigenvalues
        let unfolded = unfolding.unfold_spectrum(&eigenvalues);
        
        // Compute statistics
        let sigma2 = number_variance.compute(&unfolded, l_values);
        let delta3 = dyson_mehta.compute(&unfolded, l_values);
        
        // Store results
        for (j, &s2) in sigma2.iter().enumerate() {
            sigma2_values[j].push(s2);
        }
        for (j, &d3) in delta3.iter().enumerate() {
            delta3_values[j].push(d3);
        }
    }
    
    // Compute means and standard deviations
    let mut sigma2_mean = Vec::new();
    let mut sigma2_std = Vec::new();
    let mut delta3_mean = Vec::new();
    let mut delta3_std = Vec::new();
    
    for i in 0..l_values.len() {
        // Σ² statistics
        let s2_mean = sigma2_values[i].iter().sum::<f64>() / sigma2_values[i].len() as f64;
        let s2_var = sigma2_values[i].iter()
            .map(|x| (x - s2_mean).powi(2))
            .sum::<f64>() / sigma2_values[i].len() as f64;
        
        sigma2_mean.push(s2_mean);
        sigma2_std.push(s2_var.sqrt());
        
        // Δ₃ statistics
        let d3_mean = delta3_values[i].iter().sum::<f64>() / delta3_values[i].len() as f64;
        let d3_var = delta3_values[i].iter()
            .map(|x| (x - d3_mean).powi(2))
            .sum::<f64>() / delta3_values[i].len() as f64;
        
        delta3_mean.push(d3_mean);
        delta3_std.push(d3_var.sqrt());
    }
    
    Ok(GueResults {
        sigma2_mean,
        sigma2_std,
        delta3_mean,
        delta3_std,
    })
}

fn compute_theoretical_sigma2(l_values: &[f64]) -> Vec<f64> {
    // GUE theoretical Σ²(L) ≈ (2/π²) * (ln(L) + γ + 1 - (π²/8))
    // where γ is Euler-Mascheroni constant
    let gamma = 0.5772156649015328606065120900824024310421;
    
    l_values.iter().map(|&l| {
        if l <= 1.0 {
            return l * (2.0 / std::f64::consts::PI); // Linear for small L
        }
        (2.0 / (std::f64::consts::PI * std::f64::consts::PI)) * 
        (l.ln() + gamma + 1.0 - (std::f64::consts::PI * std::f64::consts::PI) / 8.0)
    }).collect()
}

fn compute_theoretical_delta3(l_values: &[f64]) -> Vec<f64> {
    // GUE theoretical Δ₃(L) ≈ (1/π²) * ln(L) + constant
    l_values.iter().map(|&l| {
        if l <= 1.0 {
            return l * (1.0 / (std::f64::consts::PI * std::f64::consts::PI));
        }
        (1.0 / (std::f64::consts::PI * std::f64::consts::PI)) * l.ln() + 0.1
    }).collect()
}

fn compute_ratios(measured: &[f64], theoretical: &[f64]) -> Vec<f64> {
    measured.iter().zip(theoretical.iter())
        .map(|(m, t)| {
            if t.abs() > 1e-10 {
                m / t
            } else {
                1.0
            }
        })
        .collect()
}

fn detect_artificial_crossover(l_values: &[f64], ratios: &[f64]) -> CrossoverDetection {
    // Look for significant deviation from 1.0 that could mimic a crossover
    let max_deviation = ratios.iter()
        .map(|r| (r - 1.0).abs())
        .fold(f64::NEG_INFINITY, f64::max);
    
    let min_deviation = ratios.iter()
        .map(|r| (r - 1.0).abs())
        .fold(f64::INFINITY, f64::min);
    
    // Check if there's a systematic trend that could be mistaken for crossover
    let mut trend_changes = 0;
    for i in 1..ratios.len() {
        let diff_prev = ratios[i-1] - 1.0;
        let diff_curr = ratios[i] - 1.0;
        if diff_prev * diff_curr < 0.0 {
            trend_changes += 1;
        }
    }
    
    CrossoverDetection {
        max_deviation,
        min_deviation,
        trend_changes,
        likely_artifact: max_deviation > 0.1 || trend_changes > 2,
    }
}

fn print_control_results(
    l_values: &[f64],
    sigma2_ratios: &[f64],
    delta3_ratios: &[f64],
    sigma2_crossover: &CrossoverDetection,
    delta3_crossover: &CrossoverDetection,
) {
    println!("\n=== GUE Control Results ===");
    
    println!("\n--- Σ²(L) Control ---");
    println!("Max deviation from theory: {:.3}", sigma2_crossover.max_deviation);
    println!("Min deviation from theory: {:.3}", sigma2_crossover.min_deviation);
    println!("Trend changes: {}", sigma2_crossover.trend_changes);
    
    if sigma2_crossover.likely_artifact {
        println!("⚠ WARNING: Σ² estimator shows artifacts that could mimic crossover!");
    } else {
        println!("✓ Σ² estimator appears stable (no artificial crossover)");
    }
    
    println!("\n--- Δ₃(L) Control ---");
    println!("Max deviation from theory: {:.3}", delta3_crossover.max_deviation);
    println!("Min deviation from theory: {:.3}", delta3_crossover.min_deviation);
    println!("Trend changes: {}", delta3_crossover.trend_changes);
    
    if delta3_crossover.likely_artifact {
        println!("⚠ WARNING: Δ₃ estimator shows artifacts that could mimic crossover!");
    } else {
        println!("✓ Δ₃ estimator appears stable (no artificial crossover)");
    }
    
    println!("\n--- Key Points at L ≈ 3 ---");
    let l_3_idx = ((3.0 - l_values[0]) / (l_values[1] - l_values[0])).round() as usize;
    if l_3_idx < l_values.len() {
        println!("At L = 3.0:");
        println!("  Σ²(L) ratio: {:.3}", sigma2_ratios[l_3_idx]);
        println!("  Δ₃(L) ratio: {:.3}", delta3_ratios[l_3_idx]);
        
        if (sigma2_ratios[l_3_idx] - 1.0).abs() > 0.05 || (delta3_ratios[l_3_idx] - 1.0).abs() > 0.05 {
            println!("  ⚠ Significant deviation at L=3 - could affect crossover detection");
        } else {
            println!("  ✓ Close to theoretical expectation");
        }
    }
    
    println!("\n--- Overall Assessment ---");
    if sigma2_crossover.likely_artifact || delta3_crossover.likely_artifact {
        println!("❌ ESTIMATOR ISSUE: Control shows artifacts - crossover detection unreliable");
    } else {
        println!("✅ ESTIMATOR VALID: Control shows no artifacts - crossover detection reliable");
    }
}

fn save_control_results(
    args: &Args,
    l_values: &[f64],
    gue_results: &GueResults,
    theoretical_sigma2: &[f64],
    theoretical_delta3: &[f64],
    sigma2_ratios: &[f64],
    delta3_ratios: &[f64],
    output_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = File::create(output_file)?;
    
    writeln!(file, "# GUE Control Analysis Results")?;
    writeln!(file, "# GUE size: {}, instances: {}", args.size, args.instances)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Control Assessment")?;
    writeln!(file, "sigma2_max_deviation = {:.6}", sigma2_ratios.iter()
             .map(|r| (r - 1.0).abs()).fold(f64::NEG_INFINITY, f64::max))?;
    writeln!(file, "delta3_max_deviation = {:.6}", delta3_ratios.iter()
             .map(|r| (r - 1.0).abs()).fold(f64::NEG_INFINITY, f64::max))?;
    writeln!(file, "")?;
    
    writeln!(file, "# Data Points")?;
    writeln!(file, "# L, sigma2_mean, sigma2_std, sigma2_theory, sigma2_ratio, delta3_mean, delta3_std, delta3_theory, delta3_ratio")?;
    
    for i in 0..l_values.len() {
        writeln!(file, "{:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}, {:.6}",
                 l_values[i],
                 gue_results.sigma2_mean[i], gue_results.sigma2_std[i], theoretical_sigma2[i], sigma2_ratios[i],
                 gue_results.delta3_mean[i], gue_results.delta3_std[i], theoretical_delta3[i], delta3_ratios[i])?;
    }
    
    Ok(())
}

#[derive(Debug)]
struct CrossoverDetection {
    max_deviation: f64,
    min_deviation: f64,
    trend_changes: usize,
    likely_artifact: bool,
}
