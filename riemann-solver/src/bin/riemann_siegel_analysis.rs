/// Command-line tool to test Riemann-Siegel oscillation connection to L≈3-5 crossover
/// 
/// Tests: Does L≈3-5 crossover relate to θ(t) oscillation scale?
/// 
/// Usage:
/// ./target/release/riemann_siegel_analysis zeros_file.txt

use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use riemann_solver::analysis::riemann_siegel::analyze_riemann_siegel;

#[derive(Parser)]
#[command(name = "riemann_siegel_analysis")]
#[command(about = "Test Riemann-Siegel oscillation connection to L≈3-5 crossover")]
struct Args {
    /// File containing Riemann zeros (one per line)
    zeros_file: String,
    
    /// Maximum number of zeros to analyze
    #[arg(short, long, default_value_t = 1000)]
    max_zeros: usize,
    
    /// Output results to file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Riemann-Siegel Oscillations Analysis ===");
    println!("Testing hypothesis: L≈3-5 relates to θ(t) oscillation scale");
    println!("Loading zeros from: {}", args.zeros_file);
    
    // Load zeros from file
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    if zeros.len() < 100 {
        eprintln!("Error: Need at least 100 zeros for Riemann-Siegel analysis");
        std::process::exit(1);
    }
    
    // Analyze Riemann-Siegel oscillations
    println!("Computing θ(t) structure and rigidity...");
    let result = analyze_riemann_siegel(&zeros)?;
    
    println!("\n--- θ(t) Oscillation Analysis ---");
    println!("Mean oscillation period: {:.6}", result.theta_analysis.mean_period);
    println!("Period standard deviation: {:.6}", result.theta_analysis.period_std_dev);
    println!("Mean period in spacing units: {:.3}", result.theta_analysis.mean_spacing_units);
    println!("Spacing units std dev: {:.3}", result.theta_analysis.units_std_dev);
    
    println!("\n--- Height Dependence ---");
    println!("Sampled {} different heights", result.theta_analysis.height_samples.len());
    
    for (i, ((&height, &period), &units)) in result.theta_analysis.height_samples.iter()
        .zip(result.theta_analysis.period_samples.iter())
        .zip(result.theta_analysis.spacing_units.iter())
        .enumerate() {
        println!("  Height {}: T≈{:.1}, Period={:.6}, Units={:.3}", 
                 i+1, height, period, units);
    }
    
    println!("\n--- Crossover Analysis ---");
    println!("θ(t) period in spacing units: {:.3}", result.theta_analysis.mean_spacing_units);
    println!("Crossover scale (L≈3-5): 3.0 - 5.0");
    
    if result.scale_match {
        println!("✓ MATCH: θ(t) period within crossover range!");
        println!("  → θ(t) oscillations could explain crossover");
    } else if result.theta_analysis.mean_spacing_units < 3.0 {
        println!("✗ SHORTER: θ(t) period below crossover range");
        println!("  → θ(t) oscillations too fast for crossover");
    } else {
        println!("✗ LONGER: θ(t) period above crossover range");
        println!("  → θ(t) oscillations too slow for crossover");
    }
    
    println!("\n--- Consistency Analysis ---");
    println!("Height consistency (std/mean): {:.3}", 
             result.theta_analysis.units_std_dev / result.theta_analysis.mean_spacing_units);
    
    if result.height_consistency {
        println!("✓ Consistent across heights (stable oscillations)");
    } else {
        println!("⚠ Varies with height (unstable oscillations)");
    }
    
    println!("\n--- Correlation Analysis ---");
    println!("θ(t)-zero correlation: {:.6}", result.theta_zero_correlation);
    
    if result.theta_zero_correlation.abs() > 0.5 {
        println!("✓ Strong correlation between θ(t) and zero positions");
    } else if result.theta_zero_correlation.abs() > 0.2 {
        println!("⚠ Moderate correlation between θ(t) and zero positions");
    } else {
        println!("✗ Weak correlation between θ(t) and zero positions");
    }
    
