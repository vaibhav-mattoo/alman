pub struct TemplateStats {
    pub support: u32,
    pub distinct_per_slot: Vec<u32>,
    pub literal_len: usize, // total chars in all Literal parts
    pub name_len: usize,    // suggested name length (passed in from caller)
}

pub trait TemplateScorer: Send + Sync {
    fn score(&self, s: &TemplateStats) -> f64;
}

pub struct SavingsScorer;

const SAVINGS_BASE: f64 = 1.0;
const DISTINCT_BOOST: f64 = 0.1;

impl TemplateScorer for SavingsScorer {
    fn score(&self, s: &TemplateStats) -> f64 {
        let savings = (s.literal_len.saturating_sub(s.name_len)) as f64;
        let min_distinct = s.distinct_per_slot.iter().copied().min().unwrap_or(0) as f64;
        (s.support as f64) * (SAVINGS_BASE + savings) * (1.0 + DISTINCT_BOOST * min_distinct)
    }
}
