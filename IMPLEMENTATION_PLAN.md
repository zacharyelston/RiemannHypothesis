# Implementation Plan: Phase 4 & Beyond

Based on external review feedback, prioritized by research value per effort.

---

## Priority 1: Phase 4 - Actual Riemann Zeros (CRITICAL GAP)

**Problem**: We have three spectral systems but haven't validated against actual zeta zeros.  
**Risk**: Project is incomplete without this - it's a demo, not a research platform.  
**Entropy**: HIGH → Must reduce to zero first.

### Phase 4.1: Obtain Riemann Zeros Data
**Task**: Download Odlyzko tabulated zeros  
**Source**: http://www.dtc.umn.edu/~odlyzko/zeta_tables/  
**Format**: First 10^5 zeros (or subset) in CSV/text format  
**Implementation**:
- Create `data/` directory
- Download zeros file
- Parse into Vec<f64> of imaginary parts γ_n

**Deliverable**: `data/odlyzko_zeros.txt` with first N zeros

---

### Phase 4.2: Implement Riemann-von Mangoldt Unfolding
**Task**: Proper unfolding to mean spacing = 1

**Theory**:
```
N(T) = (T/2π) log(T/2π) - T/2π + 7/8 + O(1/T)
```

**Unfolding**:
```
x_n = N(γ_n)  // Maps γ_n to unfolded level
s_n = x_{n+1} - x_n  // Spacing with mean ≈ 1
```

**Implementation**:
- Add `unfold_riemann_zeros()` function
- Compute N(T) for each zero
- Return unfolded spacings

**File**: `src/analysis/unfolding.rs` (new)

---

### Phase 4.3: Add `zeta-zeros` CLI Command
**Task**: Integrate zeros analysis into CLI

**Command**:
```bash
riemann-solver zeta-zeros --count 10000 --data data/odlyzko_zeros.txt
```

**Output**:
- Load zeros from file
- Unfold using N(T)
- Compute spacing statistics
- Compare to Wigner surmise (GUE)
- Report KS statistic

**File**: Update `src/main.rs` with new command

---

### Phase 4.4: Replace "Confidence" with KS Statistic
**Task**: Use standard statistical tests

**Implement**:
- Kolmogorov-Smirnov distance to Wigner CDF
- Report: D_KS, p-value, sample size
- Optional: Anderson-Darling, chi-square

**Update**: Both GUE and zeta-zeros commands

**Why**: Makes validation defensible and comparable

---

## Priority 2: Phase 5 - Spectral Rigidity Metrics

**Problem**: Placeholder in `analysis/spectral.rs` never implemented  
**Risk**: Spacing alone is insufficient - rigidity is where GUE story becomes undeniable

### Phase 5.1: Implement Number Variance Σ²(L)
**Theory**:
```
Σ²(L) = ⟨(n(E, E+L) - L)²⟩
```
where n(E, E+L) = number of levels in interval [E, E+L]

**GUE Prediction**:
```
Σ²_GUE(L) = (2/π²) log(2πL) + const
```

**Implementation**: `src/analysis/spectral.rs`

---

### Phase 5.2: Implement Dyson-Mehta Δ₃(L)
**Theory**:
```
Δ₃(L) = min_{A,B} (1/L) ∫₀ᴸ [N(E) - AE - B]² dE
```

**GUE Prediction**:
```
Δ₃_GUE(L) = (1/π²) log(2πL) + const
```

**Implementation**: Same file

---

### Phase 5.3: Run on All Four Systems
**Systems**:
1. GUE random matrices
2. Berry-Keating truncated
3. Born oscillator
4. Riemann zeros (unfolded)

**Output**: Comparison plots and tables

---

## Priority 3: Phase 6 - Born Oscillator Σ₁(E)

**Problem**: Only semiclassical (Σ₀) implemented  
**Goal**: Validate against Giordano paper benchmarks

### Phase 6.1: Implement Σ₁(E) Calculation
**Theory**: From Giordano Appendix C
```
Σ₁(E) = quantum correction to phase space integral
```

