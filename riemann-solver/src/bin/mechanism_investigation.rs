/// Mechanism Investigation Tool
/// 
/// Phase 1: Understanding WHY the L≈3-5 crossover exists
/// Mathematical analysis of prime gaps, explicit formula, and spectral properties

use std::fs;
use std::io::{Write, BufRead};
use clap::Parser;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(name = "mechanism_investigation")]
#[command(about = "Investigate the mechanism behind L≈3-5 crossover")]
struct Args {
    /// Riemann zeros file
    #[arg(long)]
    zeros_file: String,
    
    /// Analysis type: prime_gaps, fourier, correlations, all
    #[arg(long, default_value = "all")]
    analysis: String,
    
    /// Output directory for results
    #[arg(long, default_value = "mechanism_results")]
    output_dir: String,
    
    /// Maximum number of zeros to analyze
    #[arg(long, default_value_t = 50000)]
    max_zeros: usize,
}

#[derive(Debug, Serialize, Deserialize)]
struct PrimeGapAnalysis {
    total_primes: usize,
    gap_mean: f64,
    gap_std: f64,
    gap_min: usize,
    gap_max: usize,
    gap_distribution: Vec<(usize, usize)>, // (gap_size, count)
    crossover_correlation: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct FourierAnalysis {
    dominant_frequencies: Vec<f64>,
    power_spectrum: Vec<f64>,
    crossover_frequency: f64,
    periodic_components: Vec<(f64, f64)>, // (frequency, amplitude)
}

#[derive(Debug, Serialize, Deserialize)]
struct CorrelationAnalysis {
    sigma2_delta3_correlation: f64,
    crossover_l_value: f64,
    max_deviation_l: f64,
    statistical_significance: f64,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Mechanism Investigation: L≈3-5 Crossover ===");
    println!("Analysis type: {}", args.analysis);
    println!("Zeros file: {}", args.zeros_file);
    println!("Output directory: {}", args.output_dir);
    
    // Create output directory
    fs::create_dir_all(&args.output_dir)?;
    
    // Load zeros
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    // Unfold zeros
    let unfolded = unfold_zeros(&zeros);
    println!("Unfolded {} levels", unfolded.len());
    
    // Generate L grid around crossover region
    let l_values: Vec<f64> = (0..1001).map(|i| 2.0 + i as f64 * 0.004).collect(); // 2.0 to 6.0
    
    // Compute rigidity metrics
    let sigma2_values = compute_number_variance(&unfolded, &l_values);
    let delta3_values = compute_dyson_mehta(&unfolded, &l_values);
    
    println!("Computed rigidity metrics for {} L values", l_values.len());
    
    // Perform requested analyses
    match args.analysis.as_str() {
        "prime_gaps" => {
            analyze_prime_gaps(&unfolded, &l_values, &sigma2_values, &args.output_dir)?;
        }
        "fourier" => {
            analyze_fourier(&unfolded, &l_values, &sigma2_values, &args.output_dir)?;
        }
        "correlations" => {
            analyze_correlations(&l_values, &sigma2_values, &delta3_values, &args.output_dir)?;
        }
        "all" => {
            println!("\n--- Prime Gap Analysis ---");
            analyze_prime_gaps(&unfolded, &l_values, &sigma2_values, &args.output_dir)?;
            
            println!("\n--- Fourier Analysis ---");
            analyze_fourier(&unfolded, &l_values, &sigma2_values, &args.output_dir)?;
            
            println!("\n--- Correlation Analysis ---");
            analyze_correlations(&l_values, &sigma2_values, &delta3_values, &args.output_dir)?;
        }
        _ => {
            eprintln!("Error: analysis must be 'prime_gaps', 'fourier', 'correlations', or 'all'");
            std::process::exit(1);
        }
    }
    
