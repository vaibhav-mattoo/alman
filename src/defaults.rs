use crate::cli::cli_data::InitShell;
use crate::database::scoring::{RecencyFrequencyScorer, RelevanceScorer};
use crate::mining::miner::{FixedArityMiner, TemplateMiner};
use crate::mining::scorer::{SavingsScorer, TemplateScorer};
use crate::render::{FishRenderer, PosixRenderer, ShellRenderer};
use crate::tokenize::{ShlexTokenizer, Tokenizer};

pub fn default_tokenizer() -> Box<dyn Tokenizer> {
    Box::new(ShlexTokenizer)
}

pub fn default_relevance_scorer() -> Box<dyn RelevanceScorer> {
    Box::new(RecencyFrequencyScorer)
}

pub fn default_template_scorer() -> Box<dyn TemplateScorer> {
    Box::new(SavingsScorer)
}

pub fn default_miner() -> Box<dyn TemplateMiner> {
    Box::new(FixedArityMiner::new(default_template_scorer()))
}

pub fn renderer_for(shell: &InitShell) -> Box<dyn ShellRenderer> {
    match shell {
        InitShell::Fish => Box::new(FishRenderer),
        _ => Box::new(PosixRenderer),
    }
}
