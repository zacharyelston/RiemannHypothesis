# Model Comparison: Rigidity Analysis

## Problem Identified

The `unfold_spectrum` method in `SpacingAnalyzer` is designed for spacing analysis, not rigidity metrics.

For rigidity (Σ²(L), Δ₃(L)), we need:
- **Unfolded energy levels** (cumulative positions)
- Not just spacings

The zeros work because `unfold_zeros` uses the Riemann-von Mangoldt formula to create proper unfolded levels.

For GUE/BK/Born, we need to:
1. Sort eigenvalues
2. Map to cumulative index: E_n → n (where n is the position)
3. This gives mean spacing = 1 automatically

## Quick Test: Do Models Need Unfolding?

Actually, for **generic quantum systems**, the eigenvalues ARE already the "unfolded" spectrum if we just use their indices.

For a system with N eigenvalues E₁ < E₂ < ... < Eₙ:
- Unfolded level: n (the index)
- This automatically has mean spacing = 1

The zeros are special because they need N(T) unfolding to account for the non-uniform density.

## Revised Approach

For model comparison, I should:
1. **Zeros**: Use N(T) unfolding (already done)
2. **GUE/BK/Born**: Use index as unfolded level (no unfolding needed)

Then compute Σ²(L) on:
- Zeros: unfolded via N(T)
- Models: indices 1, 2, 3, ..., N

## Why This Matters

If GUE/BK/Born show the **same crossover** at L≈3-5 as the zeros, it means:
- Our models capture the arithmetic structure
- The crossover is universal to spectral approaches

If they show **different behavior**, it means:
- Zeros have unique properties
- Our models are missing something fundamental
- The crossover is specific to the zeta function

## Next Step

Implement proper rigidity comparison:
1. Load zeros, unfold with N(T) → get unfolded levels
2. Generate GUE eigenvalues → use indices as unfolded levels
3. Generate BK eigenvalues → use indices as unfolded levels
4. Generate Born eigenvalues → use indices as unfolded levels
5. Compute Σ²(L) for all four on same L range
6. Compare ratios

This will tell us if the L≈3-5 crossover is:
- **Universal** (all systems show it) → models work
- **Unique to zeros** (only zeros show it) → models incomplete

## Current Status

- ✓ Zeros analyzed: Crossover at L≈3-5 confirmed
- ✗ Models: Unfolding broken (using wrong method)
- ⏳ Need: Proper index-based unfolding for models
- ⏳ Need: Side-by-side comparison

## The Critical Question

**Do Born oscillator eigenvalues show the same L≈3-5 crossover as the zeros?**

If YES → We've captured the essence of arithmetic quantum chaos
If NO → We're missing something fundamental about how primes organize spectra