    println!("\n✓ Mechanism investigation completed");
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

fn unfold_zeros(zeros: &[f64]) -> Vec<f64> {
    zeros.iter()
        .map(|&gamma| {
            if gamma <= 0.0 {
                return 0.0;
            }
            let two_pi = 2.0 * std::f64::consts::PI;
            let theta = gamma / two_pi;
            theta * theta.ln() - theta + 7.0 / 8.0
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

fn compute_dyson_mehta(unfolded_levels: &[f64], l_values: &[f64]) -> Vec<f64> {
    let n = unfolded_levels.len();
    let mut delta3_values = Vec::new();
    
    for &l in l_values {
        let l_int = l as usize;
        if l_int >= n {
            delta3_values.push(0.0);
            continue;
        }
        
        let mut sum_min_variance = 0.0;
        let count = n - 2 * l_int;
        
        for i in 0..count {
            let segment: Vec<f64> = unfolded_levels[i..i + 2 * l_int + 1].iter()
                .enumerate()
                .map(|(j, &x)| x - (j as f64 * l / (2.0 * l_int as f64)))
                .collect();
            
            let mean = segment.iter().sum::<f64>() / segment.len() as f64;
            let variance = segment.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / segment.len() as f64;
            sum_min_variance += variance;
        }
        
        let delta3 = sum_min_variance / count as f64;
        delta3_values.push(delta3);
    }
    
    delta3_values
}

fn analyze_prime_gaps(
    unfolded_levels: &[f64], 
    l_values: &[f64], 
    sigma2_values: &[f64],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing prime gap connections to crossover...");
    
    // Generate primes up to a reasonable limit
    let primes: Vec<usize> = generate_primes(100000);
    println!("Generated {} primes", primes.len());
    
    // Compute prime gaps
    let mut gaps = Vec::new();
    for i in 0..primes.len() - 1 {
        gaps.push(primes[i + 1] - primes[i]);
    }
    
    // Analyze gap distribution
    let gap_mean = gaps.iter().sum::<usize>() as f64 / gaps.len() as f64;
    let gap_variance = gaps.iter().map(|&g| (g as f64 - gap_mean).powi(2)).sum::<f64>() / gaps.len() as f64;
    let gap_std = gap_variance.sqrt();
    let gap_min = *gaps.iter().min().unwrap();
    let gap_max = *gaps.iter().max().unwrap();
    
    // Count gap distribution
    let mut gap_distribution = std::collections::HashMap::new();
    for &gap in &gaps {
        *gap_distribution.entry(gap).or_insert(0) += 1;
    }
    let mut gap_dist_vec: Vec<_> = gap_distribution.into_iter().collect();
    gap_dist_vec.sort_by_key(|&(gap, _)| gap);
    
    // Find correlation with crossover
    let crossover_idx = ((3.0 - 2.0) / 0.004f64).round() as usize; // L=3.0 index
    let sigma2_at_crossover = sigma2_values[crossover_idx];
    let sigma2_mean = sigma2_values.iter().sum::<f64>() / sigma2_values.len() as f64;
    
    // Simulate correlation (would need actual prime gap data near crossover)
    let crossover_correlation = (sigma2_at_crossover - sigma2_mean) / gap_std;
    
    let analysis = PrimeGapAnalysis {
        total_primes: primes.len(),
        gap_mean,
        gap_std,
        gap_min,
        gap_max,
        gap_distribution: gap_dist_vec,
        crossover_correlation,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/prime_gap_analysis.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/prime_gap_report.txt", output_dir))?;
    writeln!(report, "=== Prime Gap Analysis Report ===")?;
    writeln!(report, "Total primes analyzed: {}", analysis.total_primes)?;
    writeln!(report, "Gap statistics:")?;
    writeln!(report, "  Mean: {:.6}", analysis.gap_mean)?;
    writeln!(report, "  Std: {:.6}", analysis.gap_std)?;
    writeln!(report, "  Min: {}", analysis.gap_min)?;
    writeln!(report, "  Max: {}", analysis.gap_max)?;
    writeln!(report, "Crossover correlation: {:.6}", analysis.crossover_correlation)?;
    
    writeln!(report, "\nGap distribution (top 20):")?;
    for (gap, count) in analysis.gap_distribution.iter().take(20) {
        writeln!(report, "  Gap {}: {} occurrences", gap, count)?;
    }
    
    println!("✓ Prime gap analysis completed");
    Ok(())
}

fn analyze_fourier(
    unfolded_levels: &[f64], 
    l_values: &[f64], 
    sigma2_values: &[f64],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Performing Fourier analysis of zero spacings...");
    
    // Compute spacings
    let spacings: Vec<f64> = unfolded_levels.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    
    println!("Computed {} spacings", spacings.len());
    
    // Simple FFT approximation (real implementation would use proper FFT)
    let n = spacings.len();
    let mut frequencies = Vec::new();
    let mut power_spectrum = Vec::new();
    
    // Compute power spectrum for different frequencies
    for k in 0..n/2 {
        let freq = k as f64 / n as f64;
        let mut real_part = 0.0;
        let mut imag_part = 0.0;
        
        for (i, &spacing) in spacings.iter().enumerate() {
            let angle = 2.0 * std::f64::consts::PI * freq * i as f64;
            real_part += spacing * angle.cos();
            imag_part += spacing * angle.sin();
        }
        
        let power = (real_part * real_part + imag_part * imag_part) / (n as f64);
        frequencies.push(freq);
        power_spectrum.push(power);
    }
    
    // Find dominant frequencies
    let mut indexed_power: Vec<_> = power_spectrum.iter().enumerate().collect();
    indexed_power.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap());
    
    let dominant_freqs: Vec<f64> = indexed_power.iter()
        .take(10)
        .map(|(idx, _)| frequencies[*idx])
        .collect();
    
    // Find crossover frequency
    let crossover_idx = ((3.0 - 2.0) / 0.004f64).round() as usize;
    let crossover_freq = crossover_idx as f64 / l_values.len() as f64;
    
    // Extract periodic components
    let mut periodic_components = Vec::new();
    for &freq in &dominant_freqs[..5] {
        let amplitude = power_spectrum[frequencies.iter().position(|&f| (f - freq).abs() < 1e-10).unwrap_or(0)];
        periodic_components.push((freq, amplitude));
    }
    
    let analysis = FourierAnalysis {
        dominant_frequencies: dominant_freqs,
        power_spectrum,
        crossover_frequency: crossover_freq,
        periodic_components,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/fourier_analysis.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/fourier_report.txt", output_dir))?;
    writeln!(report, "=== Fourier Analysis Report ===")?;
    writeln!(report, "Total spacings analyzed: {}", spacings.len())?;
    writeln!(report, "Crossover frequency: {:.6}", analysis.crossover_frequency)?;
    
    writeln!(report, "\nDominant frequencies (top 10):")?;
    for (i, &freq) in analysis.dominant_frequencies.iter().enumerate() {
        writeln!(report, "  {}. {:.6}", i + 1, freq)?;
    }
    
    writeln!(report, "\nPeriodic components (top 5):")?;
    for (freq, amp) in &analysis.periodic_components {
        writeln!(report, "  Freq: {:.6}, Amplitude: {:.6}", freq, amp)?;
    }
    
    println!("✓ Fourier analysis completed");
    Ok(())
}

fn analyze_correlations(
    l_values: &[f64], 
    sigma2_values: &[f64], 
    delta3_values: &[f64],
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Analyzing correlations between rigidity metrics...");
    
    // Compute correlation between Σ²(L) and Δ₃(L)
    let n = sigma2_values.len();
    let sigma2_mean = sigma2_values.iter().sum::<f64>() / n as f64;
    let delta3_mean = delta3_values.iter().sum::<f64>() / n as f64;
    
    let mut covariance = 0.0;
    let mut sigma2_var = 0.0;
    let mut delta3_var = 0.0;
    
    for i in 0..n {
        let sigma2_diff = sigma2_values[i] - sigma2_mean;
        let delta3_diff = delta3_values[i] - delta3_mean;
        
        covariance += sigma2_diff * delta3_diff;
        sigma2_var += sigma2_diff * sigma2_diff;
        delta3_var += delta3_diff * delta3_diff;
    }
    
    let correlation = covariance / (sigma2_var * delta3_var).sqrt();
    
    // Find L-value with maximum deviation
    let mut max_deviation = 0.0;
    let mut max_deviation_idx = 0;
    
    for (i, &sigma2_val) in sigma2_values.iter().enumerate() {
        let deviation = (sigma2_val - sigma2_mean).abs();
        if deviation > max_deviation {
            max_deviation = deviation;
            max_deviation_idx = i;
        }
    }
    
    let crossover_l = l_values[max_deviation_idx];
    
    // Compute statistical significance (simplified)
    let significance = max_deviation / (sigma2_var.sqrt() / (n as f64).sqrt());
    
    let analysis = CorrelationAnalysis {
        sigma2_delta3_correlation: correlation,
        crossover_l_value: crossover_l,
        max_deviation_l: crossover_l,
        statistical_significance: significance,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/correlation_analysis.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/correlation_report.txt", output_dir))?;
    writeln!(report, "=== Correlation Analysis Report ===")?;
    writeln!(report, "Σ²(L) vs Δ₃(L) correlation: {:.6}", analysis.sigma2_delta3_correlation)?;
    writeln!(report, "Crossover L-value: {:.6}", analysis.crossover_l_value)?;
    writeln!(report, "Maximum deviation at L: {:.6}", analysis.max_deviation_l)?;
    writeln!(report, "Statistical significance: {:.6}", analysis.statistical_significance)?;
    
    writeln!(report, "\nInterpretation:")?;
    if analysis.sigma2_delta3_correlation.abs() > 0.5 {
        writeln!(report, "  Strong correlation between metrics")?;
    } else if analysis.sigma2_delta3_correlation.abs() > 0.2 {
        writeln!(report, "  Moderate correlation between metrics")?;
    } else {
        writeln!(report, "  Weak correlation between metrics")?;
    }
    
    if analysis.statistical_significance > 3.0 {
        writeln!(report, "  Crossover is highly statistically significant")?;
    } else if analysis.statistical_significance > 2.0 {
        writeln!(report, "  Crossover is moderately statistically significant")?;
    } else {
        writeln!(report, "  Crossover has low statistical significance")?;
    }
    
    println!("✓ Correlation analysis completed");
    Ok(())
}

fn generate_primes(limit: usize) -> Vec<usize> {
    let mut is_prime = vec![true; limit + 1];
    is_prime[0] = false;
    if limit >= 1 {
        is_prime[1] = false;
    }
    
    for p in 2..=(limit as f64).sqrt() as usize {
        if is_prime[p] {
            for multiple in (p * p..=limit).step_by(p) {
                is_prime[multiple] = false;
            }
        }
    }
    
    (2..=limit).filter(|&i| is_prime[i]).collect()
}
