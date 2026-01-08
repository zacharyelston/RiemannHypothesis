// CUDA kernel for rigidity metrics computation
// Computes Σ²(L) and Δ₃(L) on GPU for massive parallelization

extern "C" __global__ void compute_number_variance(
    const float* unfolded_levels,
    const float* l_values,
    float* results,
    int num_levels,
    int num_l_values
) {
    int l_idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (l_idx >= num_l_values) return;
    
    float l = l_values[l_idx];
    float sum_sq = 0.0f;
    int count = 0;
    
    // Sample starting points
    int step = max(1, num_levels / 100);
    for (int i = 0; i < num_levels - 1; i += step) {
        float s = unfolded_levels[i];
        float s_plus_l = s + l;
        
        // Count levels in [s, s+L]
        int n_in_interval = 0;
        for (int j = 0; j < num_levels; j++) {
            if (unfolded_levels[j] >= s && unfolded_levels[j] < s_plus_l) {
                n_in_interval++;
            }
        }
        
        float deviation = (float)n_in_interval - l;
        sum_sq += deviation * deviation;
        count++;
    }
    
    results[l_idx] = (count > 0) ? (sum_sq / (float)count) : 0.0f;
}

extern "C" __global__ void compute_dyson_mehta(
    const float* unfolded_levels,
    const float* l_values,
    float* results,
    int num_levels,
    int num_l_values
) {
    int l_idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (l_idx >= num_l_values) return;
    
    float l = l_values[l_idx];
    float min_deviation = 1e10f;
    
    // Sample starting points
    int step = max(1, num_levels / 50);
    for (int i = 0; i < num_levels - 1; i += step) {
        float s_start = unfolded_levels[i];
        float s_end = s_start + l;
        
        // Find levels in window
        int window_count = 0;
        for (int j = 0; j < num_levels; j++) {
            if (unfolded_levels[j] >= s_start && unfolded_levels[j] <= s_end) {
                window_count++;
            }
        }
        
        if (window_count < 3) continue;
        
        // Simple linear regression approximation
        float sum_n = 0.0f, sum_s = 0.0f, sum_ns = 0.0f, sum_n2 = 0.0f;
        int count = 0;
        
        for (int j = 0; j < num_levels; j++) {
            if (unfolded_levels[j] >= s_start && unfolded_levels[j] <= s_end) {
                float n = (float)count;
                float s = unfolded_levels[j];
                sum_n += n;
                sum_s += s;
                sum_ns += n * s;
                sum_n2 += n * n;
                count++;
            }
        }
        
        float denominator = sum_n2 * (float)count - sum_n * sum_n;
        float slope = (fabsf(denominator) > 1e-10f) ? 
                     ((sum_ns * (float)count - sum_n * sum_s) / denominator) : 0.0f;
        float intercept = (sum_s - slope * sum_n) / (float)count;
        
        // Compute deviation
        float deviation = 0.0f;
        count = 0;
        for (int j = 0; j < num_levels; j++) {
            if (unfolded_levels[j] >= s_start && unfolded_levels[j] <= s_end) {
                float n = (float)count;
                float predicted = slope * n + intercept;
                float diff = unfolded_levels[j] - predicted;
                deviation += diff * diff;
                count++;
            }
        }
        
        float delta3_val = deviation / l;
        min_deviation = fminf(min_deviation, delta3_val);
    }
    
    results[l_idx] = min_deviation;
}

extern "C" __global__ void bootstrap_sample(
    const float* original_data,
    float* sample_data,
    const int* random_indices,
    int data_size
) {
    int idx = blockIdx.x * blockDim.x + threadIdx.x;
    if (idx >= data_size) return;
    
    sample_data[idx] = original_data[random_indices[idx]];
}
