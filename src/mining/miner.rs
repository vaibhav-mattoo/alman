use crate::mining::scorer::{TemplateScorer, TemplateStats};
use crate::template::{CommandTemplate, TemplatePart};

const MIN_SUPPORT: usize = 3;
const MIN_DISTINCT_SLOT_VALUES: usize = 2;
const MAX_SLOTS: usize = 2;
const MAX_TEMPLATE_TOKENS: usize = 8;

pub struct MinedTemplate {
    pub template: CommandTemplate,
    pub stats: TemplateStats,
    pub score: f64,
}

pub trait TemplateMiner: Send + Sync {
    fn mine(&self, tokenized_events: &[Vec<String>]) -> Vec<MinedTemplate>;
}

pub struct FixedArityMiner {
    scorer: Box<dyn TemplateScorer>,
}

impl FixedArityMiner {
    pub fn new(scorer: Box<dyn TemplateScorer>) -> Self {
        Self { scorer }
    }
}

impl TemplateMiner for FixedArityMiner {
    fn mine(&self, tokenized_events: &[Vec<String>]) -> Vec<MinedTemplate> {
        use std::collections::HashMap;
        // Bucket by (command[0], token_count)
        let mut buckets: HashMap<(String, usize), Vec<Vec<String>>> = HashMap::new();
        for tokens in tokenized_events {
            if tokens.is_empty() {
                continue;
            }
            let key = (tokens[0].clone(), tokens.len());
            buckets.entry(key).or_default().push(tokens.clone());
        }

        let mut results = Vec::new();
        let mut seen_templates: std::collections::HashSet<String> = std::collections::HashSet::new();

        for ((cmd0, len), rows) in &buckets {
            if rows.len() < MIN_SUPPORT {
                continue;
            }
            if *len > MAX_TEMPLATE_TOKENS {
                continue;
            }

            // Per-position analysis
            let mut parts: Vec<TemplatePart> = Vec::with_capacity(*len);
            let mut slot_stats: Vec<u32> = Vec::new();
            let mut slot_count = 0usize;
            let mut reject = false;

            for pos in 0..*len {
                let values: std::collections::HashSet<&str> =
                    rows.iter().map(|r| r[pos].as_str()).collect();
                let distinct = values.len();

                if pos == 0 {
                    // Token 0 is always the command — always literal
                    parts.push(TemplatePart::Literal(cmd0.clone()));
                    continue;
                }

                let any_flag = values.iter().any(|v| v.starts_with('-'));

                if distinct == 1 {
                    // Constant — literal
                    parts.push(TemplatePart::Literal(rows[0][pos].clone()));
                } else if distinct >= MIN_DISTINCT_SLOT_VALUES && !any_flag {
                    // Variable, no flags → slot
                    slot_count += 1;
                    if slot_count > MAX_SLOTS {
                        reject = true;
                        break;
                    }
                    let slot_idx = slot_count as u32;
                    parts.push(TemplatePart::Slot(slot_idx));
                    slot_stats.push(distinct as u32);
                } else {
                    // Varies but flags present, or partially flags — can't template cleanly
                    reject = true;
                    break;
                }
            }

            if reject || slot_count == 0 {
                continue;
            }

            let template = CommandTemplate { parts };
            let tmpl_key = template.to_json();
            if !seen_templates.insert(tmpl_key) {
                continue;
            }

            let literal_len: usize = template
                .parts
                .iter()
                .map(|p| match p {
                    TemplatePart::Literal(s) => s.len(),
                    TemplatePart::Slot(_) => 0,
                })
                .sum();
            let stats = TemplateStats {
                support: rows.len() as u32,
                distinct_per_slot: slot_stats,
                literal_len,
                name_len: 4, // placeholder; caller can refine
            };
            let score = self.scorer.score(&stats);
            results.push(MinedTemplate {
                template,
                stats,
                score,
            });
        }

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mining::scorer::SavingsScorer;

    fn tok(s: &str) -> Vec<String> {
        s.split_whitespace().map(|w| w.to_string()).collect()
    }

    #[test]
    fn mines_interior_slot_template() {
        let events = vec![
            tok("docker exec -it web bash"),
            tok("docker exec -it db bash"),
            tok("docker exec -it cache bash"),
        ];
        let miner = FixedArityMiner::new(Box::new(SavingsScorer));
        let results = miner.mine(&events);
        assert_eq!(results.len(), 1, "expected exactly one mined template");
        let t = &results[0].template;
        // Expect: docker exec -it <slot1> bash
        assert_eq!(t.parts.len(), 5);
        assert_eq!(t.parts[3], TemplatePart::Slot(1));
        assert_eq!(results[0].stats.support, 3);
    }
}
