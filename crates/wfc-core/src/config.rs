/// Configuration for WFC generation.
#[derive(Clone, Debug)]
pub struct Config {
    /// Pattern size (N in NxN)
    pub pattern_size: usize,
    /// Output width in cells
    pub output_width: usize,
    /// Output height in cells
    pub output_height: usize,
    /// Whether to wrap input sampling around edges
    pub periodic_input: bool,
    /// Whether to wrap output around edges
    pub periodic_output: bool,
    /// Whether to include rotations/reflections
    pub symmetry: bool,
    /// Constrain patterns at top/bottom edges based on sample position
    pub ground: bool,
    /// Constrain patterns at left/right edges based on sample position
    pub sides: bool,
    /// Optional seed for deterministic output
    pub seed: Option<u64>,
    /// Bias pattern selection toward more flexible choices
    pub use_flexibility: bool,
    /// Enable backtracking on contradiction
    pub backtracking: bool,
    /// Maximum number of backtracks before giving up
    pub max_backtracks: usize,
    /// Save a snapshot every N collapses for backtracking
    pub snapshot_interval: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            pattern_size: 3,
            output_width: 32,
            output_height: 32,
            periodic_input: true,
            periodic_output: false,
            symmetry: true,
            ground: false,
            sides: false,
            seed: None,
            use_flexibility: true,
            backtracking: true,
            max_backtracks: 100,
            snapshot_interval: 10,
        }
    }
}
