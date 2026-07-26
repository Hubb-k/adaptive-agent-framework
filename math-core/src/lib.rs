use std::f64::consts::PI;

const PHI: f64 = 1.61803398875;

pub fn calculate_noise_profile(input: f64) -> f64 {
    let wave1 = (input * PI).cos();
    let wave2 = (input * PI * PHI).cos();
    let wave3 = (input * PI / PHI).cos();
    let interference = (wave1 + wave2 + wave3).abs() / 3.0;
    interference.powf(PHI).clamp(0.0, 1.0)
}

pub fn calculate_alignment_score(impacts: &[f64], setpoint: f64) -> f64 {
    if impacts.is_empty() {
        return 0.0;
    }
    let sum: f64 = impacts
        .iter()
        .map(|&impact| 1.0 / (1.0 + (impact - setpoint).abs()))
        .sum();
    (sum / impacts.len() as f64).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_profile_bounds() {
        for i in 0..100 {
            let val = calculate_noise_profile(i as f64 * 0.1);
            assert!((0.0..=1.0).contains(&val));
        }
    }

    #[test]
    fn test_alignment_perfect_match() {
        let impacts = vec![0.5, 0.5, 0.5];
        let score = calculate_alignment_score(&impacts, 0.5);
        assert!((score - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_alignment_total_mismatch() {
        let impacts = vec![0.0, 1.0, 0.0];
        let score = calculate_alignment_score(&impacts, 0.5);
        assert!(score < 0.7);
    }

    #[test]
    fn test_alignment_empty() {
        let score = calculate_alignment_score(&[], 0.5);
        assert_eq!(score, 0.0);
    }
}
