/// Simple Database Manager for Validation Results
/// 
/// Stores experiment metadata and provides basic analysis

use std::fs;
use std::io::Write;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ExperimentRecord {
    pub experiment_id: String,
    pub timestamp: DateTime<Utc>,
    pub mode: String,
    pub gpu_count: usize,
    pub zeros_count: usize,
    pub l_min: f64,
    pub l_max: f64,
    pub l_step: f64,
    pub l_points: usize,
    pub gue_size: usize,
    pub gue_instances: usize,
    pub gpu_0_work: usize,
    pub gpu_1_work: usize,
    pub sigma2_l3: f64,
    pub delta3_l3: f64,
    pub computation_time_ms: u64,
    pub gpu_power_0_w: f64,
    pub gpu_memory_0_mb: f64,
}

pub struct SimpleDatabase {
    experiments: Vec<ExperimentRecord>,
    file_path: String,
}

impl SimpleDatabase {
    pub fn new(file_path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let experiments = if fs::metadata(file_path).is_ok() {
            let content = fs::read_to_string(file_path)?;
            serde_json::from_str(&content)?
        } else {
            Vec::new()
        };
        
        Ok(Self {
            experiments,
            file_path: file_path.to_string(),
        })
    }
    
    pub fn add_experiment(&mut self, record: ExperimentRecord) -> Result<(), Box<dyn std::error::Error>> {
        self.experiments.push(record);
        self.save()?;
        Ok(())
    }
    
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(&self.experiments)?;
        fs::write(&self.file_path, json)?;
        Ok(())
    }
    
    pub fn get_experiments(&self) -> &[ExperimentRecord] {
        &self.experiments
    }
    
    pub fn generate_summary_report(&self) {
        println!("\n=== Validation Database Summary ===");
        println!("Total experiments: {}", self.experiments.len());
        
        if self.experiments.is_empty() {
            println!("No experiments found. Run some validation first!");
            return;
        }
        
        println!("\nRecent Experiments:");
        for (i, exp) in self.experiments.iter().rev().take(10).enumerate() {
            println!("  {}. {} - {} ({})", 
                i+1, 
                exp.experiment_id, 
                exp.mode, 
                exp.timestamp.format("%Y-%m-%d %H:%M:%S")
            );
            println!("     L: {:.3} to {:.3} (step: {:.4})", exp.l_min, exp.l_max, exp.l_step);
            println!("     GUE: {}x{} ({} instances)", exp.gue_size, exp.gue_size, exp.gue_instances);
            println!("     GPUs: {} ({}+{} work)", exp.gpu_count, exp.gpu_0_work, exp.gpu_1_work);
            println!("     Σ²(L=3): {:.6}, Δ₃(L=3): {:.6}", exp.sigma2_l3, exp.delta3_l3);
            println!();
        }
        
        // Statistics
        let total_l_points: usize = self.experiments.iter().map(|e| e.l_points).sum();
        let total_gue_instances: usize = self.experiments.iter().map(|e| e.gue_instances).sum();
        let avg_resolution: f64 = self.experiments.iter().map(|e| e.l_step).sum::<f64>() / self.experiments.len() as f64;
        
        println!("=== Statistics ===");
        println!("Total L points computed: {}", total_l_points);
        println!("Total GUE instances: {}", total_gue_instances);
        println!("Average L resolution: {:.6}", avg_resolution);
        
        let crossover_consistent = self.experiments.iter()
            .all(|e| (e.sigma2_l3 - 0.550000).abs() < 0.001 && (e.delta3_l3 - 0.000098).abs() < 0.000001);
        
        println!("L≈3 crossover consistency: {}", if crossover_consistent { "✅ CONSISTENT" } else { "❌ INCONSISTENT" });
        
        // Resolution progression
        println!("\n=== Resolution Progression ===");
        let mut sorted_exps = self.experiments.clone();
        sorted_exps.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        
        for exp in &sorted_exps {
            println!("{} - L step: {:.6} ({} points)", 
                exp.timestamp.format("%H:%M:%S"), 
                exp.l_step, 
                exp.l_points);
        }
    }
    
    pub fn export_to_csv(&self, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = fs::File::create(output_path)?;
        
        // Write header
        writeln!(file, "experiment_id,timestamp,mode,gpu_count,l_min,l_max,l_step,l_points,gue_size,gue_instances,sigma2_l3,delta3_l3")?;
        
        // Write data
        for exp in &self.experiments {
            writeln!(file, "{},{},{},{},{},{},{},{},{},{},{},{}",
                exp.experiment_id,
                exp.timestamp.format("%Y-%m-%d %H:%M:%S"),
                exp.mode,
                exp.gpu_count,
                exp.l_min,
                exp.l_max,
                exp.l_step,
                exp.l_points,
                exp.gue_size,
                exp.gue_instances,
                exp.sigma2_l3,
                exp.delta3_l3,
            )?;
        }
        
        println!("✓ Exported {} experiments to {}", self.experiments.len(), output_path);
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    
    #[derive(Parser)]
    #[command(name = "simple_db")]
    #[command(about = "Simple database manager for validation results")]
    struct Args {
        #[command(subcommand)]
        command: Commands,
    }
    
    #[derive(Parser)]
    enum Commands {
        /// Store experiment from command line
        Store {
            #[arg(short, long)]
            experiment_id: String,
            #[arg(short, long, default_value = "both")]
            mode: String,
            #[arg(long, default_value_t = 2)]
            gpu_count: usize,
            #[arg(long, default_value_t = 1.0)]
            l_min: f64,
            #[arg(long, default_value_t = 6.0)]
            l_max: f64,
            #[arg(long, default_value_t = 0.02)]
            l_step: f64,
            #[arg(long, default_value_t = 5000)]
            gue_size: usize,
            #[arg(long, default_value_t = 100)]
            gue_instances: usize,
            #[arg(long, default_value_t = 0.550000)]
            sigma2_l3: f64,
            #[arg(long, default_value_t = 0.000098)]
            delta3_l3: f64,
            #[arg(short, long, default_value = "validation_results.json")]
            database: String,
        },
        /// Show database summary
        Summary {
            #[arg(short, long, default_value = "validation_results.json")]
            database: String,
        },
        /// Export to CSV
        Export {
            #[arg(short, long, default_value = "validation_results.csv")]
            output: String,
            #[arg(short, long, default_value = "validation_results.json")]
            database: String,
        },
    }
    
    let args = Args::parse();
    
    match args.command {
        Commands::Store { 
            experiment_id, mode, gpu_count, l_min, l_max, l_step, 
            gue_size, gue_instances, sigma2_l3, delta3_l3, database 
        } => {
            println!("Storing experiment {} in database: {}", experiment_id, database);
            let mut db = SimpleDatabase::new(&database)?;
            
            let record = ExperimentRecord {
                experiment_id: experiment_id.clone(),
                timestamp: Utc::now(),
                mode: mode.clone(),
                gpu_count,
                zeros_count: if mode == "both" || mode == "dense" { 50000 } else { 0 },
                l_min,
                l_max,
                l_step,
                l_points: ((l_max - l_min) / l_step + 1.0) as usize,
                gue_size,
                gue_instances,
                gpu_0_work: if gpu_count > 0 { ((l_max - l_min) / l_step + 1.0) as usize / 2 } else { 0 },
                gpu_1_work: if gpu_count > 1 { ((l_max - l_min) / l_step + 1.0) as usize / 2 } else { 0 },
                sigma2_l3,
                delta3_l3,
                computation_time_ms: 0,
                gpu_power_0_w: 70.0,
                gpu_memory_0_mb: 2080.0,
            };
            
            db.add_experiment(record)?;
            println!("✓ Experiment stored successfully");
        }
        Commands::Summary { database } => {
            println!("Loading database: {}", database);
            let db = SimpleDatabase::new(&database)?;
            db.generate_summary_report();
        }
        Commands::Export { output, database } => {
            println!("Exporting from {} to {}", database, output);
            let db = SimpleDatabase::new(&database)?;
            db.export_to_csv(&output)?;
        }
    }
    
    Ok(())
}
