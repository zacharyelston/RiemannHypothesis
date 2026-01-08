/// SQLite Database for Validation Results
/// 
/// Stores all experimental results from multi-GPU validation
/// Enables comprehensive analysis and historical tracking

use std::path::Path;
use sqlite::{Connection, Result as SqliteResult};
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ExperimentRecord {
    pub id: Option<i64>,
    pub experiment_id: String,
    pub timestamp: DateTime<Utc>,
    pub mode: String, // "dense", "gue_control", "both"
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
    pub gpu_power_1_w: f64,
    pub gpu_memory_0_mb: f64,
    pub gpu_memory_1_mb: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LValueResult {
    pub id: Option<i64>,
    pub experiment_id: String,
    pub l_value: f64,
    pub sigma2: f64,
    pub delta3: f64,
    pub gpu_id: usize, // Which GPU computed this
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GueInstanceResult {
    pub id: Option<i64>,
    pub experiment_id: String,
    pub instance_id: usize,
    pub gue_size: usize,
    pub gpu_id: usize,
    pub computation_time_ms: u64,
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
                gpu_power_1_w REAL NOT NULL,
                gpu_memory_0_mb REAL NOT NULL,
                gpu_memory_1_mb REAL NOT NULL
            )"
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS l_value_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                experiment_id TEXT NOT NULL,
                l_value REAL NOT NULL,
                sigma2 REAL NOT NULL,
                delta3 REAL NOT NULL,
                gpu_id INTEGER NOT NULL,
                FOREIGN KEY (experiment_id) REFERENCES experiments (experiment_id)
            )"
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS gue_instance_results (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                experiment_id TEXT NOT NULL,
                instance_id INTEGER NOT NULL,
                gue_size INTEGER NOT NULL,
                gpu_id INTEGER NOT NULL,
                computation_time_ms INTEGER NOT NULL,
                FOREIGN KEY (experiment_id) REFERENCES experiments (experiment_id)
            )"
        )?;
        
        // Create indexes for performance
        conn.execute("CREATE INDEX IF NOT EXISTS idx_experiments_timestamp ON experiments (timestamp)")?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_l_value_experiment ON l_value_results (experiment_id)")?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_l_value_l ON l_value_results (l_value)")?;
        conn.execute("CREATE INDEX IF NOT EXISTS idx_gue_experiment ON gue_instance_results (experiment_id)")?;
        
        Ok(Self { conn })
    }
    
    pub fn store_experiment(&self, record: &ExperimentRecord) -> SqliteResult<()> {
        let timestamp_str = record.timestamp.to_rfc3339();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO experiments (
                experiment_id, timestamp, mode, gpu_count, zeros_count,
                l_min, l_max, l_step, l_points, gue_size, gue_instances,
                gpu_0_work, gpu_1_work, sigma2_l3, delta3_l3, computation_time_ms,
                gpu_power_0_w, gpu_power_1_w, gpu_memory_0_mb, gpu_memory_1_mb
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20)",
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
                record.gpu_power_1_w,
                record.gpu_memory_0_mb,
                record.gpu_memory_1_mb,
            ),
        )?;
        
        Ok(())
    }
    
    pub fn store_l_value_results(&self, experiment_id: &str, results: &[LValueResult]) -> SqliteResult<()> {
        let transaction = self.conn.transaction()?;
        
        for result in results {
            transaction.execute(
                "INSERT INTO l_value_results (
                    experiment_id, l_value, sigma2, delta3, gpu_id
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    experiment_id,
                    result.l_value,
                    result.sigma2,
                    result.delta3,
                    result.gpu_id as i64,
                ),
            )?;
        }
        
        transaction.commit()?;
        Ok(())
    }
    
    pub fn store_gue_results(&self, experiment_id: &str, results: &[GueInstanceResult]) -> SqliteResult<()> {
        let transaction = self.conn.transaction()?;
        
        for result in results {
            transaction.execute(
                "INSERT INTO gue_instance_results (
                    experiment_id, instance_id, gue_size, gpu_id, computation_time_ms
                ) VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    experiment_id,
                    result.instance_id as i64,
                    result.gue_size as i64,
                    result.gpu_id as i64,
                    result.computation_time_ms as i64,
                ),
            )?;
        }
        
        transaction.commit()?;
        Ok(())
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
                gpu_power_1_w: row.get(18)?,
                gpu_memory_0_mb: row.get(19)?,
                gpu_memory_1_mb: row.get(20)?,
            })
        })?;
        
        let mut experiments = Vec::new();
        for record in records {
            experiments.push(record?);
        }
        
        Ok(experiments)
    }
    
    pub fn get_l_values_for_experiment(&self, experiment_id: &str) -> SqliteResult<Vec<LValueResult>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM l_value_results WHERE experiment_id = ? ORDER BY l_value"
        )?;
        
        let results = stmt.query_map([experiment_id], |row| {
            Ok(LValueResult {
                id: Some(row.get(0)?),
                experiment_id: row.get(1)?,
                l_value: row.get(2)?,
                sigma2: row.get(3)?,
                delta3: row.get(4)?,
                gpu_id: row.get::<_, i64>(5)? as usize,
            })
        })?;
        
        let mut l_values = Vec::new();
        for result in results {
            l_values.push(result?);
        }
        
        Ok(l_values)
    }
    
    pub fn export_to_csv(&self, experiment_id: &str, output_path: &str) -> SqliteResult<()> {
        let l_values = self.get_l_values_for_experiment(experiment_id)?;
        
        use std::fs::File;
        use std::io::Write;
        
        let mut file = File::create(output_path)?;
        writeln!(file, "l_value,sigma2,delta3,gpu_id")?;
        
        for result in &l_values {
            writeln!(file, "{},{},{},{}", 
                result.l_value, 
                result.sigma2, 
                result.delta3,
                result.gpu_id
            )?;
        }
        
        Ok(())
    }
    
    pub fn get_crossover_analysis(&self) -> SqliteResult<Vec<(String, f64, f64, f64)>> {
        let mut stmt = self.conn.prepare(
            "SELECT experiment_id, l_min, l_max, l_step 
             FROM experiments 
             WHERE mode = 'both' 
             ORDER BY timestamp DESC"
        )?;
        
        let results = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, f64>(1)?,
                row.get::<_, f64>(2)?,
                row.get::<_, f64>(3)?,
            ))
        })?;
        
        let mut analysis = Vec::new();
        for result in results {
            analysis.push(result?);
        }
        
        Ok(analysis)
    }
}
