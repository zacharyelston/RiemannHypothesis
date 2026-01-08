/// Validation tool for explicit formula correlation analysis
/// 
/// Tests the correlation computation with known data and checks for reasonableness

use clap::Parser;
use riemann_solver::analysis::explicit_formula::{ExplicitFormula, analyze_correlation_length};

#[derive(Parser)]
#[command(name = "validate_explicit_formula")]
#[command(about = "Validate explicit formula correlation computation")]
struct Args {
    /// File containing Riemann zeros (one per line)
    zeros_file: Option<String>,
    
    /// Use synthetic test data instead of real zeros
    #[arg(long)]
    synthetic: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Explicit Formula Correlation Validation ===");
    
    if args.synthetic {
        test_synthetic_data()?;
    } else if let Some(zeros_file) = args.zeros_file {
        test_real_data(&zeros_file)?;
    } else {
        eprintln!("Error: Must provide either --synthetic or zeros_file");
        std::process::exit(1);
    }
    
    Ok(())
}

fn test_synthetic_data() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing with synthetic zero data...");
    
    // Test 1: Small number of zeros (should be fast correlation)
    let small_zeros = vec![
        14.134725, 21.022040, 25.010858, 30.424876, 32.935062,
        36.917598, 40.918719, 43.327073, 48.005151, 49.773832,
        52.970321, 56.446247, 59.347044, 60.831779, 65.112544,
        67.079811, 69.546401, 72.067158, 75.704691, 77.144840,
        80.0, 81.0, 82.0, 83.0, 84.0, 85.0, 86.0, 87.0, 88.0, 89.0, 90.0,
        91.0, 92.0, 93.0, 94.0, 95.0, 96.0, 97.0, 98.0, 99.0, 100.0
    ];
    let length1 = analyze_correlation_length(&small_zeros)?;
    println!("Small zeros (50): correlation length = {:.3}", length1);
    
    // Test 2: More zeros (should be slower correlation)
    let medium_zeros = vec![
        14.134725, 21.022040, 25.010858, 30.424876, 32.935062,
        36.917598, 40.918719, 43.327073, 48.005151, 49.773832,
        52.970321, 56.446247, 59.347044, 60.831779, 65.112544,
        67.079811, 69.546401, 72.067158, 75.704691, 77.144840
    ];
    let length2 = analyze_correlation_length(&medium_zeros)?;
    println!("Medium zeros (20): correlation length = {:.3}", length2);
    
    // Test 3: Even more zeros
    let mut large_zeros = medium_zeros;
    for i in 0..100 {
        large_zeros.push(80.0 + i as f64 * 0.5);
    }
    let length3 = analyze_correlation_length(&large_zeros)?;
    println!("Large zeros (120): correlation length = {:.3}", length3);
    
    // Validate reasonableness
    println!("\n--- Validation ---");
    
    // Correlation length should generally increase with more zeros
    if length3 > length2 && length2 > length1 {
        println!("✓ Correlation length increases with zero count (expected)");
    } else {
        println!("⚠ Unexpected correlation length pattern");
    }
    
    // All should be < 10 (reasonable range)
    if length1 < 10.0 && length2 < 10.0 && length3 < 10.0 {
        println!("✓ All correlation lengths in reasonable range (< 10)");
    } else {
        println!("⚠ Some correlation lengths outside reasonable range");
    }
    
    // Compare with crossover scale
    println!("\n--- Crossover Comparison ---");
    println!("Crossover scale: 3.0 - 5.0");
    
    for (name, length) in [("Small", length1), ("Medium", length2), ("Large", length3)] {
        if length >= 3.0 && length <= 5.0 {
            println!("{}: {:.3} → WITHIN crossover range", name, length);
        } else {
            println!("{}: {:.3} → OUTSIDE crossover range", name, length);
        }
    }
    
    Ok(())
}

fn test_real_data(zeros_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing with real zeros from: {}", zeros_file);
    
    // Load zeros
    let zeros = load_zeros(zeros_file, 1000)?;
    println!("Loaded {} zeros", zeros.len());
    
    if zeros.len() < 100 {
        eprintln!("Error: Need at least 100 zeros for validation");
        std::process::exit(1);
    }
    
    // Test different subset sizes
    let sizes = [100, 500, 1000];
    
    for &size in &sizes {
        if size <= zeros.len() {
            let subset = &zeros[..size];
            let length = analyze_correlation_length(subset)?;
            println!("{} zeros: correlation length = {:.3}", size, length);
            
            // Check against crossover
            if length >= 3.0 && length <= 5.0 {
                println!("  → WITHIN crossover range ✓");
            } else {
                println!("  → OUTSIDE crossover range ✗");
            }
        }
    }
    
    // Detailed analysis with full dataset
    let full_length = analyze_correlation_length(&zeros)?;
    println!("\n--- Full Dataset Analysis ---");
    println!("Correlation length: {:.3}", full_length);
    
    // Create explicit formula object for detailed correlation analysis
    let formula = ExplicitFormula::new(zeros);
    
    // Use logarithmically spaced x values
    let mut x_values = Vec::new();
    for i in 0..20 {
        let x = 10.0_f64.powi(i - 10); // From 10^-10 to 10^10
        x_values.push(x);
    }
    
    // Compute correlation function
    let correlations = formula.compute_correlation(10.0, 100, &x_values);
    
    println!("\n--- Correlation Function Details ---");
    println!("L range: 0.0 to 10.0");
    
    // Find correlation at key points
    let key_points = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0];
    for &target_l in &key_points {
        if let Some((_, corr)) = correlations.iter().find(|(l, _)| (*l - target_l).abs() < 0.1) {
            println!("C({:.1}) = {:.6}", target_l, corr);
        }
    }
    
    // Check correlation decay
    let initial_corr = correlations[0].1.abs();
    let final_corr = correlations.last().unwrap().1.abs();
    
    println!("\n--- Correlation Decay ---");
    println!("Initial correlation (L≈0): {:.6}", initial_corr);
    println!("Final correlation (L≈10): {:.6}", final_corr);
    
    if final_corr < initial_corr {
        println!("✓ Correlation decays with L (expected)");
    } else {
        println!("⚠ Correlation does not decay properly");
    }
    
    let decay_ratio = final_corr / initial_corr;
    println!("Decay ratio: {:.6}", decay_ratio);
    
    if decay_ratio < 0.1 {
        println!("✓ Significant decay observed");
    } else {
        println!("⚠ Weak decay observed");
    }
    
    Ok(())
}

fn load_zeros(filename: &str, max_zeros: usize) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::{self, BufRead};
    
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
