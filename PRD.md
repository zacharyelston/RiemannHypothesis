# Product Requirements Document (PRD): Riemann Hypothesis Scientific Solver

## 1. Project Overview
The **Riemann Scientific Solver** transforms the Riemann Hypothesis from a pure mathematical conjecture into a verifiable spectral geometry problem. The goal is to develop a computational platform that simulates the **Berry-Keating Hamiltonian** ($H=xp$) and verifies its connection to the Riemann Zeros via the Gutzwiller Trace Formula.

## 2. Core Objectives
1.  **Strict Rust Implementation**: All simulation and analysis code must be written in Rust. No legacy Python/scripting.
2.  **Container-First Architecture**: Every component must run deterministically inside Docker containers.
3.  **Reproducible Science**: The system must output statistically verifiable results (GUE statistics, spectral variance).
4.  **Scientific "Process" Adherence**: Code structure must reflect the scientific method (Observation -> Hypothesis -> Experiment -> Verification).

## 3. System Architecture

### 3.1 Crate Structure
The project is organized as a single binary crate `riemann-solver` with the following modular structure. DevBots must create these modules if they do not exist.

```rust
src/
├── main.rs           # CLI Entrypoint & Orchestration
├── config.rs         # Configuration structs (Serde)
├── hamiltonian/      # Physics models & Systems
│   ├── mod.rs        # Trait definitions (QuantumSystem)
│   ├── gue.rs        # Random Matrix (GUE) implementation (Baseline)
│   └── berry_keating.rs # H=xp implementation (Target)
├── solver/           # Eigenvalue solvers
│   ├── mod.rs
│   └── lapack.rs     # Integration with nalgebra/lapack
├── analysis/         # Statistical verification
│   ├── mod.rs
│   ├── spacing.rs    # Nearest-neighbor spacing statistics
│   └── spectral.rs   # Spectral rigidity / number variance
└── utils/            # Math helpers & Error types
```

### 3.2 Core Traits (Contract)
DevBots must strictly adhere to these trait definitions to ensure modularity.

```rust
/// Represents a physical or mathematical system that generates a spectrum.
pub trait QuantumSystem {
    /// Generates the Hamiltonian matrix representation.
    /// Returns `Result` to handle dimension/memory errors.
    fn generate_hamiltonian(&self) -> Result<DMatrix<Complex<f64>>>;
    
    /// Returns the system size/dimension.
    fn size(&self) -> usize;
}

/// Analyzer trait for verifying hypotheses against the spectrum.
pub trait SpectrumAnalyzer {
    /// Normalizes raw eigenvalues (unfolding) based on local density of states.
    fn unfold_spectrum(&self, raw_eigenvalues: &[f64]) -> Vec<f64>;
    
    /// Computes statistical properties of the unfolded spectrum.
    fn analyze(&self, unfolded_spectrum: &[f64]) -> SpectralStats;
}
```

### 3.3 Error Handling & Logging
*   **Error Strategy**: Use `thiserror` for library errors and `anyhow` for application-level error handling.
    *   Define `RiemannError` enum in `utils/error.rs`.
*   **Logging**: Use `tracing` and `tracing-subscriber` for structured logging.
    *   Log all simulation parameters at startup.
    *   Log progress of matrix generation and diagonalization.

### 3.4 Core Data Structures
DevBots must implement the following structs to ensure type safety and consistency.

```rust
/// Configuration for a simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub matrix_size: usize,
    pub iterations: usize,
    pub seed: Option<u64>,
}

/// Results from a spectral analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralStats {
    pub mean_spacing: f64,
    pub variance: f64,
    pub skewness: f64,
    pub kurtosis: f64,
    pub gue_match_confidence: f64, // 0.0 to 1.0
}

/// A specific error type for the solver.
#[derive(thiserror::Error, Debug)]
pub enum RiemannError {
    #[error("Invalid matrix size: {0}")]
    InvalidSize(usize),
    #[error("LAPACK diagonalization failed: {0}")]
    DiagonalizationError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}
```

