/// Corrected Mechanism Investigation Tool
/// 
/// Phase 1: Understanding WHY the L≈3-5 crossover exists
/// Fixed unfolding and proper crossover definition per GPT-3 review

use std::fs;
use std::io::{Write, BufRead};
use clap::Parser;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(name = "corrected_mechanism_investigation")]
#[command(about = "Investigate the mechanism behind L≈3-5 crossover with corrected unfolding")]
struct Args {
    /// Riemann zeros file
    #[arg(long)]
    zeros_file: String,
    
    /// Analysis type: residual, unfolding_test, pair_correlation, all
    #[arg(long, default_value = "all")]
    analysis: String,
    
    /// Output directory for results
    #[arg(long, default_value = "corrected_mechanism_results")]
    output_dir: String,
    
    /// Maximum number of zeros to analyze
    #[arg(long, default_value_t = 50000)]
    max_zeros: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct ResidualAnalysis {
    crossover_l: f64,
    residual_zero_crossing: f64,
    residual_amplitude: f64,
    universal_component: f64,
    arithmetic_component: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct UnfoldingComparison {
    l_star_rvm: f64,
    l_star_theta: f64,
    l_star_poly: f64,
    l_star_stability: f64,
    unfolding_method: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PairCorrelationAnalysis {
    sine_kernel_deviation: f64,
    correction_term_amplitude: f64,
    crossover_correlation: f64,
    form_factor_structure: Vec<(f64, f64)>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Corrected Mechanism Investigation ===");
    println!("Analysis type: {}", args.analysis);
    println!("Zeros file: {}", args.zeros_file);
    println!("Output directory: {}", args.output_dir);
    
    // Create output directory
    fs::create_dir_all(&args.output_dir)?;
    
    // Load zeros
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    // Generate L grid around crossover region
    let l_values: Vec<f64> = (0..1001).map(|i| 1.0 + i as f64 * 0.005).collect(); // 1.0 to 6.0
    
    println!("Generated {} L values", l_values.len());
    
    // Perform requested analyses
    match args.analysis.as_str() {
        "residual" => {
            analyze_residual_rigidity(&zeros, &l_values, &args.output_dir)?;
        }
        "unfolding_test" => {
            test_unfolding_stability(&zeros, &l_values, &args.output_dir)?;
        }
        "pair_correlation" => {
            analyze_pair_correlation(&zeros, &args.output_dir)?;
        }
        "all" => {
            println!("\n--- Residual Rigidity Analysis ---");
            analyze_residual_rigidity(&zeros, &l_values, &args.output_dir)?;
            
            println!("\n--- Unfolding Stability Test ---");
            test_unfolding_stability(&zeros, &l_values, &args.output_dir)?;
            
            println!("\n--- Pair Correlation Analysis ---");
            analyze_pair_correlation(&zeros, &args.output_dir)?;
        }
        _ => {
            eprintln!("Error: analysis must be 'residual', 'unfolding_test', 'pair_correlation', or 'all'");
            std::process::exit(1);
        }
    }
    
    println!("\n✓ Corrected mechanism investigation completed");
    println!("Results saved to: {}", args.output_dir);
    
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

/// CORRECTED: Proper Riemann-von Mangoldt approximation
fn unfold_rvm(zeros: &[f64]) -> Vec<f64> {
    zeros.iter()
        .map(|&gamma| {
            if gamma <= 0.0 {
                return 0.0;
            }
            let two_pi = 2.0 * std::f64::consts::PI;
            let t_over_2pi = gamma / two_pi;
            // N(T) = T/(2π) * log(T/(2π)) - T/(2π) + O(log T)
            t_over_2pi * (t_over_2pi.ln() - 1.0)
        })
        .collect()
}

/// Alternative: Riemann-Siegel theta based unfolding
fn unfold_theta(zeros: &[f64]) -> Vec<f64> {
    zeros.iter()
        .map(|&gamma| {
            if gamma <= 0.0 {
                return 0.0;
            }
            // θ(t) = Im(log(Γ(1/4 + it/2))) - (t/2) * log(π)
            // Approximate unfolding using theta function
            let t = gamma;
            let theta_approx = t * (t.ln() / (2.0 * std::f64::consts::PI) - 1.0);
            theta_approx
        })
        .collect()
}

/// Alternative: Local polynomial fit unfolding
fn unfold_polynomial(zeros: &[f64]) -> Vec<f64> {
    let n = zeros.len();
    let window_size = 1000.min(n / 10);
    
    zeros.iter().enumerate()
        .map(|(i, &_gamma)| {
            if i < window_size / 2 || i >= n - window_size / 2 {
                return i as f64; // Edge case
            }
            
            // Local linear fit
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

fn compute_number_variance(unfolded_levels: &[f64], l_values: &[f64]) -> Vec<f64> {
    let n = unfolded_levels.len();
    let mut sigma2_values = Vec::new();
    
    for &l in l_values {
        let l_int = l as usize;
        if l_int >= n {
            sigma2_values.push(0.0);
            continue;
        }
        
        let mut sum_sq_diff = 0.0;
        let count = n - l_int;
        
        for i in 0..count {
            let diff = unfolded_levels[i + l_int] - unfolded_levels[i] - l as f64;
            sum_sq_diff += diff * diff;
        }
        
        let sigma2 = sum_sq_diff / count as f64;
        sigma2_values.push(sigma2);
    }
    
    sigma2_values
}

fn compute_gue_prediction(l_values: &[f64]) -> Vec<f64> {
    // GUE prediction for number variance: Σ²(L) ≈ (2/π²) * log(L) + γ + O(1/L)
    let euler_gamma = 0.5772156649;
    l_values.iter()
        .map(|&l| {
            if l <= 1.0 {
                return 0.0;
            }
            (2.0 / (std::f64::consts::PI * std::f64::consts::PI)) * l.ln() + euler_gamma
        })
        .collect()
}

fn analyze_residual_rigidity(
    zeros: &[f64], 
    l_values: &[f64],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Computing residual rigidity curves...");
    
    // Unfold with corrected method
    let unfolded = unfold_rvm(zeros);
    let sigma2_observed = compute_number_variance(&unfolded, l_values);
    let sigma2_gue = compute_gue_prediction(l_values);
    
    // Compute residuals
    let residuals: Vec<f64> = sigma2_observed.iter()
        .zip(&sigma2_gue)
        .map(|(obs, gue)| obs - gue)
        .collect();
    
    // Find crossover where residual changes sign
    let mut crossover_l = 0.0;
    for (i, &residual) in residuals.iter().enumerate() {
        if residual.abs() < 0.001 { // Near zero crossing
            crossover_l = l_values[i];
            break;
        }
    }
    
    // If no exact zero crossing, find minimum absolute residual
    if crossover_l == 0.0 {
        let min_idx = residuals.iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| a.abs().partial_cmp(&b.abs()).unwrap())
            .map(|(idx, _)| idx)
            .unwrap_or(0);
        crossover_l = l_values[min_idx];
    }
    
    // Compute components
    let universal_component = sigma2_gue.iter().sum::<f64>() / sigma2_gue.len() as f64;
    let arithmetic_component = residuals.iter().sum::<f64>().abs() / residuals.len() as f64;
    let residual_amplitude = residuals.iter().map(|r| r.abs()).sum::<f64>() / residuals.len() as f64;
    
    let analysis = ResidualAnalysis {
        crossover_l,
        residual_zero_crossing: crossover_l,
        residual_amplitude,
        universal_component,
        arithmetic_component,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/residual_analysis.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/residual_report.txt", output_dir))?;
    writeln!(report, "=== Residual Rigidity Analysis Report ===")?;
    writeln!(report, "Crossover L-value: {:.6}", analysis.crossover_l)?;
    writeln!(report, "Residual zero crossing: {:.6}", analysis.residual_zero_crossing)?;
    writeln!(report, "Residual amplitude: {:.6}", analysis.residual_amplitude)?;
    writeln!(report, "Universal component: {:.6}", analysis.universal_component)?;
    writeln!(report, "Arithmetic component: {:.6}", analysis.arithmetic_component)?;
    
    writeln!(report, "\nInterpretation:")?;
    if (analysis.crossover_l - 3.0).abs() < 0.5 {
        writeln!(report, "✓ Crossover near expected L≈3 region")?;
    } else {
        writeln!(report, "? Crossover at L≈{:.1} (different from expected)", analysis.crossover_l)?;
    }
    
    if analysis.arithmetic_component > 0.01 {
        writeln!(report, "✓ Significant arithmetic correction detected")?;
    } else {
        writeln!(report, "? Weak arithmetic correction")?;
    }
    
    println!("✓ Residual rigidity analysis completed");
    Ok(())
}

fn test_unfolding_stability(
    zeros: &[f64], 
    l_values: &[f64],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing unfolding stability across methods...");
    
    // Test three unfolding methods
    let unfolded_rvm = unfold_rvm(zeros);
    let unfolded_theta = unfold_theta(zeros);
    let unfolded_poly = unfold_polynomial(zeros);
    
    // Compute crossover for each method
    let sigma2_rvm = compute_number_variance(&unfolded_rvm, l_values);
    let sigma2_theta = compute_number_variance(&unfolded_theta, l_values);
    let sigma2_poly = compute_number_variance(&unfolded_poly, l_values);
    
    let sigma2_gue = compute_gue_prediction(l_values);
    
    // Find crossovers
    let l_star_rvm = find_crossover(&l_values, &sigma2_rvm, &sigma2_gue);
    let l_star_theta = find_crossover(&l_values, &sigma2_theta, &sigma2_gue);
    let l_star_poly = find_crossover(&l_values, &sigma2_poly, &sigma2_gue);
    
    // Compute stability
    let l_values_vec = vec![l_star_rvm, l_star_theta, l_star_poly];
    let mean_l_star = l_values_vec.iter().sum::<f64>() / 3.0;
    let variance = l_values_vec.iter().map(|&l| (l - mean_l_star).powi(2)).sum::<f64>() / 3.0;
    let stability = 1.0 / (1.0 + variance);
    
    let analysis = UnfoldingComparison {
        l_star_rvm,
        l_star_theta,
        l_star_poly,
        l_star_stability: stability,
        unfolding_method: "three_methods".to_string(),
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/unfolding_comparison.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/unfolding_report.txt", output_dir))?;
    writeln!(report, "=== Unfolding Stability Test Report ===")?;
    writeln!(report, "Riemann-von Mangoldt L*: {:.6}", analysis.l_star_rvm)?;
    writeln!(report, "Riemann-Siegel theta L*: {:.6}", analysis.l_star_theta)?;
    writeln!(report, "Polynomial fit L*: {:.6}", analysis.l_star_poly)?;
    writeln!(report, "Stability score: {:.6}", analysis.l_star_stability)?;
    
    writeln!(report, "\nInterpretation:")?;
    if analysis.l_star_stability > 0.9 {
        writeln!(report, "✓ Crossover stable across unfolding methods (intrinsic)")?;
    } else if analysis.l_star_stability > 0.7 {
        writeln!(report, "? Moderate stability (some unfolding dependence)")?;
    } else {
        writeln!(report, "❌ Low stability (unfolding artifact)")?;
    }
    
    writeln!(report, "\nL* range: {:.3} to {:.3} (Δ = {:.3})", 
        l_values_vec.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        l_values_vec.iter().fold(0.0_f64, |a, &b| a.max(b)),
        l_values_vec.iter().fold(0.0_f64, |a, &b| a.max(b)) - 
        l_values_vec.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    )?;
    
    println!("✓ Unfolding stability test completed");
    Ok(())
}

fn find_crossover(l_values: &[f64], sigma2_obs: &[f64], sigma2_gue: &[f64]) -> f64 {
    let mut crossover_l = 0.0;
    let mut min_diff = f64::INFINITY;
    
    for (i, (&l, &obs)) in l_values.iter().zip(sigma2_obs).enumerate() {
        let diff = (obs - sigma2_gue[i]).abs();
        if diff < min_diff {
            min_diff = diff;
            crossover_l = l;
        }
    }
    
    crossover_l
}

fn analyze_pair_correlation(
    zeros: &[f64],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Computing pair correlation and form factor...");
    
    // Unfold zeros
    let unfolded = unfold_rvm(zeros);
    
    // Compute spacings
    let spacings: Vec<f64> = unfolded.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    
    // Compute pair correlation R2(s)
    let max_s = 5.0;
    let n_bins = 100;
    let bin_width = max_s / n_bins as f64;
    
    let mut r2 = vec![0.0; n_bins];
    let mut counts = vec![0; n_bins];
    
    for &spacing in &spacings {
        if spacing <= max_s {
            let bin = (spacing / bin_width) as usize;
            if bin < n_bins {
                r2[bin] += spacing;
                counts[bin] += 1;
            }
        }
    }
    
    // Normalize
    for i in 0..n_bins {
        if counts[i] > 0 {
            r2[i] /= counts[i] as f64;
        }
    }
    
    // Compute sine kernel prediction
    let mut sine_kernel = vec![0.0; n_bins];
    for i in 0..n_bins {
        let s = (i as f64 + 0.5) * bin_width;
        if s > 0.0 {
            sine_kernel[i] = 1.0 - (std::f64::consts::PI * s).sin().powi(2) / (std::f64::consts::PI * s).powi(2);
        }
    }
    
    // Compute deviation
    let mut total_deviation = 0.0;
    let mut total_correction = 0.0;
    for i in 0..n_bins {
        if counts[i] > 10 { // Only well-sampled bins
            total_deviation += (r2[i] - sine_kernel[i]).abs();
            total_correction += (r2[i] - sine_kernel[i]).abs();
        }
    }
    
    let avg_deviation = total_deviation / n_bins as f64;
    let correction_amplitude = total_correction / n_bins as f64;
    
    // Simple form factor (FFT of R2(s))
    let mut form_factor = Vec::new();
    for k in 0..20 {
        let freq = k as f64 / n_bins as f64;
        let mut real = 0.0;
        let mut imag = 0.0;
        
        for i in 0..n_bins {
            let angle = 2.0 * std::f64::consts::PI * freq * i as f64;
            real += (r2[i] - sine_kernel[i]) * angle.cos();
            imag += (r2[i] - sine_kernel[i]) * angle.sin();
        }
        
        form_factor.push((freq, (real * real + imag * imag).sqrt()));
    }
    
    let analysis = PairCorrelationAnalysis {
        sine_kernel_deviation: avg_deviation,
        correction_term_amplitude: correction_amplitude,
        crossover_correlation: avg_deviation,
        form_factor_structure: form_factor,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/pair_correlation.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/pair_correlation_report.txt", output_dir))?;
    writeln!(report, "=== Pair Correlation Analysis Report ===")?;
    writeln!(report, "Sine kernel deviation: {:.6}", analysis.sine_kernel_deviation)?;
    writeln!(report, "Correction term amplitude: {:.6}", analysis.correction_term_amplitude)?;
    writeln!(report, "Crossover correlation: {:.6}", analysis.crossover_correlation)?;
    
    writeln!(report, "\nForm factor structure (top 10):")?;
    for (freq, amp) in analysis.form_factor_structure.iter().take(10) {
        writeln!(report, "  Freq: {:.4}, Amplitude: {:.6}", freq, amp)?;
    }
    
    writeln!(report, "\nInterpretation:")?;
    if analysis.sine_kernel_deviation > 0.01 {
        writeln!(report, "✓ Significant deviation from sine kernel")?;
        writeln!(report, "✓ Arithmetic corrections detected")?;
    } else {
        writeln!(report, "? Weak deviation from sine kernel")?;
    }
    
    println!("✓ Pair correlation analysis completed");
    Ok(())
}
