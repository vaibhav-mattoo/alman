use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TemplatePart {
    Literal(String),
    Slot(u32), // 1-based
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CommandTemplate {
    pub parts: Vec<TemplatePart>,
}

impl CommandTemplate {
    /// Count distinct Slot indices.
    pub fn slot_count(&self) -> u32 {
        let mut seen: HashSet<u32> = HashSet::new();
        for part in &self.parts {
            if let TemplatePart::Slot(n) = part {
                seen.insert(*n);
            }
        }
        seen.len() as u32
    }

    pub fn is_zero_slot(&self) -> bool {
        self.slot_count() == 0
    }

    /// True iff exactly one slot, it is last, and there are no other slots.
    pub fn only_trailing_single_slot(&self) -> bool {
        if self.slot_count() != 1 {
            return false;
        }
        matches!(self.parts.last(), Some(TemplatePart::Slot(1)))
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn from_json(s: &str) -> Option<Self> {
        serde_json::from_str(s).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_count_distinct() {
        let t = CommandTemplate {
            parts: vec![
                TemplatePart::Literal("git".into()),
                TemplatePart::Slot(1),
                TemplatePart::Slot(2),
            ],
        };
        assert_eq!(t.slot_count(), 2);
        assert!(!t.is_zero_slot());
    }

    #[test]
    fn trailing_single_slot_detection() {
        let trailing = CommandTemplate {
            parts: vec![TemplatePart::Literal("git".into()), TemplatePart::Slot(1)],
        };
        assert!(trailing.only_trailing_single_slot());

        let interior = CommandTemplate {
            parts: vec![
                TemplatePart::Literal("docker".into()),
                TemplatePart::Literal("exec".into()),
                TemplatePart::Slot(1),
                TemplatePart::Literal("bash".into()),
            ],
        };
        assert!(!interior.only_trailing_single_slot());

        let zero = CommandTemplate {
            parts: vec![TemplatePart::Literal("git status".into())],
        };
        assert!(!zero.only_trailing_single_slot());
        assert!(zero.is_zero_slot());
    }

    #[test]
    fn json_round_trip() {
        let t = CommandTemplate {
            parts: vec![
                TemplatePart::Literal("docker".into()),
                TemplatePart::Literal("exec".into()),
                TemplatePart::Slot(1),
                TemplatePart::Literal("bash".into()),
            ],
        };
        let json = t.to_json();
        let back = CommandTemplate::from_json(&json).expect("round trip");
        assert_eq!(t, back);
    }
}
