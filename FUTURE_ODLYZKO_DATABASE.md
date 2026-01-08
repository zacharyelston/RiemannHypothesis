# Future Enhancement: Odlyzko Database Integration

## SageMath Database
**Source**: https://doc.sagemath.org/html/en/reference/databases/sage/databases/odlyzko.html

**Available Data**:
- **2,001,052 zeros** of Riemann zeta function
- Accuracy: 4e-9
- Accessible via SageMath's `zeta_zeros()` function

## Current Implementation
- Using first 100 known zeros (manually entered)
- File: `data/riemann_zeros_first100.txt`
- Sufficient for Phase 4 validation

## Future Enhancement Options

### Option 1: SageMath Integration
```bash
sage -i database_odlyzko_zeta
```

```python
from sage.all import *
zz = zeta_zeros()
# Export to file for Rust consumption
with open('data/odlyzko_zeros_2M.txt', 'w') as f:
    for z in zz:
        f.write(f"{z}\n")
```

### Option 2: Direct Download
- Source: http://www.dtc.umn.edu/~odlyzko/zeta_tables/index.html
- Download pre-computed tables
- Parse into our format

### Option 3: mpmath Computation
```python
import mpmath
mpmath.mp.dps = 50  # 50 decimal places

zeros = []
for n in range(1, 10001):
    zero = mpmath.zetazero(n)
    zeros.append(float(zero.imag))
```

## Benefits of Larger Dataset

**Statistical Power**:
- Current: 99 spacings (N=100)
- With 2M zeros: ~2M spacings
- Much tighter confidence intervals on KS test
- Can study higher-order correlations

**Spectral Rigidity**:
- Σ²(L) and Δ₃(L) require large datasets
- Current 100 zeros insufficient for rigidity analysis
- 2M zeros would enable full spectral rigidity study

**Validation**:
- Test unfolding accuracy over wider range
- Study deviations from GUE at different scales
- Compare low vs high zeros

## Implementation Plan (Future)

1. **Phase 4.5**: Download Odlyzko database
   - Use SageMath or direct download
   - Store in `data/odlyzko_zeros_full.txt`
   
2. **Phase 5**: Spectral Rigidity
   - Implement Σ²(L) using large dataset
   - Implement Δ₃(L) 
   - Compare to GUE predictions

3. **Phase 6**: Scale Analysis
   - Study spacing statistics vs zero height
   - Test Montgomery-Odlyzko at different scales
   - Look for deviations from GUE

## Current Status

✓ Phase 4 complete with 100 zeros
✓ KS test validates GUE match (p=0.051)
✓ Infrastructure ready for larger datasets

**Next**: Can drop in larger dataset without code changes
- Just replace data file
- All analysis code already supports arbitrary N
- KS test, unfolding, spacing all scale to millions

## Notes

The current 100-zero implementation is **sufficient for validation** but not for research-grade analysis. The infrastructure is built to scale - we can upgrade the dataset anytime without changing code.

For Phase 5 (spectral rigidity), we'll need the larger dataset to get meaningful Σ²(L) and Δ₃(L) curves.