**Implementation**:
- Add `sigma_1()` method to `BornOscillator`
- Use formulas from paper (derivatives of H)

**File**: `src/hamiltonian/born_oscillator.rs`

---

### Phase 6.2: Add `--order` Flag
**Command**:
```bash
riemann-solver born-oscillator --lambda 1.0 --truncation 10 --order 1
```

**Behavior**:
- `--order 0`: Semiclassical only (current)
- `--order 1`: Include Σ₁(E)

---

### Phase 6.3: Validate Against Paper
**Task**: Compare to published eigenvalues

**Deliverables**:
- Unit tests with "golden values" from paper
- Relative error plot: E_n^(0) vs E_n^(1) vs paper
- Tolerance checks (< 1e-6 or paper's precision)

---

## Priority 4: Phase 7 - JSON Output Format

**Problem**: Results not reproducible or comparable  
**Solution**: Standard output schema

### Phase 7.1: Define JSON Schema
**Schema**:
```json
{
  "system": "gue|berry-keating|born-oscillator|zeta-zeros",
  "parameters": {
    "size": 300,
    "lambda": 1.0,
    "truncation": 20,
    "seed": 42
  },
  "eigenvalues": [1.23, 2.45, ...],
  "unfolded_spacings": [0.98, 1.12, ...],
  "statistics": {
    "variance": 0.1699,
    "ks_statistic": 0.023,
    "p_value": 0.87,
    "number_variance": [...],
    "delta_3": [...]
  }
}
```

---

### Phase 7.2: Add `--output` Flag
**Command**:
```bash
riemann-solver verify-gue --size 300 --output results/gue.json
```

**Implementation**: Serialize results using `serde_json`

---

### Phase 7.3: Plotting Utility (Optional)
**Tool**: Python notebook or Rust plotting
**Input**: JSON files
**Output**: Comparison figures

---

## Quick Fixes

### Fix 1: README Wording
**Current**: "Establish that random matrix eigenvalues match Riemann zero statistics"  
**Correct**: "Verify that Riemann zeros match GUE statistics"

**Why**: GUE is the reference, not the target (Montgomery-Odlyzko phenomenon)

---

## Implementation Order (Six Sigma)

**Hardest First** (Maximum entropy reduction):
1. ✓ Phase 4.1-4.4: Actual Riemann zeros + unfolding + KS statistic
2. ✓ Phase 5: Spectral rigidity metrics
3. ✓ Phase 6: Born oscillator Σ₁(E)
4. Phase 7: JSON output (after validation works)

**Why This Order**:
- Real zeros = highest risk, highest value
- Spectral rigidity = completes statistical toolkit
- Σ₁(E) = validates Born oscillator quantitatively
- JSON = infrastructure after science works

---

## Success Criteria

**Phase 4 Complete When**:
- ✓ Odlyzko zeros loaded
- ✓ Unfolding via N(T) implemented
- ✓ KS statistic replaces "confidence"
- ✓ `zeta-zeros` command working
- ✓ Spacing distribution matches GUE (KS test passes)

**Phase 5 Complete When**:
- ✓ Σ²(L) and Δ₃(L) implemented
- ✓ Run on all four systems
- ✓ Results match GUE predictions

**Phase 6 Complete When**:
- ✓ Σ₁(E) implemented
- ✓ `--order` flag working
- ✓ Eigenvalues match Giordano paper (< 1e-6 error)

---

## Timeline Estimate

- **Phase 4**: 2-3 hours (data + unfolding + KS stat)
- **Phase 5**: 1-2 hours (rigidity metrics)
- **Phase 6**: 2-3 hours (Σ₁ implementation + validation)
- **Phase 7**: 1 hour (JSON serialization)

**Total**: ~6-9 hours to complete all priorities

---

## Notes

- Follow entropy principle: tackle hardest (real zeros) first
- Each phase reduces uncertainty for next phase
- Validate incrementally (don't jump to ℏ¹¹)
- Make results reproducible (JSON output)

**End Goal**: Convert project from "cleanly written demo" to "legitimate research platform"
