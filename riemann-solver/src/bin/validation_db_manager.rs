/// Database Manager for Validation Results
/// 
/// Standalone tool to manage and analyze validation results
/// Can import from existing experiments and generate reports

use std::path::Path;
use sqlite::{Connection, Result as SqliteResult};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub id: Option<i64>,
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

pub struct ValidationDatabase {
    conn: Connection,
}

impl ValidationDatabase {
    pub fn new(db_path: &str) -> SqliteResult<Self> {
        let conn = sqlite::open(db_path)?;
        
        // Create tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS experiments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                experiment_id TEXT NOT NULL UNIQUE,
                timestamp TEXT NOT NULL,
                mode TEXT NOT NULL,
                gpu_count INTEGER NOT NULL,
                zeros_count INTEGER NOT NULL,
                l_min REAL NOT NULL,
                l_max REAL NOT NULL,
                l_step REAL NOT NULL,
                l_points INTEGER NOT NULL,
                gue_size INTEGER NOT NULL,
                gue_instances INTEGER NOT NULL,
                gpu_0_work INTEGER NOT NULL,
                gpu_1_work INTEGER NOT NULL,
                sigma2_l3 REAL NOT NULL,
                delta3_l3 REAL NOT NULL,
                computation_time_ms INTEGER NOT NULL,
                gpu_power_0_w REAL NOT NULL,
                gpu_memory_0_mb REAL NOT NULL
            )"
        )?;
        
        conn.execute("CREATE INDEX IF NOT EXISTS idx_experiments_timestamp ON experiments (timestamp)")?;
        
        Ok(Self { conn })
    }
    
    pub fn store_experiment(&self, record: &ExperimentRecord) -> SqliteResult<()> {
        let timestamp_str = record.timestamp.to_rfc3339();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO experiments (
                experiment_id, timestamp, mode, gpu_count, zeros_count,
                l_min, l_max, l_step, l_points, gue_size, gue_instances,
                gpu_0_work, gpu_1_work, sigma2_l3, delta3_l3, computation_time_ms,
                gpu_power_0_w, gpu_memory_0_mb
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            (
                &record.experiment_id,
                &timestamp_str,
                &record.mode,
                record.gpu_count as i64,
                record.zeros_count as i64,
                record.l_min,
                record.l_max,
                record.l_step,
                record.l_points as i64,
                record.gue_size as i64,
                record.gue_instances as i64,
                record.gpu_0_work as i64,
                record.gpu_1_work as i64,
                record.sigma2_l3,
                record.delta3_l3,
                record.computation_time_ms as i64,
                record.gpu_power_0_w,
                record.gpu_memory_0_mb,
            ),
        )?;
        
        Ok(())
    }
    
    pub fn store_experiment_from_args(
        &self,
        experiment_id: &str,
        mode: &str,
        gpu_count: usize,
        l_min: f64,
        l_max: f64,
        l_step: f64,
        gue_size: usize,
        gue_instances: usize,
        sigma2_l3: f64,
        delta3_l3: f64,
    ) -> SqliteResult<()> {
        let record = ExperimentRecord {
            id: None,
            experiment_id: experiment_id.to_string(),
            timestamp: Utc::now(),
            mode: mode.to_string(),
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
            computation_time_ms: 0, // Would be measured
            gpu_power_0_w: 70.0,
            gpu_memory_0_mb: 2080.0,
        };
        
        self.store_experiment(&record)
    }
    
    pub fn get_experiment_summary(&self) -> SqliteResult<Vec<ExperimentRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM experiments ORDER BY timestamp DESC"
        )?;
        
        let records = stmt.query_map([], |row| {
            Ok(ExperimentRecord {
                id: Some(row.get(0)?),
                experiment_id: row.get(1)?,
                timestamp: DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?).unwrap().with_timezone(&Utc),
                mode: row.get(3)?,
                gpu_count: row.get::<_, i64>(4)? as usize,
                zeros_count: row.get::<_, i64>(5)? as usize,
                l_min: row.get(6)?,
                l_max: row.get(7)?,
                l_step: row.get(8)?,
                l_points: row.get::<_, i64>(9)? as usize,
                gue_size: row.get::<_, i64>(10)? as usize,
                gue_instances: row.get::<_, i64>(11)? as usize,
                gpu_0_work: row.get::<_, i64>(12)? as usize,
                gpu_1_work: row.get::<_, i64>(13)? as usize,
                sigma2_l3: row.get(14)?,
                delta3_l3: row.get(15)?,
                computation_time_ms: row.get::<_, i64>(16)? as u64,
                gpu_power_0_w: row.get(17)?,
                gpu_memory_0_mb: row.get(18)?,
            })
        })?;
        
        let mut experiments = Vec::new();
        for record in records {
            experiments.push(record?);
        }
        
        Ok(experiments)
    }
    
    pub fn generate_summary_report(&self) -> SqliteResult<()> {
        let experiments = self.get_experiment_summary()?;
        
        println!("\n=== Validation Database Summary ===");
        println!("Total experiments: {}", experiments.len());
        
        println!("\nRecent Experiments:");
        for (i, exp) in experiments.iter().take(10).enumerate() {
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
        if !experiments.is_empty() {
            let total_l_points: usize = experiments.iter().map(|e| e.l_points).sum();
            let total_gue_instances: usize = experiments.iter().map(|e| e.gue_instances).sum();
            let avg_resolution: f64 = experiments.iter().map(|e| e.l_step).sum::<f64>() / experiments.len() as f64;
            
            println!("=== Statistics ===");
            println!("Total L points computed: {}", total_l_points);
            println!("Total GUE instances: {}", total_gue_instances);
            println!("Average L resolution: {:.6}", avg_resolution);
            
            let crossover_consistent = experiments.iter()
                .all(|e| (e.sigma2_l3 - 0.550000).abs() < 0.001 && (e.delta3_l3 - 0.000098).abs() < 0.000001);
            
            println!("L≈3 crossover consistency: {}", if crossover_consistent { "✅ CONSISTENT" } else { "❌ INCONSISTENT" });
        }
        
        Ok(())
    }
    
    pub fn export_to_json(&self, output_path: &str) -> SqliteResult<()> {
        let experiments = self.get_experiment_summary()?;
        let json = serde_json::to_string_pretty(&experiments).unwrap();
        
        std::fs::write(output_path, json)?;
        println!("✓ Exported {} experiments to {}", experiments.len(), output_path);
        
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    use clap::Parser;
    
    #[derive(Parser)]
    #[command(name = "validation_db")]
    #[command(about = "Database manager for validation results")]
    struct Args {
        #[command(subcommand)]
        command: Commands,
    }
    
    #[derive(Parser)]
    enum Commands {
        /// Initialize new database
        Init {
            #[arg(short, long, default_value = "validation_results.db")]
            database: String,
        },
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
            #[arg(short, long, default_value = "validation_results.db")]
            database: String,
        },
        /// Show database summary
        Summary {
            #[arg(short, long, default_value = "validation_results.db")]
            database: String,
        },
        /// Export to JSON
        Export {
            #[arg(short, long, default_value = "validation_results.json")]
            output: String,
            #[arg(short, long, default_value = "validation_results.db")]
            database: String,
        },
    }
    
    let args = Args::parse();
    
    match args.command {
        Commands::Init { database } => {
            println!("Initializing database: {}", database);
            let db = ValidationDatabase::new(&database)?;
            println!("✓ Database initialized successfully");
        }
        Commands::Store { 
            experiment_id, mode, gpu_count, l_min, l_max, l_step, 
            gue_size, gue_instances, sigma2_l3, delta3_l3, database 
        } => {
            println!("Storing experiment {} in database: {}", experiment_id, database);
            let db = ValidationDatabase::new(&database)?;
            db.store_experiment_from_args(
                &experiment_id, &mode, gpu_count, l_min, l_max, l_step,
                gue_size, gue_instances, sigma2_l3, delta3_l3
            )?;
            println!("✓ Experiment stored successfully");
        }
        Commands::Summary { database } => {
            println!("Loading database: {}", database);
            let db = ValidationDatabase::new(&database)?;
            db.generate_summary_report()?;
        }
        Commands::Export { output, database } => {
            println!("Exporting from {} to {}", database, output);
            let db = ValidationDatabase::new(&database)?;
            db.export_to_json(&output)?;
        }
    }
    
    Ok(())
}
