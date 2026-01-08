# Faked or Simulated Components

This list captures spots in the current repo where the implementation explicitly calls out
placeholder, simulated, or mocked behavior rather than real computation.

## Rust code

- `riemann-solver/src/bin/cuda_validation.rs`: multi-GPU GUE control path prints per-GPU
  work completion but explicitly states it is **simulating the work** rather than running
  kernels.
- `riemann-solver/src/bin/cuda_validation_with_db.rs`: multi-GPU GUE control loop builds
  results with hard-coded `computation_time_ms` and comments that GPU computation is
  **simulated**.
- `riemann-solver/src/bin/mechanism_investigation.rs`: prime-gap crossover analysis uses a
  **simulated correlation** value with a comment noting actual data is needed.
- `riemann-solver/src/analysis/primes.rs` (tests): uses **mock zeros** (π-spaced) for a test
  rather than real zeros.

## Project notes

- `GPU_WORKER_REVIEW.md`: explicitly labels the GPU code as **fake** (Rayon CPU
  multi-threading rather than GPU).
- `IMPLEMENTATION_PLAN.md`: calls out a **placeholder** in `analysis/spectral.rs` that was
  never implemented.
