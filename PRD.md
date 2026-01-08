# Product Requirements Document (PRD): Riemann Hypothesis Scientific Solver

## 1. Project Overview
The **Riemann Scientific Solver** transforms the Riemann Hypothesis from a pure mathematical conjecture into a verifiable spectral geometry problem. The goal is to develop a computational platform that simulates the **Berry-Keating Hamiltonian** ($H=xp$) and verifies its connection to the Riemann Zeros via the Gutzwiller Trace Formula.

## 2. Core Objectives
1.  **Strict Rust Implementation**: All simulation and analysis code must be written in Rust. No legacy Python/scripting.
2.  **Container-First Architecture**: Every component must run deterministically inside Docker containers.
3.  **Reproducible Science**: The system must output statistically verifiable results (GUE statistics, spectral variance).
4.  **Scientific "Process" Adherence**: Code structure must reflect the scientific method (Observation -> Hypothesis -> Experiment -> Verification).

## 3. Technical Requirements

### 3.1. Infrastructure
*   **Language**: Rust (Latest Stable)
*   **Build System**: Docker with `cargo-chef` for layer caching.
*   **Dependencies**:
    *   `nalgebra`: Linear algebra operations.
    *   `rand` / `rand_distr`: Statistical generation.
    *   `statrs`: Statistical analysis.
*   **Output**: JSON-structured analysis results and terminal-based visualization (ASCII charts or similar).

### 3.2. Simulation Modules
*   **Module A: GUE Verifier (Baseline)**:
    *   Generate large Random Hermitian Matrices.
    *   Compute eigenvalue spacings.
    *   Verify match with Wigner Surmise.
*   **Module B: Berry-Keating Hamiltonian (Planned)**:
    *   Simulate the $H=xp$ operator on a truncated Hilbert space.
    *   Implement boundary condition parameters.
    *   Search for spectral matches with known Zeta zeros.

## 4. Development Standards
*   **No Python**: Remove all legacy Python scripts (`verify_gue.py`, etc.).
*   **Strict Typing**: Use Rust's type system to enforce physical unit consistency where applicable.
*   **Testing**: Unit tests for all mathematical functions (e.g., matrix generation, spacing normalization).
*   **Documentation**: All public API functions must have doc comments explaining the mathematical/physical significance.

## 5. Success Metrics
*   **GUE Variance Match**: Simulation variance must be within $5\%$ of the theoretical value ($Var \approx 0.178$).
*   **Performance**: Matrix generation and diagonalization for $N=1000$ should take $<5$ seconds on standard hardware.
*   **Clean Build**: `docker build` must pass with zero warnings and leverage cache layers.

## 6. Future Roadmap
*   Integrate Guth-Maynard density bounds as a filter for search space.
*   Implement Connes' Adèle space trace formula.
