/// Constraint checking task implementations

/// Simple constraint check: verifies that values fall within bounds
pub fn check_bounds_i32(data: &[i32], min: i32, max: i32) -> Vec<usize> {
    data.iter()
        .enumerate()
        .filter_map(|(i, &v)| if v < min || v > max { Some(i) } else { None })
        .collect()
}

/// Constraint check using Eisenstein integer norm
/// For integers a, b: norm = a² - ab + b²
pub fn eisenstein_norm(a: i64, b: i64) -> i64 {
    a * a - a * b + b * b
}

/// Batch constraint check — find all (a,b) pairs where eisenstein_norm <= threshold
pub fn eisenstein_filter(pairs: &[(i64, i64)], threshold: i64) -> Vec<usize> {
    pairs.iter()
        .enumerate()
        .filter_map(|(i, &(a, b))| {
            let norm = eisenstein_norm(a, b);
            if norm <= threshold { Some(i) } else { None }
        })
        .collect()
}

/// Run a benchmark constraint check
pub fn bench_constraint_i32(count: usize) -> (u64, std::time::Duration) {
    let data: Vec<i32> = (0..count as i32).map(|i| i.wrapping_mul(17)).collect();
    let start = std::time::Instant::now();
    let violations = check_bounds_i32(&data, -1000, 1000);
    let elapsed = start.elapsed();
    (violations.len() as u64, elapsed)
}
