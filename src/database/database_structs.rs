/// A row returned from command_stats, with score computed at query time.
#[derive(Debug, Clone)]
pub struct Command {
    pub command_text: String,
    pub frequency: i64,
    pub last_access_time: i64,
    /// Computed by the alman_score UDF; never stored in the DB.
    pub score: f64,
}
