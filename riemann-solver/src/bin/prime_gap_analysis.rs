/// Command-line tool to test prime gap connection to L≈3-5 crossover
/// 
/// Tests: Does L≈3-5 crossover scale match typical prime gap scale?
/// 
/// Usage:
/// ./target/release/prime_gap_analysis zeros_file.txt

use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use riemann_solver::analysis::primes::analyze_prime_gap_connection;

#[derive(Parser)]
#[command(name = "prime_gap_analysis")]
#[command(about = "Test prime gap connection to L≈3-5 crossover")]
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
    
    println!("=== Prime Gap Analysis for L≈3-5 Crossover ===");
    println!("Loading zeros from: {}", args.zeros_file);
    
    // Load zeros from file
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    if zeros.len() < 100 {
        eprintln!("Error: Need at least 100 zeros for prime gap analysis");
        std::process::exit(1);
    }
    
    // Analyze prime gap connection
    println!("Computing prime gap statistics...");
    let result = analyze_prime_gap_connection(&zeros)?;
    
    println!("\n--- Prime Gap Statistics ---");
    println!("Overall gap statistics:");
    println!("  Count: {}", result.overall_stats.count);
    println!("  Mean gap: {:.3}", result.overall_stats.mean_gap);
    println!("  Std dev: {:.3}", result.overall_stats.std_dev);
    println!("  Mode gap: {:.3}", result.overall_stats.mode_gap);
    println!("  Median (P50): {:.3}", result.overall_stats.p50);
    println!("  P10-P90 range: {:.3} - {:.3}", result.overall_stats.p10, result.overall_stats.p90);
    
    println!("\n--- Gap Statistics by Zero Height ---");
    println!("Low zeros (T≈{:.1}):", result.low_height_stats.0);
    print_gap_stats(&result.low_height_stats.1);
    
    println!("Mid zeros (T≈{:.1}):", result.mid_height_stats.0);
    print_gap_stats(&result.mid_height_stats.1);
    
    println!("High zeros (T≈{:.1}):", result.high_height_stats.0);
    print_gap_stats(&result.high_height_stats.1);
    
    println!("\n--- Crossover Analysis ---");
    println!("Prime gap scale (in zero spacing units): {:.3}", result.prime_gap_scale);
    println!("Crossover scale (L≈3-5): 3.0 - 5.0");
    
    if result.crossover_match {
        println!("✓ MATCH: Prime gap scale within crossover range!");
        println!("  → Prime gaps could explain crossover");
    } else if result.prime_gap_scale < 3.0 {
        println!("✗ SHORTER: Prime gap scale below crossover range");
        println!("  → Prime gaps too small for crossover");
    } else {
        println!("✗ LONGER: Prime gap scale above crossover range");
        println!("  → Prime gaps too large for crossover");
    }
    
    println!("\n--- Correlation Analysis ---");
    println!("Zero-prime gap correlation: {:.6}", result.zero_gap_correlation);
    
    if result.zero_gap_correlation.abs() > 0.5 {
        println!("✓ Strong correlation between zeros and prime gaps");
    } else if result.zero_gap_correlation.abs() > 0.2 {
        println!("⚠ Moderate correlation between zeros and prime gaps");
    } else {
        println!("✗ Weak correlation between zeros and prime gaps");
    }
    
    // Additional analysis
    let ratio = result.prime_gap_scale / 4.0; // Compare to midpoint
    println!("Ratio to crossover midpoint (L=4): {:.3}", ratio);
    
    // Check for height dependence
    let low_gap = result.low_height_stats.1.p50;
    let mid_gap = result.mid_height_stats.1.p50;
    let high_gap = result.high_height_stats.1.p50;
    
    println!("\n--- Height Dependence ---");
    println!("Median gaps by height:");
    println!("  Low: {:.3}, Mid: {:.3}, High: {:.3}", low_gap, mid_gap, high_gap);
    
    let variation = ((high_gap - low_gap) / mid_gap).abs();
    if variation < 0.1 {
        println!("✓ Minimal height variation (stable gaps)");
    } else if variation < 0.3 {
        println!("⚠ Moderate height variation");
    } else {
        println!("✗ Strong height variation (gaps change with height)");
    }
    
    // Save results if requested
    if let Some(output_file) = &args.output {
        save_results(&args.zeros_file, &result, output_file)?;
        println!("\nResults saved to: {}", output_file);
    }
    
    Ok(())
}

fn print_gap_stats(stats: &riemann_solver::analysis::primes::GapStats) {
    println!("  Mean: {:.3}, Median: {:.3}, Mode: {:.3}", 
             stats.mean_gap, stats.p50, stats.mode_gap);
    println!("  Std dev: {:.3}, P10-P90: {:.3}-{:.3}", 
             stats.std_dev, stats.p10, stats.p90);
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

fn save_results(zeros_file: &str, result: &riemann_solver::analysis::primes::PrimeGapResult, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let mut file = File::create(output_file)?;
    
    writeln!(file, "# Prime Gap Analysis for L≈3-5 Crossover")?;
    writeln!(file, "# Zeros file: {}", zeros_file)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Overall Gap Statistics")?;
    writeln!(file, "gap_count = {}", result.overall_stats.count)?;
    writeln!(file, "mean_gap = {:.6}", result.overall_stats.mean_gap)?;
    writeln!(file, "gap_std_dev = {:.6}", result.overall_stats.std_dev)?;
    writeln!(file, "mode_gap = {:.6}", result.overall_stats.mode_gap)?;
    writeln!(file, "median_gap = {:.6}", result.overall_stats.p50)?;
    writeln!(file, "p10_gap = {:.6}", result.overall_stats.p10)?;
    writeln!(file, "p90_gap = {:.6}", result.overall_stats.p90)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Crossover Analysis")?;
    writeln!(file, "prime_gap_scale = {:.6}", result.prime_gap_scale)?;
    writeln!(file, "crossover_min = 3.0")?;
    writeln!(file, "crossover_max = 5.0")?;
    writeln!(file, "crossover_match = {}", result.crossover_match)?;
    writeln!(file, "ratio_to_midpoint = {:.6}", result.prime_gap_scale / 4.0)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Correlation Analysis")?;
    writeln!(file, "zero_gap_correlation = {:.6}", result.zero_gap_correlation)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Height-dependent Statistics")?;
    writeln!(file, "low_height = {:.6}", result.low_height_stats.0)?;
    writeln!(file, "low_median_gap = {:.6}", result.low_height_stats.1.p50)?;
    writeln!(file, "mid_height = {:.6}", result.mid_height_stats.0)?;
    writeln!(file, "mid_median_gap = {:.6}", result.mid_height_stats.1.p50)?;
    writeln!(file, "high_height = {:.6}", result.high_height_stats.0)?;
    writeln!(file, "high_median_gap = {:.6}", result.high_height_stats.1.p50)?;
    
    Ok(())
}
