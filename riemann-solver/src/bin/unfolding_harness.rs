/// Unfolding Validation Harness
/// 
/// Tests unfolding methods for validity before allowing rigidity analysis
/// Implements density and drift tests per GPT-4 recommendations

use std::fs;
use std::io::{Write, BufRead};
use clap::Parser;
use serde::{Serialize, Deserialize};
use nalgebra::DVector;

#[derive(Parser)]
#[command(name = "unfold_test")]
#[command(about = "Test unfolding methods for validity and stability")]
struct Args {
    /// Riemann zeros file
    #[arg(long)]
    zeros_file: String,
    
    /// Unfolding methods to test
    #[arg(long, value_delimiter = ',', default_values = ["rvm", "theta", "poly"])]
    method: Vec<String>,
    
    /// Output directory
    #[arg(long, default_value = "unfold_test_results")]
    output_dir: String,
    
    /// Maximum zeros to test
    #[arg(long, default_value_t = 50000)]
    max_zeros: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnfoldingTest {
    method: String,
    passed: bool,
    mean_spacing: f64,
    spacing_std: f64,
    drift_score: f64,
    density_flatness: f64,
    issues: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TestSuite {
    tests: Vec<UnfoldingTest>,
    accepted_methods: Vec<String>,
    rejected_methods: Vec<String>,
    summary: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Unfolding Validation Harness ===");
    println!("Zeros file: {}", args.zeros_file);
    println!("Methods: {:?}", args.method);
    println!("Output: {}", args.output_dir);
    
    // Create output directory
    fs::create_dir_all(&args.output_dir)?;
    
    // Load zeros
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    // Test each unfolding method
    let mut tests = Vec::new();
    for method in &args.method {
        println!("\n--- Testing {} unfolding ---", method);
        let test = test_unfolding_method(&zeros, method)?;
        tests.push(test);
    }
    
    // Determine accepted/rejected methods
    let accepted: Vec<String> = tests.iter()
        .filter(|t| t.passed)
        .map(|t| t.method.clone())
        .collect();
    
    let rejected: Vec<String> = tests.iter()
        .filter(|t| !t.passed)
        .map(|t| t.method.clone())
        .collect();
    
    // Create summary
    let summary = if accepted.is_empty() {
        "❌ NO UNFOLDING METHODS PASSED VALIDATION".to_string()
    } else if rejected.is_empty() {
        "✅ ALL UNFOLDING METHODS PASSED VALIDATION".to_string()
    } else {
        format!("⚠️ MIXED RESULTS: {} passed, {} rejected", accepted.len(), rejected.len())
    };
    
    let test_suite = TestSuite {
        tests,
        accepted_methods: accepted.clone(),
        rejected_methods: rejected.clone(),
        summary,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&test_suite)?;
    fs::write(format!("{}/unfold_test_results.json", args.output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/unfold_test_report.txt", args.output_dir))?;
    writeln!(report, "=== Unfolding Validation Report ===")?;
    writeln!(report, "Summary: {}", test_suite.summary)?;
    
    writeln!(report, "\n--- Test Results ---")?;
    for test in &test_suite.tests {
        writeln!(report, "\nMethod: {}", test.method)?;
        writeln!(report, "Status: {}", if test.passed { "✅ PASSED" } else { "❌ REJECTED" })?;
        writeln!(report, "Mean spacing: {:.6}", test.mean_spacing)?;
        writeln!(report, "Spacing std: {:.6}", test.spacing_std)?;
        writeln!(report, "Drift score: {:.6}", test.drift_score)?;
        writeln!(report, "Density flatness: {:.6}", test.density_flatness)?;
        
        if !test.issues.is_empty() {
            writeln!(report, "Issues:")?;
            for issue in &test.issues {
                writeln!(report, "  - {}", issue)?;
            }
        }
    }
    
    writeln!(report, "\n--- Recommendations ---")?;
    if accepted.is_empty() {
        writeln!(report, "❌ NO VALID UNFOLDING METHODS FOUND")?;
        writeln!(report, "   Rigidity analysis should NOT proceed")?;
        writeln!(report, "   Review unfolding implementations")?;
    } else {
        writeln!(report, "✅ ACCEPTED METHODS FOR RIGIDITY ANALYSIS:")?;
        for method in &accepted {
            writeln!(report, "   - {}", method)?;
        }
        
        if !rejected.is_empty() {
            writeln!(report, "\n❌ REJECTED METHODS:")?;
            for method in &rejected {
                writeln!(report, "   - {}", method)?;
            }
        }
    }
    
    println!("\n✓ Unfolding validation completed");
    println!("Summary: {}", test_suite.summary);
    
    if accepted.is_empty() {
        println!("⚠️ WARNING: No valid unfolding methods. Rigidity analysis not recommended.");
    }
    
    Ok(())
}

fn load_zeros(filename: &str, max_zeros: usize) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
    let file = fs::File::open(filename)?;
    let reader = std::io::BufReader::new(file);
    
    let mut zeros = Vec::new();
    for line in reader.lines() {
        let line: String = line?;
        if let Ok(zero) = line.trim().parse::<f64>() {
            zeros.push(zero);
            if zeros.len() >= max_zeros {
                break;
            }
        }
    }
    
    Ok(zeros)
}

fn test_unfolding_method(zeros: &[f64], method: &str) -> Result<UnfoldingTest, Box<dyn std::error::Error>> {
    let unfolded = match method {
        "rvm" => unfold_rvm(zeros),
        "theta" => unfold_theta(zeros),
        "poly" => unfold_polynomial(zeros),
        _ => return Err(format!("Unknown unfolding method: {}", method).into()),
    };
    
    // Test 1: Density test (mean spacing ≈ 1)
    let spacings = compute_spacings(&unfolded);
    let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
    let spacing_std = {
        let mean = mean_spacing;
        spacings.iter().map(|&s| (s - mean).powi(2)).sum::<f64>() / spacings.len() as f64
    }.sqrt();
    
    // Test 2: Drift test (δ_n = x_n - n should be stationary)
    let drift_score = compute_drift_score(&unfolded);
    
    // Test 3: Density flatness test
    let density_flatness = compute_density_flatness(&unfolded);
    
    // Evaluate results
    let mut issues = Vec::new();
    let mut passed = true;
    
    // Check mean spacing
    if (mean_spacing - 1.0).abs() > 0.1 {
        issues.push(format!("Mean spacing {:.3} not close to 1.0", mean_spacing));
        passed = false;
    }
    
    // Check drift
    if drift_score > 0.1 {
        issues.push(format!("High drift score {:.3}", drift_score));
        passed = false;
    }
    
    // Check density flatness
    if density_flatness < 0.8 {
        issues.push(format!("Poor density flatness {:.3}", density_flatness));
        passed = false;
    }
    
    // Additional method-specific checks
    match method {
        "poly" => {
            // Polynomial unfolding is always suspicious
            issues.push("Local polynomial unfolding uses neighboring zeros".to_string());
            if drift_score > 0.05 {
                passed = false;
            }
        }
        "theta" => {
            // Check if theta approximation is reasonable
            if spacing_std > 0.5 {
                issues.push("High spacing variance in theta unfolding".to_string());
                passed = false;
            }
        }
        "rvm" => {
            // RVM should be most reliable
            if drift_score > 0.05 {
                issues.push("Unexpected drift in RVM unfolding".to_string());
                passed = false;
            }
        }
        _ => {}
    }
    
    Ok(UnfoldingTest {
        method: method.to_string(),
        passed,
        mean_spacing,
        spacing_std,
        drift_score,
        density_flatness,
        issues,
    })
}

fn unfold_rvm(zeros: &[f64]) -> Vec<f64> {
    zeros.iter()
        .map(|&gamma| {
            if gamma <= 0.0 {
                return 0.0;
            }
            let two_pi = 2.0 * std::f64::consts::PI;
            let t_over_2pi = gamma / two_pi;
            t_over_2pi * (t_over_2pi.ln() - 1.0)
        })
        .collect()
}

fn unfold_theta(zeros: &[f64]) -> Vec<f64> {
    zeros.iter()
        .map(|&gamma| {
            if gamma <= 0.0 {
                return 0.0;
            }
            // Approximate theta function unfolding
            let t = gamma;
            let theta_approx = t * (t.ln() / (2.0 * std::f64::consts::PI) - 1.0);
            theta_approx
        })
        .collect()
}

fn unfold_polynomial(zeros: &[f64]) -> Vec<f64> {
    let n = zeros.len();
    let window_size = 1000.min(n / 10);
    
    zeros.iter().enumerate()
        .map(|(i, &_gamma)| {
            if i < window_size / 2 || i >= n - window_size / 2 {
                return i as f64;
            }
            
            let start = i - window_size / 2;
            let end = i + window_size / 2;
            
            let sum_x: f64 = (start..end).map(|j| j as f64).sum();
            let sum_y: f64 = zeros[start..end].iter().sum();
            let sum_xx: f64 = (start..end).map(|j| (j as f64).powi(2)).sum();
            let sum_xy: f64 = (start..end).zip(&zeros[start..end]).map(|(j, &y)| j as f64 * y).sum();
            
            let n_window = window_size as f64;
            let slope = (n_window * sum_xy - sum_x * sum_y) / (n_window * sum_xx - sum_x * sum_x);
            let intercept = (sum_y - slope * sum_x) / n_window;
            
            slope * i as f64 + intercept
        })
        .collect()
}

fn compute_spacings(levels: &[f64]) -> Vec<f64> {
    if levels.len() < 2 {
        return Vec::new();
    }
    
    levels.windows(2)
        .map(|w| w[1] - w[0])
        .collect()
}

fn compute_drift_score(unfolded: &[f64]) -> f64 {
    // Compute δ_n = x_n - n
    let deltas: Vec<f64> = unfolded.iter()
        .enumerate()
        .map(|(i, &x)| x - i as f64)
        .collect();
    
    // Check for linear trend in deltas
    let n = deltas.len();
    if n < 10 {
        return 1.0; // High drift for small samples
    }
    
    let sum_x: f64 = (0..n).map(|i| i as f64).sum();
    let sum_y: f64 = deltas.iter().sum();
    let sum_xx: f64 = (0..n).map(|i| (i as f64).powi(2)).sum();
    let sum_xy: f64 = (0..n).zip(&deltas).map(|(i, &y)| i as f64 * y).sum();
    
    let slope = (n as f64 * sum_xy - sum_x * sum_y) / (n as f64 * sum_xx - sum_x * sum_x);
    
    slope.abs() // Return absolute slope as drift score
}

fn compute_density_flatness(unfolded: &[f64]) -> f64 {
    // Divide unfolded levels into bins and check density variation
    let n_bins = 20;
    let n = unfolded.len();
    let bin_size = n / n_bins;
    
    if bin_size < 10 {
        return 0.0; // Poor flatness for small bins
    }
    
    let mut densities = Vec::new();
    for i in 0..n_bins {
        let start = i * bin_size;
        let end = ((i + 1) * bin_size).min(n);
        
        if end > start {
            let count = end - start;
            let range = unfolded[end - 1] - unfolded[start];
            let density = count as f64 / range;
            densities.push(density);
        }
    }
    
    if densities.is_empty() {
        return 0.0;
    }
    
    let mean_density = densities.iter().sum::<f64>() / densities.len() as f64;
    let variance = densities.iter()
        .map(|&d| (d - mean_density).powi(2))
        .sum::<f64>() / densities.len() as f64;
    
    let cv = variance.sqrt() / mean_density; // Coefficient of variation
    
    // Convert to flatness score (lower CV = higher flatness)
    (1.0 / (1.0 + cv)).min(1.0)
}
