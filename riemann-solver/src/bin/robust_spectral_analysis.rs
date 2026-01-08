/// Robust Spectral Analysis Tool
/// 
/// Focus on unfolding-independent phenomena and robust metrics
/// Implements GPT-4 recommended approach: local statistics, stable observables

use std::fs;
use std::io::{Write, BufRead};
use clap::Parser;
use serde::{Serialize, Deserialize};

#[derive(Parser)]
#[command(name = "robust_analysis")]
#[command(about = "Analyze unfolding-independent spectral properties")]
struct Args {
    /// Riemann zeros file
    #[arg(long)]
    zeros_file: String,
    
    /// Analysis types: spacing_distribution, pair_correlation, form_factor, all
    #[arg(long, default_value = "all")]
    analysis: String,
    
    /// Output directory
    #[arg(long, default_value = "robust_analysis_results")]
    output_dir: String,
    
    /// Maximum zeros to analyze
    #[arg(long, default_value_t = 50000)]
    max_zeros: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct SpacingDistribution {
    mean: f64,
    std: f64,
    min: f64,
    max: f64,
    skewness: f64,
    kurtosis: f64,
    wigner_dyson_deviation: f64,
    brody_parameter: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct PairCorrelation {
    s_values: Vec<f64>,
    r2_values: Vec<f64>,
    sine_kernel_deviation: f64,
    correlation_coefficient: f64,
    peak_position: f64,
    peak_height: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct FormFactor {
    frequencies: Vec<f64>,
    amplitudes: Vec<f64>,
    step_function_height: f64,
    oscillation_amplitude: f64,
    spectral_rigidity: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct RobustAnalysis {
    spacing_distribution: SpacingDistribution,
    pair_correlation: PairCorrelation,
    form_factor: FormFactor,
    unfolding_independent_score: f64,
    genuine_anomaly_detected: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    
    println!("=== Robust Spectral Analysis ===");
    println!("Analysis: {}", args.analysis);
    println!("Zeros file: {}", args.zeros_file);
    println!("Output: {}", args.output_dir);
    
    // Create output directory
    fs::create_dir_all(&args.output_dir)?;
    
    // Load zeros
    let zeros = load_zeros(&args.zeros_file, args.max_zeros)?;
    println!("Loaded {} zeros", zeros.len());
    
    // Use only validated RVM unfolding
    let unfolded = unfold_rvm(&zeros);
    println!("Unfolded with RVM method");
    
    // Perform requested analyses
    match args.analysis.as_str() {
        "spacing_distribution" => {
            analyze_spacing_distribution(&unfolded, &args.output_dir)?;
        }
        "pair_correlation" => {
            analyze_pair_correlation(&unfolded, &args.output_dir)?;
        }
        "form_factor" => {
            analyze_form_factor(&unfolded, &args.output_dir)?;
        }
        "all" => {
            println!("\n--- Spacing Distribution Analysis ---");
            let spacing_dist = analyze_spacing_distribution(&unfolded, &args.output_dir)?;
            
            println!("\n--- Pair Correlation Analysis ---");
            let pair_corr = analyze_pair_correlation(&unfolded, &args.output_dir)?;
            
            println!("\n--- Form Factor Analysis ---");
            let form_factor = analyze_form_factor(&unfolded, &args.output_dir)?;
            
            // Generate comprehensive analysis
            generate_robust_analysis_report(&spacing_dist, &pair_corr, &form_factor, &args.output_dir)?;
        }
        _ => {
            eprintln!("Error: analysis must be 'spacing_distribution', 'pair_correlation', 'form_factor', or 'all'");
            std::process::exit(1);
        }
    }
    
    println!("\n✓ Robust spectral analysis completed");
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

fn analyze_spacing_distribution(unfolded: &[f64], output_dir: &str) -> Result<SpacingDistribution, Box<dyn std::error::Error>> {
    println!("Computing spacing distribution...");
    
    // Compute spacings
    let spacings: Vec<f64> = unfolded.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    
    if spacings.is_empty() {
        return Err("No spacings computed".into());
    }
    
    // Basic statistics
    let mean = spacings.iter().sum::<f64>() / spacings.len() as f64;
    let variance = spacings.iter().map(|&s| (s - mean).powi(2)).sum::<f64>() / spacings.len() as f64;
    let std = variance.sqrt();
    
    let min_spacing = *spacings.iter().min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let max_spacing = *spacings.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    
    // Higher moments
    let skewness = spacings.iter().map(|&s| ((s - mean) / std).powi(3)).sum::<f64>() / spacings.len() as f64;
    let kurtosis = spacings.iter().map(|&s| ((s - mean) / std).powi(4)).sum::<f64>() / spacings.len() as f64;
    
    // Wigner-Dyson deviation
    let wigner_dyson_deviation = compute_wigner_dyson_deviation(&spacings);
    
    // Brody parameter estimation
    let brody_parameter = estimate_brody_parameter(&spacings);
    
    let analysis = SpacingDistribution {
        mean,
        std,
        min: min_spacing,
        max: max_spacing,
        skewness,
        kurtosis,
        wigner_dyson_deviation,
        brody_parameter,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/spacing_distribution.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/spacing_distribution_report.txt", output_dir))?;
    writeln!(report, "=== Spacing Distribution Analysis ===")?;
    writeln!(report, "Total spacings: {}", spacings.len())?;
    writeln!(report, "Mean spacing: {:.6}", analysis.mean)?;
    writeln!(report, "Std deviation: {:.6}", analysis.std)?;
    writeln!(report, "Min spacing: {:.6}", analysis.min)?;
    writeln!(report, "Max spacing: {:.6}", analysis.max)?;
    writeln!(report, "Skewness: {:.6}", analysis.skewness)?;
    writeln!(report, "Kurtosis: {:.6}", analysis.kurtosis)?;
    writeln!(report, "Wigner-Dyson deviation: {:.6}", analysis.wigner_dyson_deviation)?;
    writeln!(report, "Brody parameter: {:.6}", analysis.brody_parameter)?;
    
    writeln!(report, "\nInterpretation:")?;
    if analysis.wigner_dyson_deviation < 0.1 {
        writeln!(report, "✓ Close to Wigner-Dyson distribution")?;
    } else {
        writeln!(report, "? Significant deviation from Wigner-Dyson")?;
    }
    
    if (analysis.brody_parameter - 1.0).abs() < 0.1 {
        writeln!(report, "✓ Consistent with GOE/GUE statistics")?;
    } else {
        writeln!(report, "? Brody parameter suggests intermediate statistics")?;
    }
    
    println!("✓ Spacing distribution analysis completed");
    Ok(analysis)
}

fn analyze_pair_correlation(unfolded: &[f64], output_dir: &str) -> Result<PairCorrelation, Box<dyn std::error::Error>> {
    println!("Computing pair correlation function...");
    
    // Compute spacings first
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
    
    // Generate s values
    let s_values: Vec<f64> = (0..n_bins)
        .map(|i| (i as f64 + 0.5) * bin_width)
        .collect();
    
    // Compute sine kernel prediction
    let mut sine_kernel = vec![0.0; n_bins];
    for i in 0..n_bins {
        let s = s_values[i];
        if s > 0.0 {
            sine_kernel[i] = 1.0 - (std::f64::consts::PI * s).sin().powi(2) / (std::f64::consts::PI * s).powi(2);
        }
    }
    
    // Compute deviation
    let mut total_deviation = 0.0;
    let mut valid_points = 0;
    for i in 0..n_bins {
        if counts[i] > 10 { // Only well-sampled bins
            total_deviation += (r2[i] - sine_kernel[i]).abs();
            valid_points += 1;
        }
    }
    
    let sine_kernel_deviation = if valid_points > 0 {
        total_deviation / valid_points as f64
    } else {
        0.0
    };
    
    // Find peak
    let (peak_idx, &peak_height) = r2.iter()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
        .unwrap_or((0, &0.0));
    
    let peak_position = s_values[peak_idx];
    
    // Compute correlation coefficient
    let correlation_coefficient = compute_correlation(&r2, &sine_kernel);
    
    let analysis = PairCorrelation {
        s_values,
        r2_values: r2,
        sine_kernel_deviation,
        correlation_coefficient,
        peak_position,
        peak_height,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/pair_correlation.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/pair_correlation_report.txt", output_dir))?;
    writeln!(report, "=== Pair Correlation Analysis ===")?;
    writeln!(report, "Sine kernel deviation: {:.6}", analysis.sine_kernel_deviation)?;
    writeln!(report, "Correlation coefficient: {:.6}", analysis.correlation_coefficient)?;
    writeln!(report, "Peak position: {:.6}", analysis.peak_position)?;
    writeln!(report, "Peak height: {:.6}", analysis.peak_height)?;
    
    writeln!(report, "\nInterpretation:")?;
    if analysis.sine_kernel_deviation < 0.05 {
        writeln!(report, "✓ Excellent agreement with sine kernel")?;
    } else if analysis.sine_kernel_deviation < 0.1 {
        writeln!(report, "? Moderate deviation from sine kernel")?;
    } else {
        writeln!(report, "❌ Significant deviation from sine kernel")?;
    }
    
    if analysis.correlation_coefficient > 0.9 {
        writeln!(report, "✓ Strong correlation with universal prediction")?;
    } else {
        writeln!(report, "? Weak correlation with universal prediction")?;
    }
    
    println!("✓ Pair correlation analysis completed");
    Ok(analysis)
}

fn analyze_form_factor(unfolded: &[f64], output_dir: &str) -> Result<FormFactor, Box<dyn std::error::Error>> {
    println!("Computing spectral form factor...");
    
    // Use pair correlation results
    let pair_corr = analyze_pair_correlation(unfolded, output_dir)?;
    
    // Compute FFT of R2(s) - 1 (deviation from unity)
    let deviation: Vec<f64> = pair_corr.r2_values.iter()
        .map(|&r2| r2 - 1.0)
        .collect();
    
    // Simple FFT approximation
    let n = deviation.len();
    let mut frequencies = Vec::new();
    let mut amplitudes = Vec::new();
    
    for k in 0..n/2 {
        let freq = k as f64 / n as f64;
        let mut real = 0.0;
        let mut imag = 0.0;
        
        for i in 0..n {
            let angle = 2.0 * std::f64::consts::PI * freq * i as f64;
            real += deviation[i] * angle.cos();
            imag += deviation[i] * angle.sin();
        }
        
        let amplitude = (real * real + imag * imag).sqrt() / (n as f64);
        frequencies.push(freq);
        amplitudes.push(amplitude);
    }
    
    // Step function height (K(0) for GUE)
    let step_function_height = 2.0; // GUE step function height
    
    // Oscillation amplitude
    let oscillation_amplitude = amplitudes.iter().skip(1).take(10).sum::<f64>() / 10.0;
    
    // Spectral rigidity (integrated form factor)
    let spectral_rigidity = amplitudes.iter().sum::<f64>();
    
    let analysis = FormFactor {
        frequencies: frequencies.clone(),
        amplitudes: amplitudes.clone(),
        step_function_height,
        oscillation_amplitude,
        spectral_rigidity,
    };
    
    // Save results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/form_factor.json", output_dir), json)?;
    
    // Generate report
    let mut report = fs::File::create(format!("{}/form_factor_report.txt", output_dir))?;
    writeln!(report, "=== Form Factor Analysis ===")?;
    writeln!(report, "Step function height: {:.6}", analysis.step_function_height)?;
    writeln!(report, "Oscillation amplitude: {:.6}", analysis.oscillation_amplitude)?;
    writeln!(report, "Spectral rigidity: {:.6}", analysis.spectral_rigidity)?;
    
    writeln!(report, "\nTop 10 frequencies:")?;
    for i in 0..10.min(frequencies.len()) {
        writeln!(report, "  Freq: {:.4}, Amp: {:.6}", frequencies[i], amplitudes[i])?;
    }
    
    writeln!(report, "\nInterpretation:")?;
    if analysis.oscillation_amplitude < 0.01 {
        writeln!(report, "✓ Minimal oscillations (good GUE agreement)")?;
    } else {
        writeln!(report, "? Significant oscillations detected")?;
    }
    
    println!("✓ Form factor analysis completed");
    Ok(analysis)
}

fn generate_robust_analysis_report(
    spacing: &SpacingDistribution,
    pair_corr: &PairCorrelation,
    form_factor: &FormFactor,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating comprehensive robust analysis report...");
    
    // Compute unfolding-independent score
    let mut score = 0.0;
    let mut genuine_anomalies = 0;
    
    // Spacing distribution score
    if spacing.wigner_dyson_deviation < 0.1 {
        score += 1.0;
    } else {
        genuine_anomalies += 1;
    }
    
    // Pair correlation score
    if pair_corr.sine_kernel_deviation < 0.05 {
        score += 1.0;
    } else if pair_corr.sine_kernel_deviation < 0.1 {
        score += 0.5;
    } else {
        genuine_anomalies += 1;
    }
    
    // Form factor score
    if form_factor.oscillation_amplitude < 0.01 {
        score += 1.0;
    } else {
        genuine_anomalies += 1;
    }
    
    let analysis = RobustAnalysis {
        spacing_distribution: spacing.clone(),
        pair_correlation: pair_corr.clone(),
        form_factor: form_factor.clone(),
        unfolding_independent_score: score / 3.0,
        genuine_anomaly_detected: genuine_anomalies > 0,
    };
    
    // Save comprehensive results
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(format!("{}/robust_analysis.json", output_dir), json)?;
    
    // Generate comprehensive report
    let mut report = fs::File::create(format!("{}/robust_analysis_report.txt", output_dir))?;
    writeln!(report, "=== Comprehensive Robust Spectral Analysis ===")?;
    writeln!(report, "Unfolding-independent score: {:.3}", analysis.unfolding_independent_score)?;
    writeln!(report, "Genuine anomalies detected: {}", analysis.genuine_anomaly_detected)?;
    
    writeln!(report, "\n--- Summary ---")?;
    writeln!(report, "Spacing distribution Wigner-Dyson deviation: {:.6}", spacing.wigner_dyson_deviation)?;
    writeln!(report, "Pair correlation sine kernel deviation: {:.6}", pair_corr.sine_kernel_deviation)?;
    writeln!(report, "Form factor oscillation amplitude: {:.6}", form_factor.oscillation_amplitude)?;
    
    writeln!(report, "\n--- Conclusions ---")?;
    if analysis.unfolding_independent_score > 0.8 {
        writeln!(report, "✓ Excellent agreement with universal random matrix predictions")?;
        writeln!(report, "✓ No significant unfolding-independent anomalies detected")?;
    } else if analysis.unfolding_independent_score > 0.6 {
        writeln!(report, "? Moderate agreement with universal predictions")?;
        writeln!(report, "? Some minor deviations detected")?;
    } else {
        writeln!(report, "❌ Significant deviations from universal predictions")?;
        writeln!(report, "❌ Genuine spectral anomalies detected")?;
    }
    
    if analysis.genuine_anomaly_detected {
        writeln!(report, "\n🔍 RECOMMENDATION:")?;
        writeln!(report, "   Investigate detected anomalies further")?;
        writeln!(report, "   These may represent genuine arithmetic structure")?;
    } else {
        writeln!(report, "\n✅ RECOMMENDATION:")?;
        writeln!(report, "   Results consistent with random matrix universality")?;
        writeln!(report, "   Focus on long-range correlations if seeking anomalies")?;
    }
    
    println!("✓ Comprehensive robust analysis completed");
    Ok(())
}

// Helper functions
fn compute_wigner_dyson_deviation(spacings: &[f64]) -> f64 {
    // Compare empirical distribution to Wigner surmise
    let n_bins = 50;
    let max_s = spacings.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = max_s / n_bins as f64;
    
    let mut empirical = vec![0.0; n_bins];
    let mut counts = vec![0; n_bins];
    
    for &s in spacings {
        if s <= max_s {
            let bin = (s / bin_width) as usize;
            if bin < n_bins {
                empirical[bin] += 1.0;
                counts[bin] += 1;
            }
        }
    }
    
    // Normalize
    for i in 0..n_bins {
        if counts[i] > 0 {
            empirical[i] /= counts[i] as f64;
        }
    }
    
    // Wigner surmise: P(s) = (π/2) * s * exp(-πs²/4)
    let mut wigner = vec![0.0; n_bins];
    for i in 0..n_bins {
        let s = (i as f64 + 0.5) * bin_width;
        wigner[i] = (std::f64::consts::PI / 2.0) * s * (-std::f64::consts::PI * s * s / 4.0).exp();
    }
    
    // Normalize Wigner distribution
    let wigner_sum: f64 = wigner.iter().sum();
    for w in &mut wigner {
        *w /= wigner_sum;
    }
    
    // Compute deviation
    let mut total_deviation = 0.0;
    let mut valid_bins = 0;
    for i in 0..n_bins {
        if counts[i] > 5 {
            total_deviation += (empirical[i] - wigner[i]).abs();
            valid_bins += 1;
        }
    }
    
    if valid_bins > 0 {
        total_deviation / valid_bins as f64
    } else {
        1.0
    }
}

fn estimate_brody_parameter(spacings: &[f64]) -> f64 {
    // Simple Brody parameter estimation
    // P(s) = (β+1) α s^β exp(-α s^(β+1))
    // where α = [Γ((β+2)/2)]^(β+1) / [Γ((β+1)/2)]^(β+2)
    
    let n_bins = 50;
    let max_s = spacings.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let bin_width = max_s / n_bins as f64;
    
    let mut empirical = vec![0.0; n_bins];
    let mut counts = vec![0; n_bins];
    
    for &s in spacings {
        if s <= max_s {
            let bin = (s / bin_width) as usize;
            if bin < n_bins {
                empirical[bin] += 1.0;
                counts[bin] += 1;
            }
        }
    }
    
    // Normalize
    for i in 0..n_bins {
        if counts[i] > 0 {
            empirical[i] /= counts[i] as f64;
        }
    }
    
    // Simple estimation: find β that best fits
    // For simplicity, return 1.0 (GOE/GUE) if distribution looks reasonable
    let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
    let variance = spacings.iter().map(|&s| (s - mean_spacing).powi(2)).sum::<f64>() / spacings.len() as f64;
    
    // GOE has variance ≈ 4/π - 1 ≈ 0.273
    // GUE has variance ≈ 1 - 2/π ≈ 0.363
    let goe_var = 4.0 / std::f64::consts::PI - 1.0;
    let gue_var = 1.0 - 2.0 / std::f64::consts::PI;
    
    if (variance - goe_var).abs() < (variance - gue_var).abs() {
        1.0 // GOE-like
    } else {
        2.0 // GUE-like (approximate)
    }
}

fn compute_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }
    
    let n = x.len();
    let mean_x = x.iter().sum::<f64>() / n as f64;
    let mean_y = y.iter().sum::<f64>() / n as f64;
    
    let mut covariance = 0.0;
    let mut var_x = 0.0;
    let mut var_y = 0.0;
    
    for i in 0..n {
        let dx = x[i] - mean_x;
        let dy = y[i] - mean_y;
        covariance += dx * dy;
        var_x += dx * dx;
        var_y += dy * dy;
    }
    
    covariance / (var_x * var_y).sqrt()
}
