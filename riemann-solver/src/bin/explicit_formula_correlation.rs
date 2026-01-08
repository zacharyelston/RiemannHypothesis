/// Command-line tool to test explicit formula correlation length hypothesis
/// 
/// Tests: L≈3-5 is the correlation length of Σ x^ρ/ρ terms
/// 
/// Usage:
/// ./target/release/explicit_formula_correlation zeros_file.txt

use std::fs::File;
use std::io::{self, BufRead};
use clap::Parser;
use riemann_solver::analysis::explicit_formula::analyze_correlation_length;

#[derive(Parser)]
#[command(name = "explicit_formula_correlation")]
#[command(about = "Test explicit formula correlation length hypothesis")]
struct Args {
    /// File containing Riemann zeros (one per line)
    zeros_file: String,
    
    /// Maximum number of zeros to analyze
    #[arg(short, long, default_value_t = 1000)]
    max_zeros: usize,
    
    /// Output correlation data to file
    #[arg(short, long)]
    output: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Explicit Formula Correlation Length Analysis ===");
    println!("Loading zeros from: {}", args.zeros_file);
    
    // Load zeros from file
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    if zeros.len() < 100 {
        eprintln!("Error: Need at least 100 zeros for correlation analysis");
        std::process::exit(1);
    }
    
    // Analyze correlation length
    println!("Computing correlation length...");
    let correlation_length = analyze_correlation_length(&zeros)?;
    
    println!("\n--- Results ---");
    println!("Estimated correlation length: {:.3}", correlation_length);
    
    // Compare with L≈3-5 crossover scale
    println!("Crossover scale (L≈3-5): 3.0 - 5.0");
    
    if correlation_length >= 3.0 && correlation_length <= 5.0 {
        println!("✓ MATCH: Correlation length within crossover range!");
        println!("  → Explicit formula correlation explains crossover");
    } else if correlation_length < 3.0 {
        println!("✗ SHORTER: Correlation length below crossover range");
        println!("  → Explicit formula correlation too fast");
    } else {
        println!("✗ LONGER: Correlation length above crossover range");
        println!("  → Explicit formula correlation too slow");
    }
    
    // Additional analysis
    let ratio = correlation_length / 4.0; // Compare to midpoint of crossover
    println!("Ratio to crossover midpoint (L=4): {:.3}", ratio);
    
    if (ratio - 1.0).abs() < 0.25 {
        println!("  → Strong correlation with crossover scale");
    } else if (ratio - 1.0).abs() < 0.5 {
        println!("  → Moderate correlation with crossover scale");
    } else {
        println!("  → Weak correlation with crossover scale");
    }
    
    // Save results if requested
    if let Some(output_file) = &args.output {
        save_results(&args.zeros_file, correlation_length, output_file)?;
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

fn save_results(zeros_file: &str, correlation_length: f64, output_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    
    let mut file = File::create(output_file)?;
    
    writeln!(file, "# Explicit Formula Correlation Length Analysis")?;
    writeln!(file, "# Zeros file: {}", zeros_file)?;
    writeln!(file, "# Correlation length: {:.6}", correlation_length)?;
    writeln!(file, "")?;
    
    writeln!(file, "# Comparison with L≈3-5 crossover:")?;
    writeln!(file, "crossover_min = 3.0")?;
    writeln!(file, "crossover_max = 5.0")?;
    writeln!(file, "crossover_mid = 4.0")?;
    writeln!(file, "")?;
    
    writeln!(file, "# Results:")?;
    if correlation_length >= 3.0 && correlation_length <= 5.0 {
        writeln!(file, "match = true")?;
        writeln!(file, "conclusion = \"Correlation length within crossover range\"")?;
    } else {
        writeln!(file, "match = false")?;
        writeln!(file, "conclusion = \"Correlation length outside crossover range\"")?;
    }
    
    writeln!(file, "ratio_to_midpoint = {:.6}", correlation_length / 4.0)?;
    
    Ok(())
}
