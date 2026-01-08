import mpmath
import numpy as np
import matplotlib.pyplot as plt
from scipy import stats

def get_zeta_zeros(n_zeros):
    """
    Compute the first n_zeros imaginary parts of the non-trivial zeros.
    """
    print(f"Computing first {n_zeros} non-trivial zeros...")
    zeros = []
    # mpmath.zetazero(n) returns the n-th zero
    for i in range(1, n_zeros + 1):
        z = mpmath.zetazero(i)
        zeros.append(float(z.imag))
    return np.array(zeros)

def compute_spacings(zeros):
    """
    Compute normalized spacings between consecutive zeros.
    The average spacing at height t is 2*pi / log(t/(2*pi)).
    To normalize, we multiply the difference by the local density.
    """
    spacings = np.diff(zeros)
    
    # Normalize
    # The density of zeros at height T is approx (1/(2*pi)) * log(T/(2*pi))
    # We want the mean spacing to be 1.
    
    # Using the local density for normalization
    normalized_spacings = []
    for i in range(len(spacings)):
        t = zeros[i]
        # Average density d(t) ~ (1/2pi) * log(t/2pi)
        density = (1 / (2 * np.pi)) * np.log(t / (2 * np.pi))
        normalized_s = spacings[i] * density
        normalized_spacings.append(normalized_s)
        
    return np.array(normalized_spacings)

def gue_distribution(s):
    """
    The GUE pair correlation prediction for nearest neighbor spacings
    is approximated by the Wigner Surmise.
    p(s) = (32/pi^2) * s^2 * exp(-4*s^2/pi)
    """
    return (32 / np.pi**2) * (s**2) * np.exp(-4 * s**2 / np.pi)

def main():
    # Number of zeros to compute
    # Note: mpmath is precise but can be slow for very large N. 
    # 1000 is enough to see the trend, though Odlyzko used billions.
    N_ZEROS = 1000 
    
    zeros = get_zeta_zeros(N_ZEROS)
    spacings = compute_spacings(zeros)
    
    # Filter outliers if any (rare with correct normalization)
    spacings = spacings[spacings < 5]
    
    print(f"Mean spacing: {np.mean(spacings):.4f} (Expected ~1.0)")
    
    # Plotting
    plt.figure(figsize=(10, 6))
    
    # Histogram of actual zeros
    plt.hist(spacings, bins=30, density=True, alpha=0.6, color='blue', label='Zeta Zeros Spacings')
    
    # GUE Prediction
    x = np.linspace(0, 4, 100)
    y_gue = gue_distribution(x)
    plt.plot(x, y_gue, 'r-', linewidth=2, label='GUE Prediction (Wigner Surmise)')
    
    # Poisson Prediction (for comparison - random unrelated numbers)
    y_poisson = np.exp(-x)
    plt.plot(x, y_poisson, 'g--', linewidth=2, label='Poisson (Random Uncorrelated)')
    
    plt.title(f"Level Spacing Distribution of First {N_ZEROS} Riemann Zeros")
    plt.xlabel("Normalized Spacing (s)")
    plt.ylabel("Probability Density P(s)")
    plt.legend()
    plt.grid(True, alpha=0.3)
    
    output_file = "zeta_spacing_distribution.png"
    plt.savefig(output_file)
    print(f"Plot saved to {output_file}")

if __name__ == "__main__":
    main()
