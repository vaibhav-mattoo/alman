pub trait Tokenizer: Send + Sync {
    fn tokenize(&self, cmd: &str) -> Vec<String>;
}

pub struct ShlexTokenizer;

impl Tokenizer for ShlexTokenizer {
    fn tokenize(&self, cmd: &str) -> Vec<String> {
        shlex::split(cmd).unwrap_or_else(|| cmd.split_whitespace().map(str::to_owned).collect())
    }
}