    println!("\n--- Overall Assessment ---");
    if result.explains_crossover {
        println!("✓ θ(t) oscillations LIKELY explain L≈3-5 crossover");
        println!("  → Scale match ✓");
        println!("  → Height consistency ✓");
        println!("  → Correlation strength ✓");
    } else {
        println!("✗ θ(t) oscillations DO NOT explain L≈3-5 crossover");
        
        let mut reasons = Vec::new();
        if !result.scale_match {
            reasons.push("Scale mismatch");
        }
        if !result.height_consistency {
            reasons.push("Height inconsistency");
        }
        if result.theta_zero_correlation.abs() <= 0.3 {
            reasons.push("Weak correlation");
        }
        
        println!("  → Reasons: {}", reasons.join(", "));
    }
    
    // Additional diagnostics
    let ratio = result.theta_analysis.mean_spacing_units / 4.0;
    println!("Ratio to crossover midpoint (L=4): {:.3}", ratio);
    
    // Check for potential mechanisms
    println!("\n--- Physical Interpretation ---");
    if result.scale_match {
        println!("θ(t) oscillations create spectral rigidity at L≈3-5 scale");
        println!("Mechanism: Phase coherence in Z(t) = e^{{iθ(t)}}ζ(1/2 + it)");
    } else {
        println!("θ(t) oscillations operate at different scale than crossover");
        println!("Crossover likely caused by different mechanism");
    }
    
    // Save results if requested
    if let Some(output_file) = &args.output {
        save_results(&args.zeros_file, &result, output_file)?;
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

fn save_results(zeros_file: &str, result: &riemann_solver::analysis::riemann_siegel::RigidityResult, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let mut file = File::create(output_file)?;
    
    writeln!(file, "# Riemann-Siegel Oscillations Analysis")?;
    writeln!(file, "# Zeros file: {}", zeros_file)?;
    writeln!(file, "# Testing: θ(t) oscillation scale vs L≈3-5 crossover")?;
    writeln!(file, "")?;
    
    writeln!(file, "# θ(t) Oscillation Statistics")?;
    writeln!(file, "mean_period = {:.6}", result.theta_analysis.mean_period)?;
    writeln!(file, "period_std_dev = {:.6}", result.theta_analysis.period_std_dev)?;
    writeln!(file, "mean_spacing_units = {:.6}", result.theta_analysis.mean_spacing_units)?;
    writeln!(file, "units_std_dev = {:.6}", result.theta_analysis.units_std_dev)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Crossover Analysis")?;
    writeln!(file, "scale_match = {}", result.scale_match)?;
    writeln!(file, "crossover_min = 3.0")?;
    writeln!(file, "crossover_max = 5.0")?;
    writeln!(file, "ratio_to_midpoint = {:.6}", result.theta_analysis.mean_spacing_units / 4.0)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Consistency Analysis")?;
    writeln!(file, "height_consistency = {}", result.height_consistency)?;
    writeln!(file, "consistency_ratio = {:.6}", 
             result.theta_analysis.units_std_dev / result.theta_analysis.mean_spacing_units)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Correlation Analysis")?;
    writeln!(file, "theta_zero_correlation = {:.6}", result.theta_zero_correlation)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Overall Assessment")?;
    writeln!(file, "explains_crossover = {}", result.explains_crossover)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Height-dependent Data")?;
    writeln!(file, "# height, period, spacing_units")?;
    for (i, ((&height, &period), &units)) in result.theta_analysis.height_samples.iter()
        .zip(result.theta_analysis.period_samples.iter())
        .zip(result.theta_analysis.spacing_units.iter())
        .enumerate() {
        writeln!(file, "sample_{} = {:.6}, {:.6}, {:.6}", i+1, height, period, units)?;
    }
    
    Ok(())
}
