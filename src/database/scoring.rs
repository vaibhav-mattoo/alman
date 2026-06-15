/// Query-time relevance score — mirrors the `alman_score` SQLite UDF.
///
/// Recency multipliers:
///   ≤ 1 h  → 4.0 | ≤ 24 h → 2.0 | ≤ 7 d → 0.5 | older → 0.25
pub fn score(frequency: f64, last_access: i64, length: f64, now: i64) -> f64 {
    let diff = now - last_access;
    let mult: f64 = if diff <= 3_600 {
        4.0
    } else if diff <= 86_400 {
        2.0
    } else if diff <= 604_800 {
        0.5
    } else {
        0.25
    };
    mult * length.powf(0.6) * frequency
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_000_000;

    fn s(freq: f64, diff: i64, len: f64) -> f64 {
        score(freq, NOW - diff, len, NOW)
    }

    #[test]
    fn hot_bucket() {
        // diff = 60 ≤ 3600 → multiplier 4.0
        let got = s(1.0, 60, 10.0);
        let want = 4.0 * 10.0_f64.powf(0.6);
        assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
    }

    #[test]
    fn today_bucket() {
        // diff = 7200 (2 h): 3600 < 7200 ≤ 86400 → multiplier 2.0
        let got = s(1.0, 7_200, 10.0);
        let want = 2.0 * 10.0_f64.powf(0.6);
        assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
    }

    #[test]
    fn week_bucket() {
        // diff = 172800 (2 d): 86400 < 172800 ≤ 604800 → multiplier 0.5
        let got = s(1.0, 172_800, 10.0);
        let want = 0.5 * 10.0_f64.powf(0.6);
        assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
    }

    #[test]
    fn old_bucket() {
        // diff = 700000 > 604800 → multiplier 0.25
        let got = s(1.0, 700_000, 10.0);
        let want = 0.25 * 10.0_f64.powf(0.6);
        assert!((got - want).abs() < 1e-9, "got {got}, want {want}");
    }

    #[test]
    fn higher_frequency_scores_higher() {
        let low = s(1.0, 60, 10.0);
        let high = s(10.0, 60, 10.0);
        assert!(high > low, "frequency monotonicity violated");
    }

    #[test]
    fn more_recent_scores_higher() {
        let recent = s(1.0, 60, 10.0);    // hot bucket (×4)
        let older = s(1.0, 7_200, 10.0);  // today bucket (×2)
        assert!(recent > older, "recency monotonicity violated");
    }

    #[test]
    fn bucket_boundary_3600() {
        // At exactly 3600 s → still hot bucket
        let at = s(1.0, 3_600, 10.0);
        let just_over = s(1.0, 3_601, 10.0);
        assert!(at > just_over, "boundary at 3600 s violated");
    }

    #[test]
    fn bucket_boundary_86400() {
        let at = s(1.0, 86_400, 10.0);
        let just_over = s(1.0, 86_401, 10.0);
        assert!(at > just_over, "boundary at 86400 s violated");
    }

    #[test]
    fn bucket_boundary_604800() {
        let at = s(1.0, 604_800, 10.0);
        let just_over = s(1.0, 604_801, 10.0);
        assert!(at > just_over, "boundary at 604800 s violated");
    }
}
