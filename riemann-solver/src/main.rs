use nalgebra::DMatrix;
use rand::thread_rng;
use rand_distr::{Normal, Distribution};
use std::f64::consts::PI;

fn main() {
    let n = 300; // Reduced size for faster execution
    println!("Scientific Process Step 4: Empirical Verification");
    println!("Generating Random Hermitian Matrix (GUE) of size {}x{}...", n, n);

    let eigenvalues = generate_gue_eigenvalues(n);
    let spacings = compute_normalized_spacings(&eigenvalues);
    
    analyze_spacings(&spacings);
}

fn generate_gue_eigenvalues(n: usize) -> Vec<f64> {
    let mut rng = thread_rng();
    let normal = Normal::new(0.0, 1.0 / (2.0 * n as f64).sqrt()).unwrap();

    let mut re_parts = DMatrix::from_fn(n, n, |_, _| normal.sample(&mut rng));
    let mut im_parts = DMatrix::from_fn(n, n, |_, _| normal.sample(&mut rng));

    for i in 0..n {
        for j in 0..i {
            let re = (re_parts[(i, j)] + re_parts[(j, i)]) / 2.0_f64.sqrt();
            let im = (im_parts[(i, j)] - im_parts[(j, i)]) / 2.0_f64.sqrt();
            
            re_parts[(i, j)] = re;
            re_parts[(j, i)] = re;
            im_parts[(i, j)] = im;
            im_parts[(j, i)] = -im;
        }
        re_parts[(i, i)] = normal.sample(&mut rng);
        im_parts[(i, i)] = 0.0;
    }

    let mut big_mat = DMatrix::zeros(2 * n, 2 * n);
    for i in 0..n {
        for j in 0..n {
            big_mat[(i, j)] = re_parts[(i, j)];
            big_mat[(i + n, j + n)] = re_parts[(i, j)];
            big_mat[(i + n, j)] = im_parts[(i, j)];
            big_mat[(i, j + n)] = -im_parts[(i, j)];
        }
    }

    println!("Computing eigenvalues...");
    let eig = big_mat.symmetric_eigen();
    let mut evs = eig.eigenvalues.as_slice().to_vec();
    
    evs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let distinct_evs: Vec<f64> = evs.iter().step_by(2).cloned().collect();
    
    distinct_evs
}

fn compute_normalized_spacings(eigenvalues: &[f64]) -> Vec<f64> {
    let center_evs: Vec<f64> = eigenvalues.iter()
        .filter(|&&x| x.abs() < 1.0)
        .cloned()
        .collect();
        
    let mut spacings = Vec::new();
    for i in 0..center_evs.len()-1 {
        let diff = center_evs[i+1] - center_evs[i];
        spacings.push(diff);
    }
    
    if spacings.is_empty() { return vec![]; }
    
    let mean_spacing = spacings.iter().sum::<f64>() / spacings.len() as f64;
    spacings.iter().map(|s| s / mean_spacing).collect()
}

fn analyze_spacings(spacings: &[f64]) {
    if spacings.is_empty() {
        println!("Not enough data.");
        return;
    }
    let mean = spacings.iter().sum::<f64>() / spacings.len() as f64;
    let variance = spacings.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / spacings.len() as f64;
    
    println!("Analysis Results:");
    println!("  Number of spacings: {}", spacings.len());
    println!("  Mean spacing: {:.4} (Theory: 1.0)", mean);
    println!("  Variance: {:.4} (Theory GUE: ~0.178, Poisson: 1.0)", variance);
    
    if (variance - 0.178).abs() < 0.15 {
        println!("  CONCLUSION: Matches GUE statistics (Quantum Chaos / Riemann Zeros)");
    } else {
        println!("  CONCLUSION: Deviates from GUE (Variance={:.4})", variance);
    }
}
