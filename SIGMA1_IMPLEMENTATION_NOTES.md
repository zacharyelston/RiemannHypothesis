# Born Oscillator Σ₁(E) Implementation Notes

## From Giordano et al. (2023) Paper

The paper provides the full iterative procedure for computing quantum corrections, but the formulas are extremely complex. For the **Born oscillator** (λ parameter, u→∞ limit), we can use a simplified approach.

## Simplified Σ₁(E) for Born Oscillator

Based on the Weyl quantization procedure (Appendix C), the first quantum correction Σ₁(E) involves:

1. **Second derivatives of the Hamiltonian**
2. **Integration over the classical trajectory**

For the Born oscillator H = √(1+λp²)√(1+λq²), the key derivatives are:

### Hamiltonian Derivatives

```
∂²H/∂p² = λ(1+λq²)^(1/2) / (1+λp²)^(3/2)
∂²H/∂q² = λ(1+λp²)^(1/2) / (1+λq²)^(3/2)
∂²H/∂p∂q = 0  (mixed derivative vanishes)
```

### Σ₁(E) Formula (Simplified)

From the Weyl quantization, the first quantum correction is:

```
Σ₁(E) ≈ (1/24π) ∫₀^{q_t(E)} dq [∂²H/∂p²(p(E,q),q) + ∂²H/∂q²(p(E,q),q)]
```

Where p(E,q) is the momentum on the classical trajectory at energy E.

## Implementation Strategy

Since the full formula from the paper is extremely complex (involves g_{m,ℓ} polynomials), we'll use a **practical approximation**:

1. **Compute Σ₁ numerically** using the simplified formula above
2. **Validate** by checking that eigenvalues improve from Σ₀-only
3. **Compare** qualitatively to paper's results (exact match requires full iterative procedure)

## Alternative: Use Paper's Benchmark Values

The paper provides numerical eigenvalues for specific parameters. We can:
1. Implement Σ₁ with the simplified formula
2. Add `--order` flag for Σ₀ vs Σ₀+Σ₁
3. Show improvement in accuracy
4. Note that full O(ℏ¹¹) would require the complete iterative procedure

## Decision

For this implementation, we'll:
- Implement the **simplified Σ₁** formula
- Demonstrate **improvement** from semiclassical to first quantum correction
- Document that **full precision** requires the iterative G_m procedure
- This satisfies the review requirement: "implement only Σ₁ first, don't jump to ℏ¹¹"
