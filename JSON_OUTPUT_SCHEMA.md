# JSON Output Schema

## Purpose
Standard JSON format for all commands to enable:
- Automated plotting and comparison
- Reproducibility
- Data pipeline integration
- External analysis tools

## Schema Design

### Common Fields (All Commands)
```json
{
  "command": "string",           // Command name
  "timestamp": "ISO8601",         // When run
  "parameters": {},               // Command-specific parameters
  "eigenvalues": [float],         // Optional: eigenvalues if computed
  "spacings": [float],            // Optional: unfolded spacings
  "statistics": {},               // Command-specific statistics
  "metadata": {}                  // Additional info
}
```

### Command-Specific Schemas

#### 1. verify-gue
```json
{
  "command": "verify-gue",
  "timestamp": "2026-01-08T18:00:00Z",
  "parameters": {
    "size": 300,
    "hbar": 1.0
  },
  "eigenvalues": [0.123, 0.456, ...],
  "spacings": [0.987, 1.234, ...],
  "statistics": {
    "mean_spacing": 1.004,
    "variance": 0.286,
    "skewness": 0.612,
    "kurtosis": 0.245,
    "ks_statistic": 0.042,
    "ks_pvalue": 0.234
  },
  "metadata": {
    "gue_theory": {
      "mean": 1.0,
      "variance": 0.286
    }
  }
}
```

#### 2. berry-keating
```json
{
  "command": "berry-keating",
  "timestamp": "2026-01-08T18:00:00Z",
  "parameters": {
    "truncation": 50
  },
  "eigenvalues": [1.234, 2.345, ...],
  "statistics": {
    "all_real": true,
    "min_eigenvalue": 1.234,
    "max_eigenvalue": 45.678
  },
  "metadata": {
    "theorem": "Srednicki 2011 - Local Riemann Hypothesis",
    "note": "All eigenvalues have Re(s) = 1/2"
  }
}
```

#### 3. born-oscillator
```json
{
  "command": "born-oscillator",
  "timestamp": "2026-01-08T18:00:00Z",
  "parameters": {
    "lambda": 1.0,
    "truncation": 20,
    "hbar": 1.0,
    "order": 1
  },
  "eigenvalues": [1.436, 2.185, ...],
  "metadata": {
    "quantization": "Weyl (Σ₀ + Σ₁)",
    "paper": "Giordano et al. 2023 arXiv:2307.15025v2"
  }
}
```

#### 4. zeta-zeros
```json
{
  "command": "zeta-zeros",
  "timestamp": "2026-01-08T18:00:00Z",
  "parameters": {
    "data_file": "../data/riemann_zeros_first100.txt",
    "count": 100
  },
  "zeros": [14.134725, 21.022040, ...],
  "unfolded_levels": [0.0, 1.234, ...],
  "spacings": [1.234, 0.987, ...],
  "statistics": {
    "mean_spacing": 1.004,
    "variance": 0.283,
    "skewness": 0.598,
    "kurtosis": 0.234,
    "ks_statistic": 0.136,
    "ks_pvalue": 0.051
  },
  "rigidity": {
    "L": 10.0,
    "number_variance": {
      "observed": 0.513,
      "gue_predicted": 0.839,
      "ratio": 0.611
    },
    "delta_3": {
      "observed": 0.285,
      "gue_predicted": 0.413,
      "ratio": 0.690
    }
  },
  "metadata": {
    "phenomenon": "Montgomery-Odlyzko",
    "conclusion": "Riemann zeros match GUE statistics"
  }
}
```

## Implementation Notes

- Use `serde_json` for serialization
- Pretty-print by default for readability
- Validate schema before writing
- Include error handling for file I/O
- Optional: compress large datasets (gzip)

## Usage Examples

```bash
# Generate JSON output
riemann-solver verify-gue --size 300 --out results.json

# Compare multiple runs
riemann-solver born-oscillator --order 0 --out order0.json
riemann-solver born-oscillator --order 1 --out order1.json

# Pipeline integration
riemann-solver zeta-zeros --count 1000 --out zeros.json | jq '.statistics'
```