## 4. Functional Requirements

### 4.1. Infrastructure
*   **Language**: Rust (Latest Stable).
*   **Container**: Docker with `cargo-chef` (Optimized build caching).
*   **Required Crates**:
    ```toml
    [dependencies]
    nalgebra = "0.32"        # Linear Algebra
    rand = "0.8"             # RNG
    rand_distr = "0.4"       # Distributions (Normal, etc.)
    statrs = "0.16"          # Stats helpers
    clap = { version = "4.4", features = ["derive"] } # CLI
    serde = { version = "1.0", features = ["derive"] } # Config
    thiserror = "1.0"        # Error defining
    anyhow = "1.0"           # Error handling
    tracing = "0.1"          # Logging
    tracing-subscriber = "0.3"
    ```

### 4.2. Simulation Modules
*   **Module A: GUE Verifier (Baseline)**
    *   **Input**: Matrix dimension $N$ (e.g., 1000).
    *   **Logic**: 
        1. Construct Hermitian matrix $H$ where $H_{ij} \sim \mathcal{N}(0, 1) + i\mathcal{N}(0, 1)$.
        2. Diagonalize to get eigenvalues $E_n$.
        3. Unfold eigenvalues to mean spacing $\Delta E = 1$.
        4. Calculate spacing variance.
    *   **Success**: Variance $\approx 0.178$.
*   **Module B: Berry-Keating Hamiltonian (Target)**
    *   **Input**: Phase space volume $\hbar_{\text{eff}}$.
    *   **Logic**: Discretize $H = \frac{1}{2}(xp + px)$ on a basis.
    *   **Success**: Spectrum matches Riemann zeros.

### 4.3 CLI Interface
The application must use `clap` to support subcommands:

```bash
# Run baseline GUE verification
riemann-solver verify-gue --size 1000 --iterations 100

# Run Berry-Keating simulation
riemann-solver simulate-bk --cutoff 50.0
```

## 5. Development Standards
*   **Type Safety**: Use Newtypes (e.g., `struct Energy(f64)`) to prevent unit errors.
*   **Testing**: 
    *   `cargo test` must pass.
    *   Include property-based tests (quickcheck/proptest) for matrix properties (Hermiticity).
*   **Documentation**: `cargo doc` must generate clean docs for all modules.

## 6. Detailed Task Breakdown (DevBot Backlog)

### Phase 1: Skeleton & Infrastructure
- [ ] **Task 1.1**: Update `Cargo.toml` with all dependencies listed in Section 4.1.
- [ ] **Task 1.2**: Create the directory structure from Section 3.1.
- [ ] **Task 1.3**: Implement `config.rs` and `main.rs` with `clap` subcommand parsing.

### Phase 2: Core Traits & GUE Implementation
- [ ] **Task 2.1**: Define `QuantumSystem` and `SpectrumAnalyzer` traits in their respective `mod.rs` files.
- [ ] **Task 2.2**: Implement `GueSystem` in `hamiltonian/gue.rs`. Ensure efficient matrix generation.
- [ ] **Task 2.3**: Implement `LapackSolver` (or nalgebra equivalent) in `solver/lapack.rs`.

### Phase 3: Analysis & Verification
- [ ] **Task 3.1**: Implement `unfold_spectrum` logic in `analysis/spacing.rs`.
- [ ] **Task 3.2**: Implement statistical analysis (mean, variance) in `analysis/spectral.rs`.
- [ ] **Task 3.3**: Wire everything into the `verify-gue` CLI command.

### Phase 4: Containerization & CI
- [ ] **Task 4.1**: Verify `Dockerfile` builds successfully with new dependencies.
- [ ] **Task 4.2**: Create a GitHub Actions workflow (or local script) to run tests and verification.

## 7. Success Metrics
*   **GUE Variance Match**: Simulation variance must be within $5\%$ of the theoretical value ($Var \approx 0.178$).
*   **Performance**: Matrix generation and diagonalization for $N=1000$ should take $<5$ seconds.
*   **Code Quality**: `cargo clippy` returns no warnings.
